//! Constant folding optimization pass.
//!
//! Evaluates constant expressions at compile time and replaces
//! them with their computed values.

use crate::ir::{ConstValue, IrFunction, IrModule, IrOp};
use std::collections::HashMap;

/// Statistics from constant folding.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConstantFoldStats {
    pub constants_folded: usize,
}

/// Run constant folding on an entire module.
pub fn constant_fold_module(module: &mut IrModule) -> ConstantFoldStats {
    let mut stats = ConstantFoldStats::default();
    for func in &mut module.functions {
        let func_stats = constant_fold_function(func);
        stats.constants_folded += func_stats.constants_folded;
    }
    stats
}

/// Run constant folding on a single function.
pub fn constant_fold_function(func: &mut IrFunction) -> ConstantFoldStats {
    let mut stats = ConstantFoldStats::default();

    // Build a map of register -> known constant value
    let mut known_constants: HashMap<usize, ConstValue> = HashMap::new();

    // First pass: collect all known constants
    for op in &func.ops {
        if let IrOp::LoadConst { dest, value } = op {
            known_constants.insert(*dest, value.clone());
        }
    }

    // Repeatedly fold until no more changes
    let mut changed = true;
    while changed {
        changed = false;
        let mut new_ops = Vec::new();

        for op in &func.ops {
            match op {
                IrOp::BinaryOp { dest, op: binop, lhs, rhs } => {
                    if let (Some(lval), Some(rval)) =
                        (known_constants.get(lhs), known_constants.get(rhs))
                    {
                        if let Some(result) = binop.eval(lval, rval) {
                            known_constants.insert(*dest, result.clone());
                            new_ops.push(IrOp::LoadConst { dest: *dest, value: result });
                            stats.constants_folded += 1;
                            changed = true;
                            continue;
                        }
                    }
                    new_ops.push(op.clone());
                }
                IrOp::UnaryOp { dest, op: unop, operand } => {
                    if let Some(val) = known_constants.get(operand) {
                        if let Some(result) = unop.eval(val) {
                            known_constants.insert(*dest, result.clone());
                            new_ops.push(IrOp::LoadConst { dest: *dest, value: result });
                            stats.constants_folded += 1;
                            changed = true;
                            continue;
                        }
                    }
                    new_ops.push(op.clone());
                }
                _ => {
                    new_ops.push(op.clone());
                }
            }
        }

        func.ops = new_ops;
    }

    stats
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{BinaryOp, IrType, UnaryOp};

    #[test]
    fn test_fold_i32_add() {
        let mut func = IrFunction::new("f", IrType::I32);
        func.push_op(IrOp::LoadConst { dest: 0, value: ConstValue::I32(3) });
        func.push_op(IrOp::LoadConst { dest: 1, value: ConstValue::I32(4) });
        func.push_op(IrOp::BinaryOp { dest: 2, op: BinaryOp::Add, lhs: 0, rhs: 1 });
        func.push_op(IrOp::Return { value: Some(2) });

        let stats = constant_fold_function(&mut func);
        assert_eq!(stats.constants_folded, 1);

        // The BinaryOp should be replaced with LoadConst(7)
        let folded = &func.ops[2];
        assert!(matches!(folded, IrOp::LoadConst { dest: 2, value: ConstValue::I32(7) }));
    }

    #[test]
    fn test_fold_i32_sub() {
        let mut func = IrFunction::new("f", IrType::I32);
        func.push_op(IrOp::LoadConst { dest: 0, value: ConstValue::I32(10) });
        func.push_op(IrOp::LoadConst { dest: 1, value: ConstValue::I32(3) });
        func.push_op(IrOp::BinaryOp { dest: 2, op: BinaryOp::Sub, lhs: 0, rhs: 1 });
        func.push_op(IrOp::Return { value: Some(2) });

        let stats = constant_fold_function(&mut func);
        assert_eq!(stats.constants_folded, 1);
        assert!(matches!(&func.ops[2], IrOp::LoadConst { value: ConstValue::I32(7), .. }));
    }

    #[test]
    fn test_fold_i32_mul() {
        let mut func = IrFunction::new("f", IrType::I32);
        func.push_op(IrOp::LoadConst { dest: 0, value: ConstValue::I32(6) });
        func.push_op(IrOp::LoadConst { dest: 1, value: ConstValue::I32(7) });
        func.push_op(IrOp::BinaryOp { dest: 2, op: BinaryOp::Mul, lhs: 0, rhs: 1 });
        func.push_op(IrOp::Return { value: Some(2) });

        let stats = constant_fold_function(&mut func);
        assert_eq!(stats.constants_folded, 1);
        assert!(matches!(&func.ops[2], IrOp::LoadConst { value: ConstValue::I32(42), .. }));
    }

    #[test]
    fn test_fold_f64_add() {
        let mut func = IrFunction::new("f", IrType::F64);
        func.push_op(IrOp::LoadConst { dest: 0, value: ConstValue::F64(1.5) });
        func.push_op(IrOp::LoadConst { dest: 1, value: ConstValue::F64(2.5) });
        func.push_op(IrOp::BinaryOp { dest: 2, op: BinaryOp::Add, lhs: 0, rhs: 1 });
        func.push_op(IrOp::Return { value: Some(2) });

        let stats = constant_fold_function(&mut func);
        assert_eq!(stats.constants_folded, 1);
        assert!(matches!(&func.ops[2], IrOp::LoadConst { value: ConstValue::F64(4.0), .. }));
    }

    #[test]
    fn test_fold_comparison() {
        let mut func = IrFunction::new("f", IrType::Bool);
        func.push_op(IrOp::LoadConst { dest: 0, value: ConstValue::I32(5) });
        func.push_op(IrOp::LoadConst { dest: 1, value: ConstValue::I32(3) });
        func.push_op(IrOp::BinaryOp { dest: 2, op: BinaryOp::Gt, lhs: 0, rhs: 1 });
        func.push_op(IrOp::Return { value: Some(2) });

        let stats = constant_fold_function(&mut func);
        assert_eq!(stats.constants_folded, 1);
        assert!(matches!(&func.ops[2], IrOp::LoadConst { value: ConstValue::Bool(true), .. }));
    }

    #[test]
    fn test_fold_string_concat() {
        let mut func = IrFunction::new("f", IrType::String);
        func.push_op(IrOp::LoadConst { dest: 0, value: ConstValue::String("hello".into()) });
        func.push_op(IrOp::LoadConst { dest: 1, value: ConstValue::String(" world".into()) });
        func.push_op(IrOp::BinaryOp { dest: 2, op: BinaryOp::Add, lhs: 0, rhs: 1 });
        func.push_op(IrOp::Return { value: Some(2) });

        let stats = constant_fold_function(&mut func);
        assert_eq!(stats.constants_folded, 1);
        assert!(matches!(&func.ops[2], IrOp::LoadConst { value: ConstValue::String(s), .. } if s == "hello world"));
    }

    #[test]
    fn test_fold_neg() {
        let mut func = IrFunction::new("f", IrType::I32);
        func.push_op(IrOp::LoadConst { dest: 0, value: ConstValue::I32(42) });
        func.push_op(IrOp::UnaryOp { dest: 1, op: UnaryOp::Neg, operand: 0 });
        func.push_op(IrOp::Return { value: Some(1) });

        let stats = constant_fold_function(&mut func);
        assert_eq!(stats.constants_folded, 1);
        assert!(matches!(&func.ops[1], IrOp::LoadConst { value: ConstValue::I32(-42), .. }));
    }

    #[test]
    fn test_fold_not() {
        let mut func = IrFunction::new("f", IrType::Bool);
        func.push_op(IrOp::LoadConst { dest: 0, value: ConstValue::Bool(true) });
        func.push_op(IrOp::UnaryOp { dest: 1, op: UnaryOp::Not, operand: 0 });
        func.push_op(IrOp::Return { value: Some(1) });

        let stats = constant_fold_function(&mut func);
        assert_eq!(stats.constants_folded, 1);
        assert!(matches!(&func.ops[1], IrOp::LoadConst { value: ConstValue::Bool(false), .. }));
    }

    #[test]
    fn test_no_fold_variable() {
        let mut func = IrFunction::new("f", IrType::I32);
        func.push_op(IrOp::LoadVar { dest: 0, name: "x".into() });
        func.push_op(IrOp::LoadConst { dest: 1, value: ConstValue::I32(5) });
        func.push_op(IrOp::BinaryOp { dest: 2, op: BinaryOp::Add, lhs: 0, rhs: 1 });
        func.push_op(IrOp::Return { value: Some(2) });

        let stats = constant_fold_function(&mut func);
        assert_eq!(stats.constants_folded, 0);
    }

    #[test]
    fn test_no_fold_div_by_zero() {
        let mut func = IrFunction::new("f", IrType::I32);
        func.push_op(IrOp::LoadConst { dest: 0, value: ConstValue::I32(10) });
        func.push_op(IrOp::LoadConst { dest: 1, value: ConstValue::I32(0) });
        func.push_op(IrOp::BinaryOp { dest: 2, op: BinaryOp::Div, lhs: 0, rhs: 1 });
        func.push_op(IrOp::Return { value: Some(2) });

        let stats = constant_fold_function(&mut func);
        assert_eq!(stats.constants_folded, 0);
    }

    #[test]
    fn test_nested_folding() {
        // (3 + 4) * 2 = 14
        let mut func = IrFunction::new("f", IrType::I32);
        func.push_op(IrOp::LoadConst { dest: 0, value: ConstValue::I32(3) });
        func.push_op(IrOp::LoadConst { dest: 1, value: ConstValue::I32(4) });
        func.push_op(IrOp::BinaryOp { dest: 2, op: BinaryOp::Add, lhs: 0, rhs: 1 });
        func.push_op(IrOp::LoadConst { dest: 3, value: ConstValue::I32(2) });
        func.push_op(IrOp::BinaryOp { dest: 4, op: BinaryOp::Mul, lhs: 2, rhs: 3 });
        func.push_op(IrOp::Return { value: Some(4) });

        let stats = constant_fold_function(&mut func);
        // Both the add and the multiply should fold
        assert_eq!(stats.constants_folded, 2);
        assert!(matches!(&func.ops[4], IrOp::LoadConst { value: ConstValue::I32(14), .. }));
    }

    #[test]
    fn test_fold_logical_and() {
        let mut func = IrFunction::new("f", IrType::Bool);
        func.push_op(IrOp::LoadConst { dest: 0, value: ConstValue::Bool(true) });
        func.push_op(IrOp::LoadConst { dest: 1, value: ConstValue::Bool(false) });
        func.push_op(IrOp::BinaryOp { dest: 2, op: BinaryOp::And, lhs: 0, rhs: 1 });
        func.push_op(IrOp::Return { value: Some(2) });

        let stats = constant_fold_function(&mut func);
        assert_eq!(stats.constants_folded, 1);
        assert!(matches!(&func.ops[2], IrOp::LoadConst { value: ConstValue::Bool(false), .. }));
    }

    #[test]
    fn test_fold_module() {
        let mut module = IrModule::new("main");
        let mut func = IrFunction::new("main", IrType::I32);
        func.push_op(IrOp::LoadConst { dest: 0, value: ConstValue::I32(100) });
        func.push_op(IrOp::LoadConst { dest: 1, value: ConstValue::I32(200) });
        func.push_op(IrOp::BinaryOp { dest: 2, op: BinaryOp::Add, lhs: 0, rhs: 1 });
        func.push_op(IrOp::Return { value: Some(2) });
        module.add_function(func);

        let stats = constant_fold_module(&mut module);
        assert_eq!(stats.constants_folded, 1);
    }

    #[test]
    fn test_fold_eq_strings() {
        let mut func = IrFunction::new("f", IrType::Bool);
        func.push_op(IrOp::LoadConst { dest: 0, value: ConstValue::String("abc".into()) });
        func.push_op(IrOp::LoadConst { dest: 1, value: ConstValue::String("abc".into()) });
        func.push_op(IrOp::BinaryOp { dest: 2, op: BinaryOp::Eq, lhs: 0, rhs: 1 });
        func.push_op(IrOp::Return { value: Some(2) });

        let stats = constant_fold_function(&mut func);
        assert_eq!(stats.constants_folded, 1);
        assert!(matches!(&func.ops[2], IrOp::LoadConst { value: ConstValue::Bool(true), .. }));
    }

    #[test]
    fn test_fold_le_comparison() {
        let mut func = IrFunction::new("f", IrType::Bool);
        func.push_op(IrOp::LoadConst { dest: 0, value: ConstValue::I32(3) });
        func.push_op(IrOp::LoadConst { dest: 1, value: ConstValue::I32(3) });
        func.push_op(IrOp::BinaryOp { dest: 2, op: BinaryOp::Le, lhs: 0, rhs: 1 });
        func.push_op(IrOp::Return { value: Some(2) });

        let stats = constant_fold_function(&mut func);
        assert_eq!(stats.constants_folded, 1);
        assert!(matches!(&func.ops[2], IrOp::LoadConst { value: ConstValue::Bool(true), .. }));
    }

    #[test]
    fn test_fold_mod() {
        let mut func = IrFunction::new("f", IrType::I32);
        func.push_op(IrOp::LoadConst { dest: 0, value: ConstValue::I32(10) });
        func.push_op(IrOp::LoadConst { dest: 1, value: ConstValue::I32(3) });
        func.push_op(IrOp::BinaryOp { dest: 2, op: BinaryOp::Mod, lhs: 0, rhs: 1 });
        func.push_op(IrOp::Return { value: Some(2) });

        let stats = constant_fold_function(&mut func);
        assert_eq!(stats.constants_folded, 1);
        assert!(matches!(&func.ops[2], IrOp::LoadConst { value: ConstValue::I32(1), .. }));
    }

    #[test]
    fn test_fold_or() {
        let mut func = IrFunction::new("f", IrType::Bool);
        func.push_op(IrOp::LoadConst { dest: 0, value: ConstValue::Bool(false) });
        func.push_op(IrOp::LoadConst { dest: 1, value: ConstValue::Bool(true) });
        func.push_op(IrOp::BinaryOp { dest: 2, op: BinaryOp::Or, lhs: 0, rhs: 1 });
        func.push_op(IrOp::Return { value: Some(2) });

        let stats = constant_fold_function(&mut func);
        assert_eq!(stats.constants_folded, 1);
        assert!(matches!(&func.ops[2], IrOp::LoadConst { value: ConstValue::Bool(true), .. }));
    }
}
