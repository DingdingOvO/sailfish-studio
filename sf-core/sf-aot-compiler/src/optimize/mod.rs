//! Optimization passes for the Sailfish AOT Compiler IR.

pub mod constant_folding;
pub mod dead_code;
pub mod inlining;

use crate::ir::IrModule;
use serde::{Deserialize, Serialize};

/// Run all optimization passes on the module.
pub fn optimize(module: &mut IrModule) -> OptimizeStats {
    let mut stats = OptimizeStats::default();

    // Run constant folding
    let fold_stats = constant_folding::constant_fold_module(module);
    stats.constants_folded += fold_stats.constants_folded;

    // Run dead code elimination
    let dce_stats = dead_code::dead_code_eliminate_module(module);
    stats.unused_vars_removed += dce_stats.unused_vars_removed;
    stats.unreachable_ops_removed += dce_stats.unreachable_ops_removed;
    stats.unused_functions_removed += dce_stats.unused_functions_removed;

    // Run inlining
    let inline_stats = inlining::inline_module(module);
    stats.functions_inlined += inline_stats.functions_inlined;

    stats
}

/// Statistics from optimization passes.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptimizeStats {
    pub constants_folded: usize,
    pub unused_vars_removed: usize,
    pub unreachable_ops_removed: usize,
    pub unused_functions_removed: usize,
    pub functions_inlined: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::*;

    #[test]
    fn test_optimize_empty_module() {
        let mut module = IrModule::new("main");
        let func = IrFunction::new("main", IrType::Void);
        module.add_function(func);
        let stats = optimize(&mut module);
        assert_eq!(stats.constants_folded, 0);
    }

    #[test]
    fn test_optimize_with_foldable_code() {
        let mut module = IrModule::new("main");
        let mut func = IrFunction::new("main", IrType::I32);
        func.push_op(IrOp::LoadConst { dest: 0, value: ConstValue::I32(3) });
        func.push_op(IrOp::LoadConst { dest: 1, value: ConstValue::I32(4) });
        func.push_op(IrOp::BinaryOp { dest: 2, op: BinaryOp::Add, lhs: 0, rhs: 1 });
        func.push_op(IrOp::Return { value: Some(2) });
        module.add_function(func);
        let stats = optimize(&mut module);
        assert!(stats.constants_folded > 0);
    }
}
