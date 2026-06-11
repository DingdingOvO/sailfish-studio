//! Dead code elimination optimization pass.
//!
//! Removes:
//! - Unused variables (stored but never loaded)
//! - Unreachable code after Return/Jump
//! - Functions that are never called from the entry point

use crate::ir::{IrFunction, IrModule, IrOp};
use std::collections::HashSet;

/// Statistics from dead code elimination.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DceStats {
    pub unused_vars_removed: usize,
    pub unreachable_ops_removed: usize,
    pub unused_functions_removed: usize,
}

/// Run dead code elimination on an entire module.
pub fn dead_code_eliminate_module(module: &mut IrModule) -> DceStats {
    let mut stats = DceStats::default();

    // Remove unused functions
    let reachable = crate::ir::reachable_functions(module);
    let before_count = module.functions.len();
    module.functions.retain(|f| reachable.contains(&f.name));
    stats.unused_functions_removed = before_count - module.functions.len();

    // Run DCE on each remaining function
    for func in &mut module.functions {
        let func_stats = dead_code_eliminate_function(func);
        stats.unused_vars_removed += func_stats.unused_vars_removed;
        stats.unreachable_ops_removed += func_stats.unreachable_ops_removed;
    }

    stats
}

/// Run dead code elimination on a single function.
pub fn dead_code_eliminate_function(func: &mut IrFunction) -> DceStats {
    let mut stats = DceStats::default();

    // Phase 1: Remove unreachable code after terminators
    let unreachable_stats = remove_unreachable_code(func);
    stats.unreachable_ops_removed += unreachable_stats;

    // Phase 2: Remove unused variable stores
    let var_stats = remove_unused_vars(func);
    stats.unused_vars_removed += var_stats;

    stats
}

/// Remove unreachable code after terminators within basic blocks.
fn remove_unreachable_code(func: &mut IrFunction) -> usize {
    let mut removed = 0;
    let mut new_ops = Vec::new();
    let mut terminated = false;

    for op in &func.ops {
        if terminated {
            // Skip everything after a terminator until we see a label
            if let IrOp::Label { .. } = op {
                terminated = false;
                new_ops.push(op.clone());
            } else {
                removed += 1;
            }
        } else {
            new_ops.push(op.clone());
            if op.is_terminator() {
                terminated = true;
            }
        }
    }

    func.ops = new_ops;
    removed
}

