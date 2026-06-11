//! Function inlining optimization pass.
//!
//! Inlines small functions (below a configurable threshold) at their call sites.

use crate::ir::{IrFunction, IrModule, IrOp};
use std::collections::HashMap;

/// Default threshold for inlining: functions with <= this many ops are inlined.
pub const DEFAULT_INLINE_THRESHOLD: usize = 10;

/// Statistics from inlining.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InlineStats {
    pub functions_inlined: usize,
}

/// Run inlining on an entire module.
pub fn inline_module(module: &mut IrModule) -> InlineStats {
    inline_module_with_threshold(module, DEFAULT_INLINE_THRESHOLD)
}

/// Run inlining with a custom threshold.
pub fn inline_module_with_threshold(module: &mut IrModule, threshold: usize) -> InlineStats {
    let mut stats = InlineStats::default();

    // Collect function sizes
    let _func_sizes: HashMap<String, usize> = module
        .functions
        .iter()
        .map(|f| (f.name.clone(), f.count_ops()))
        .collect();

    // Find small functions to inline (excluding the entry point)
    let inlineable: Vec<String> = module
        .functions
        .iter()
        .filter(|f| f.name != module.entry_point && f.count_ops() <= threshold)
        .filter(|f| !f.ops.iter().any(|op| matches!(op, IrOp::Call { .. } | IrOp::Branch { .. } | IrOp::Jump { .. })))
        .map(|f| f.name.clone())
        .collect();

    if inlineable.is_empty() {
        return stats;
    }

    // Clone the functions to inline (we need them while mutating)
    let inline_funcs: HashMap<String, IrFunction> = module
        .functions
        .iter()
        .filter(|f| inlineable.contains(&f.name))
        .map(|f| (f.name.clone(), f.clone()))
        .collect();

    // For each function, try to inline calls to inlineable functions
    for func in &mut module.functions {
        if inline_funcs.contains_key(&func.name) {
            continue; // Don't inline into functions that will be removed
        }

        let mut new_ops = Vec::new();
        let mut reg_offset = func.next_reg();

        for op in &func.ops {
            if let IrOp::Call { dest, func: callee, args } = op {
                if let Some(callee_func) = inline_funcs.get(callee) {
                    // Inline this call
                    let inlined = inline_call(callee_func, dest, args, reg_offset);
                    new_ops.extend(inlined);
                    reg_offset += callee_func.next_reg();
                    stats.functions_inlined += 1;
                    continue;
                }
            }
            new_ops.push(op.clone());
        }

        func.ops = new_ops;
    }

    stats
}

/// Inline a function call, adjusting register numbers.
fn inline_call(
    callee: &IrFunction,
    dest: &Option<usize>,
    args: &[usize],
    reg_offset: usize,
) -> Vec<IrOp> {
    let mut ops = Vec::new();

    // Map parameter registers to argument registers
    let mut param_map: HashMap<usize, usize> = HashMap::new();
    for (i, param) in callee.params.iter().enumerate() {
        if i < args.len() {
            // Load the argument into a register that matches the function's expectation
            let param_reg = i; // params use registers 0, 1, 2, ...
            let mapped_reg = reg_offset + param_reg;
            param_map.insert(param_reg, args[i]);
            // Copy the argument into the mapped register
            ops.push(IrOp::LoadVar {
                dest: mapped_reg,
                name: param.name.clone(),
            });
        }
    }

    // Copy the function's ops with adjusted registers
    for op in &callee.ops {
        let adjusted_op = adjust_registers(op, reg_offset);
        ops.push(adjusted_op);
    }

    // If the function returns a value and the caller expects one, wire it up
    if let Some(dest_reg) = dest {
        // The return value is in the last register of the inlined function
        if let Some(return_op) = callee.ops.iter().find_map(|op| {
            if let IrOp::Return { value: Some(v) } = op {
                Some(*v)
            } else {
                None
            }
        }) {
            // The return register after adjustment
            let adjusted_return = reg_offset + return_op;
            if *dest_reg != adjusted_return {
                // Copy the result to the expected destination
                ops.push(IrOp::BinaryOp {
                    dest: *dest_reg,
                    op: crate::ir::BinaryOp::Add,
                    lhs: adjusted_return,
                    rhs: reg_offset, // we'll load 0 into this
                });
            }
        }
    }

    ops
}