/// Remove stores to variables that are never loaded.
fn remove_unused_vars(func: &mut IrFunction) -> usize {
    let mut removed = 0;

    // Collect all loaded variable names
    let loaded_vars: HashSet<String> = func
        .ops
        .iter()
        .filter_map(|op| match op {
            IrOp::LoadVar { name, .. } => Some(name.clone()),
            _ => None,
        })
        .collect();

    // Remove stores to variables that are never loaded
    // But keep stores that are used in called functions or as side effects
    let new_ops: Vec<IrOp> = func
        .ops
        .iter()
        .filter(|op| {
            if let IrOp::StoreVar { name, .. } = op {
                if !loaded_vars.contains(name) {
                    removed += 1;
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect();

    func.ops = new_ops;
    removed
}

/// Find all registers that are used (read) in a function.
pub fn find_used_registers(func: &IrFunction) -> HashSet<usize> {
    let mut used = HashSet::new();
    for op in &func.ops {
        for reg in op.src_regs() {
            used.insert(reg);
        }
    }
    used
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::*;

    #[test]
    fn test_remove_unreachable_after_return() {
        let mut func = IrFunction::new("f", IrType::Void);
        func.push_op(IrOp::Return { value: None });
        func.push_op(IrOp::LoadConst { dest: 0, value: ConstValue::I32(42) }); // unreachable
        func.push_op(IrOp::Return { value: Some(0) }); // unreachable

        let stats = dead_code_eliminate_function(&mut func);
        assert_eq!(stats.unreachable_ops_removed, 2);
        assert_eq!(func.ops.len(), 1); // Only the first return
    }

    #[test]
    fn test_keep_code_after_label() {
        let mut func = IrFunction::new("f", IrType::Void);
        func.push_op(IrOp::Return { value: None });
        func.push_op(IrOp::Label { name: "restart".into() });
        func.push_op(IrOp::LoadConst { dest: 0, value: ConstValue::I32(1) });
        func.push_op(IrOp::Return { value: Some(0) });

        let stats = dead_code_eliminate_function(&mut func);
        // Only the first return is a terminator, label restarts the block
        assert_eq!(stats.unreachable_ops_removed, 0);
        assert_eq!(func.ops.len(), 4);
    }

    #[test]
    fn test_remove_unused_variable_store() {
        let mut func = IrFunction::new("f", IrType::Void);
        func.push_op(IrOp::LoadConst { dest: 0, value: ConstValue::I32(42) });
        func.push_op(IrOp::StoreVar { name: "unused_var".into(), src: 0 }); // never loaded
        func.push_op(IrOp::Return { value: None });

        let stats = dead_code_eliminate_function(&mut func);
        assert_eq!(stats.unused_vars_removed, 1);
        assert!(!func.ops.iter().any(|op| matches!(op, IrOp::StoreVar { name, .. } if name == "unused_var")));
    }

    #[test]
    fn test_keep_used_variable_store() {
        let mut func = IrFunction::new("f", IrType::I32);
        func.push_op(IrOp::LoadConst { dest: 0, value: ConstValue::I32(42) });
        func.push_op(IrOp::StoreVar { name: "used_var".into(), src: 0 });
        func.push_op(IrOp::LoadVar { dest: 1, name: "used_var".into() });
        func.push_op(IrOp::Return { value: Some(1) });

        let stats = dead_code_eliminate_function(&mut func);
        assert_eq!(stats.unused_vars_removed, 0);
        assert!(func.ops.iter().any(|op| matches!(op, IrOp::StoreVar { name, .. } if name == "used_var")));
    }

    #[test]
    fn test_remove_unused_function() {
        let mut module = IrModule::new("main");
        let mut main_func = IrFunction::new("main", IrType::Void);
        main_func.push_op(IrOp::Return { value: None });
        module.add_function(main_func);

        let unused_func = IrFunction::new("unused", IrType::Void);
        module.add_function(unused_func);

        let stats = dead_code_eliminate_module(&mut module);
        assert_eq!(stats.unused_functions_removed, 1);
        assert!(module.get_function("main").is_some());
        assert!(module.get_function("unused").is_none());
    }

    #[test]
    fn test_keep_called_function() {
        let mut module = IrModule::new("main");
        let mut main_func = IrFunction::new("main", IrType::Void);
        main_func.push_op(IrOp::Call { dest: None, func: "helper".into(), args: vec![] });
        main_func.push_op(IrOp::Return { value: None });
        module.add_function(main_func);

        let helper = IrFunction::new("helper", IrType::Void);
        module.add_function(helper);

        let stats = dead_code_eliminate_module(&mut module);
        assert_eq!(stats.unused_functions_removed, 0);
        assert!(module.get_function("helper").is_some());
    }

    #[test]
    fn test_keep_transitive_call() {
        let mut module = IrModule::new("main");
        let mut main_func = IrFunction::new("main", IrType::Void);
        main_func.push_op(IrOp::Call { dest: None, func: "a".into(), args: vec![] });
        main_func.push_op(IrOp::Return { value: None });
        module.add_function(main_func);

        let mut a_func = IrFunction::new("a", IrType::Void);
        a_func.push_op(IrOp::Call { dest: None, func: "b".into(), args: vec![] });
        a_func.push_op(IrOp::Return { value: None });
        module.add_function(a_func);

        let b_func = IrFunction::new("b", IrType::Void);
        module.add_function(b_func);

        let orphan = IrFunction::new("orphan", IrType::Void);
        module.add_function(orphan);

        let stats = dead_code_eliminate_module(&mut module);
        assert_eq!(stats.unused_functions_removed, 1);
        assert!(module.get_function("b").is_some());
        assert!(module.get_function("orphan").is_none());
    }

    #[test]
    fn test_find_used_registers() {
        let mut func = IrFunction::new("f", IrType::I32);
        func.push_op(IrOp::LoadConst { dest: 0, value: ConstValue::I32(1) });
        func.push_op(IrOp::LoadConst { dest: 1, value: ConstValue::I32(2) });
        func.push_op(IrOp::BinaryOp { dest: 2, op: BinaryOp::Add, lhs: 0, rhs: 1 });
        func.push_op(IrOp::Return { value: Some(2) });

        let used = find_used_registers(&func);
        assert!(used.contains(&0));
        assert!(used.contains(&1));
        assert!(used.contains(&2));
    }

    #[test]
    fn test_unreachable_after_jump() {
        let mut func = IrFunction::new("f", IrType::Void);
        func.push_op(IrOp::Jump { target: "end".into() });
        func.push_op(IrOp::LoadConst { dest: 0, value: ConstValue::I32(99) }); // unreachable
        func.push_op(IrOp::Label { name: "end".into() });
        func.push_op(IrOp::Return { value: None });

        let stats = dead_code_eliminate_function(&mut func);
        assert_eq!(stats.unreachable_ops_removed, 1);
    }

    #[test]
    fn test_unreachable_after_branch() {
        let mut func = IrFunction::new("f", IrType::Void);
        func.push_op(IrOp::Branch {
            cond: 0,
            then_label: "a".into(),
            else_label: "b".into(),
        });
        func.push_op(IrOp::Nop); // unreachable
        func.push_op(IrOp::Label { name: "a".into() });
        func.push_op(IrOp::Return { value: None });

        let stats = dead_code_eliminate_function(&mut func);
        assert_eq!(stats.unreachable_ops_removed, 1);
    }

    #[test]
    fn test_empty_function_dce() {
        let mut func = IrFunction::new("f", IrType::Void);
        func.push_op(IrOp::Return { value: None });
        let stats = dead_code_eliminate_function(&mut func);
        assert_eq!(stats.unreachable_ops_removed, 0);
        assert_eq!(stats.unused_vars_removed, 0);
    }

    #[test]
    fn test_dce_module_no_functions() {
        let mut module = IrModule::new("main");
        // No functions — entry point missing, but DCE should still work
        let stats = dead_code_eliminate_module(&mut module);
        assert_eq!(stats.unused_functions_removed, 0);
    }

    #[test]
    fn test_remove_multiple_unused_vars() {
        let mut func = IrFunction::new("f", IrType::Void);
        func.push_op(IrOp::LoadConst { dest: 0, value: ConstValue::I32(1) });
        func.push_op(IrOp::StoreVar { name: "a".into(), src: 0 }); // unused
        func.push_op(IrOp::StoreVar { name: "b".into(), src: 0 }); // unused
        func.push_op(IrOp::LoadVar { dest: 1, name: "c".into() }); // used
        func.push_op(IrOp::Return { value: Some(1) });

        let stats = dead_code_eliminate_function(&mut func);
        assert_eq!(stats.unused_vars_removed, 2);
    }
}