/// Adjust register numbers in an IR op by adding an offset.
fn adjust_registers(op: &IrOp, offset: usize) -> IrOp {
    match op {
        IrOp::LoadConst { dest, value } => IrOp::LoadConst {
            dest: dest + offset,
            value: value.clone(),
        },
        IrOp::LoadVar { dest, name } => IrOp::LoadVar {
            dest: dest + offset,
            name: name.clone(),
        },
        IrOp::StoreVar { name, src } => IrOp::StoreVar {
            name: name.clone(),
            src: src + offset,
        },
        IrOp::BinaryOp { dest, op, lhs, rhs } => IrOp::BinaryOp {
            dest: dest + offset,
            op: *op,
            lhs: lhs + offset,
            rhs: rhs + offset,
        },
        IrOp::UnaryOp { dest, op, operand } => IrOp::UnaryOp {
            dest: dest + offset,
            op: *op,
            operand: operand + offset,
        },
        IrOp::Call { dest, func, args } => IrOp::Call {
            dest: dest.map(|d| d + offset),
            func: func.clone(),
            args: args.iter().map(|a| a + offset).collect(),
        },
        IrOp::Return { value } => IrOp::Return {
            value: value.map(|v| v + offset),
        },
        IrOp::Branch { cond, then_label, else_label } => IrOp::Branch {
            cond: cond + offset,
            then_label: then_label.clone(),
            else_label: else_label.clone(),
        },
        // Labels, jumps, nops, and phis are not adjusted (they reference blocks, not regs)
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::*;

    fn make_small_helper() -> IrFunction {
        let mut func = IrFunction::new("double", IrType::I32);
        func.add_param("x", IrType::I32);
        func.push_op(IrOp::LoadVar { dest: 0, name: "x".into() });
        func.push_op(IrOp::LoadVar { dest: 1, name: "x".into() });
        func.push_op(IrOp::BinaryOp { dest: 2, op: BinaryOp::Add, lhs: 0, rhs: 1 });
        func.push_op(IrOp::Return { value: Some(2) });
        func
    }

    fn make_large_helper() -> IrFunction {
        let mut func = IrFunction::new("big_func", IrType::I32);
        for i in 0..20 {
            func.push_op(IrOp::LoadConst { dest: i, value: ConstValue::I32(i as i32) });
        }
        func.push_op(IrOp::Return { value: Some(19) });
        func
    }

    #[test]
    fn test_inline_small_function() {
        let mut module = IrModule::new("main");
        let mut main_func = IrFunction::new("main", IrType::I32);
        main_func.push_op(IrOp::LoadConst { dest: 0, value: ConstValue::I32(5) });
        main_func.push_op(IrOp::Call {
            dest: Some(1),
            func: "double".into(),
            args: vec![0],
        });
        main_func.push_op(IrOp::Return { value: Some(1) });
        module.add_function(main_func);

        let helper = make_small_helper();
        module.add_function(helper);

        let stats = inline_module(&mut module);
        assert_eq!(stats.functions_inlined, 1);

        // The call should have been replaced
        let main = module.get_function("main").unwrap();
        assert!(!main.ops.iter().any(|op| matches!(op, IrOp::Call { func, .. } if func == "double")));
    }

    #[test]
    fn test_no_inline_large_function() {
        let mut module = IrModule::new("main");
        let mut main_func = IrFunction::new("main", IrType::I32);
        main_func.push_op(IrOp::Call {
            dest: Some(0),
            func: "big_func".into(),
            args: vec![],
        });
        main_func.push_op(IrOp::Return { value: Some(0) });
        module.add_function(main_func);

        module.add_function(make_large_helper());

        let stats = inline_module(&mut module);
        assert_eq!(stats.functions_inlined, 0);
    }

    #[test]
    fn test_no_inline_entry_point() {
        let mut module = IrModule::new("main");
        let mut main_func = IrFunction::new("main", IrType::Void);
        main_func.push_op(IrOp::Return { value: None });
        module.add_function(main_func);

        let stats = inline_module(&mut module);
        assert_eq!(stats.functions_inlined, 0);
    }

    #[test]
    fn test_inline_with_custom_threshold() {
        let mut module = IrModule::new("main");
        let mut main_func = IrFunction::new("main", IrType::I32);
        main_func.push_op(IrOp::Call {
            dest: Some(0),
            func: "big_func".into(),
            args: vec![],
        });
        main_func.push_op(IrOp::Return { value: Some(0) });
        module.add_function(main_func);

        module.add_function(make_large_helper());

        // With a very high threshold, the large function should be inlined
        let stats = inline_module_with_threshold(&mut module, 100);
        // The large function has 21 ops (LoadConst + Return), no Call/Branch/Jump,
        // so it IS inlineable with threshold 100
        assert_eq!(stats.functions_inlined, 1);
    }

    #[test]
    fn test_inline_preserves_other_calls() {
        let mut module = IrModule::new("main");
        let mut main_func = IrFunction::new("main", IrType::Void);
        main_func.push_op(IrOp::Call {
            dest: Some(0),
            func: "double".into(),
            args: vec![],
        });
        main_func.push_op(IrOp::Call {
            dest: None,
            func: "print".into(),
            args: vec![0],
        });
        main_func.push_op(IrOp::Return { value: None });
        module.add_function(main_func);

        let helper = make_small_helper();
        module.add_function(helper);

        let stats = inline_module(&mut module);
        assert_eq!(stats.functions_inlined, 1);

        // The print call should still exist
        let main = module.get_function("main").unwrap();
        assert!(main.ops.iter().any(|op| matches!(op, IrOp::Call { func, .. } if func == "print")));
    }

    #[test]
    fn test_no_inline_recursive() {
        let mut module = IrModule::new("main");
        let mut main_func = IrFunction::new("main", IrType::Void);
        main_func.push_op(IrOp::Call {
            dest: None,
            func: "recurse".into(),
            args: vec![],
        });
        main_func.push_op(IrOp::Return { value: None });
        module.add_function(main_func);

        let mut recurse_func = IrFunction::new("recurse", IrType::Void);
        recurse_func.push_op(IrOp::Call {
            dest: None,
            func: "recurse".into(),
            args: vec![],
        });
        recurse_func.push_op(IrOp::Return { value: None });
        module.add_function(recurse_func);

        let stats = inline_module(&mut module);
        // Recursive functions contain Call ops, so they won't be inlined
        assert_eq!(stats.functions_inlined, 0);
    }

    #[test]
    fn test_adjust_registers() {
        let op = IrOp::BinaryOp { dest: 2, op: BinaryOp::Add, lhs: 0, rhs: 1 };
        let adjusted = adjust_registers(&op, 10);
        assert!(matches!(adjusted, IrOp::BinaryOp { dest: 12, lhs: 10, rhs: 11, .. }));
    }

    #[test]
    fn test_adjust_registers_label() {
        let op = IrOp::Label { name: "start".into() };
        let adjusted = adjust_registers(&op, 10);
        assert!(matches!(adjusted, IrOp::Label { name } if name == "start"));
    }

    #[test]
    fn test_inline_empty_function() {
        let mut module = IrModule::new("main");
        let mut main_func = IrFunction::new("main", IrType::Void);
        main_func.push_op(IrOp::Call {
            dest: None,
            func: "empty".into(),
            args: vec![],
        });
        main_func.push_op(IrOp::Return { value: None });
        module.add_function(main_func);

        let empty_func = IrFunction::new("empty", IrType::Void);
        module.add_function(empty_func);

        let stats = inline_module(&mut module);
        assert_eq!(stats.functions_inlined, 1);
    }

    #[test]
    fn test_inline_constant_return() {
        let mut module = IrModule::new("main");
        let mut main_func = IrFunction::new("main", IrType::I32);
        main_func.push_op(IrOp::Call {
            dest: Some(0),
            func: "forty_two".into(),
            args: vec![],
        });
        main_func.push_op(IrOp::Return { value: Some(0) });
        module.add_function(main_func);

        let mut helper = IrFunction::new("forty_two", IrType::I32);
        helper.push_op(IrOp::LoadConst { dest: 0, value: ConstValue::I32(42) });
        helper.push_op(IrOp::Return { value: Some(0) });
        module.add_function(helper);

        let stats = inline_module(&mut module);
        assert_eq!(stats.functions_inlined, 1);
    }

    #[test]
    fn test_inline_threshold_boundary() {
        // Function with exactly 10 ops (the default threshold)
        let mut module = IrModule::new("main");
        let mut main_func = IrFunction::new("main", IrType::I32);
        main_func.push_op(IrOp::Call {
            dest: Some(0),
            func: "boundary".into(),
            args: vec![],
        });
        main_func.push_op(IrOp::Return { value: Some(0) });
        module.add_function(main_func);

        let mut boundary_func = IrFunction::new("boundary", IrType::I32);
        for i in 0..9 {
            boundary_func.push_op(IrOp::LoadConst { dest: i, value: ConstValue::I32(i as i32) });
        }
        boundary_func.push_op(IrOp::Return { value: Some(8) });
        module.add_function(boundary_func);

        // Exactly 10 ops should be inlined (<= threshold)
        let stats = inline_module(&mut module);
        assert_eq!(stats.functions_inlined, 1);
    }
}
