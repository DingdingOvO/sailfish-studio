//! Sailfish Studio AOT Compiler
//!
//! Compiles Sailfish projects (.sf/.sfl) to native executables.
//!
//! # Compilation Pipeline
//!
//! 1. **Parse**: Load .sf or .sfl project file
//! 2. **Lower to IR**: Convert project operations to compiler IR
//! 3. **Optimize**: Constant folding, dead code elimination, inlining
//! 4. **Code Generation**: Generate native machine code (LLVM or Cranelift - stub)
//! 5. **Link**: Add startup code, link runtime
//! 6. **Output**: Native executable

pub mod codegen;
pub mod ir;
pub mod linker;
pub mod lower;
pub mod optimize;

use codegen::{CodegenError, Target};
use ir::IrValidationError;
use linker::{LinkError, Linker};
use lower::{lower_project, LowerError, ProjectData};
use optimize::OptimizeStats;
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

/// Errors that can occur during AOT compilation.
#[derive(Debug, Error)]
pub enum CompileError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("lowering error: {0}")]
    LowerError(#[from] LowerError),
    #[error("IR validation error: {0}")]
    ValidationError(#[from] IrValidationError),
    #[error("code generation error: {0}")]
    CodegenError(#[from] CodegenError),
    #[error("link error: {0}")]
    LinkError(#[from] LinkError),
    #[error("compilation error: {0}")]
    General(String),
}

/// Result type for AOT compilation.
pub type CompileResult<T> = Result<T, CompileError>;

/// Optimization level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OptLevel {
    /// No optimization, fast compile, good for debugging.
    Debug,
    /// Full optimization, slower compile, best performance.
    Release,
    /// Optimize for size.
    Size,
}

impl OptLevel {
    /// Returns true if this is the Debug optimization level.
    pub fn is_debug(&self) -> bool {
        matches!(self, OptLevel::Debug)
    }

    /// Returns true if this is the Release optimization level.
    pub fn is_release(&self) -> bool {
        matches!(self, OptLevel::Release)
    }

    /// Returns the optimization level as a string.
    pub fn as_str(&self) -> &str {
        match self {
            OptLevel::Debug => "debug",
            OptLevel::Release => "release",
            OptLevel::Size => "size",
        }
    }

    /// All optimization levels.
    pub fn all() -> Vec<OptLevel> {
        vec![OptLevel::Debug, OptLevel::Release, OptLevel::Size]
    }
}

impl std::fmt::Display for OptLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Feature flags for AOT compilation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeatureFlags {
    /// Run in headless mode (no graphics).
    pub headless: bool,
    /// Disable network access.
    pub no_network: bool,
    /// Enable debug symbols.
    pub debug_symbols: bool,
    /// Static linking.
    pub static_link: bool,
    /// Custom feature flags.
    pub custom: Vec<String>,
}

impl FeatureFlags {
    /// Create new default feature flags.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable headless mode.
    pub fn with_headless(mut self) -> Self {
        self.headless = true;
        self
    }

    /// Disable network.
    pub fn with_no_network(mut self) -> Self {
        self.no_network = true;
        self
    }

    /// Enable debug symbols.
    pub fn with_debug_symbols(mut self) -> Self {
        self.debug_symbols = true;
        self
    }

    /// Enable static linking.
    pub fn with_static_link(mut self) -> Self {
        self.static_link = true;
        self
    }
}

/// Configuration for AOT compilation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileConfig {
    /// Target platform.
    pub target: Target,
    /// Optimization level.
    pub opt_level: OptLevel,
    /// Feature flags.
    pub features: FeatureFlags,
}

impl CompileConfig {
    /// Create a new compilation config.
    pub fn new(target: Target, opt_level: OptLevel) -> Self {
        Self {
            target,
            opt_level,
            features: FeatureFlags::default(),
        }
    }

    /// Set feature flags.
    pub fn with_features(mut self, features: FeatureFlags) -> Self {
        self.features = features;
        self
    }
}

impl Default for CompileConfig {
    fn default() -> Self {
        Self::new(Target::LinuxX64, OptLevel::Debug)
    }
}

/// The result of a successful compilation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationOutput {
    /// Path to the output executable.
    pub output_path: String,
    /// Size of the output in bytes.
    pub size: u64,
    /// Time taken for compilation in milliseconds.
    pub compile_time_ms: u64,
    /// Warnings generated during compilation.
    pub warnings: Vec<CompileWarning>,
    /// Optimization statistics.
    pub optimize_stats: OptimizeStats,
}

/// A compilation warning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileWarning {
    pub message: String,
    pub kind: WarningKind,
}

/// The kind of compilation warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarningKind {
    /// An unknown opcode was encountered.
    UnknownOpcode,
    /// Code was unreachable and removed.
    UnreachableCode,
    /// A variable was unused.
    UnusedVariable,
    /// A function was unused and removed.
    UnusedFunction,
    /// General warning.
    General,
}

/// The AOT compiler.
pub struct AotCompiler {
    config: CompileConfig,
}

impl AotCompiler {
    /// Create a new AOT compiler with the given configuration.
    pub fn new(config: CompileConfig) -> Self {
        Self { config }
    }

    /// Get the compiler configuration.
    pub fn config(&self) -> &CompileConfig {
        &self.config
    }

    /// Compile a project from a path.
    pub fn compile<P: AsRef<Path>>(
        &self,
        project_path: P,
        output_path: P,
    ) -> CompileResult<CompilationOutput> {
        let start = std::time::Instant::now();

        // Step 1: Parse (simulated)
        let project = ProjectData {
            name: project_path.as_ref().to_string_lossy().to_string(),
            targets: vec![],
        };

        // Step 2: Lower to IR
        let mut module = lower_project(&project)?;

        // Step 3: Optimize
        let optimize_stats = if !self.config.opt_level.is_debug() {
            optimize::optimize(&mut module)
        } else {
            OptimizeStats::default()
        };

        // Step 4: Validate IR
        module.validate()?;

        // Step 5: Code generation (stub)
        let mut codegen = codegen::create_codegen(self.config.target)?;
        let native_code = codegen.generate(&module).ok();

        // Step 6: Link (stub)
        let mut linker = Linker::new(self.config.target);
        let _linked = linker.link(&module, &module.entry_point).ok();

        // Calculate output
        let size = native_code.as_ref().map(|c| c.size() as u64).unwrap_or(0);
        let compile_time_ms = start.elapsed().as_millis() as u64;

        // Generate warnings
        let mut warnings = Vec::new();
        for func in &module.functions {
            for op in &func.ops {
                if matches!(op, ir::IrOp::Nop) {
                    warnings.push(CompileWarning {
                        message: "unknown opcode lowered to nop".to_string(),
                        kind: WarningKind::UnknownOpcode,
                    });
                }
            }
        }

        Ok(CompilationOutput {
            output_path: output_path.as_ref().to_string_lossy().to_string(),
            size,
            compile_time_ms,
            warnings,
            optimize_stats,
        })
    }

    /// Compile from project data directly (bypasses file I/O).
    pub fn compile_project_data(
        &self,
        project: &ProjectData,
    ) -> CompileResult<(ir::IrModule, OptimizeStats)> {
        let mut module = lower_project(project)?;

        let stats = if !self.config.opt_level.is_debug() {
            optimize::optimize(&mut module)
        } else {
            OptimizeStats::default()
        };

        module.validate()?;

        Ok((module, stats))
    }

    /// Lower a project to IR without optimization.
    pub fn lower_only(&self, project: &ProjectData) -> CompileResult<ir::IrModule> {
        let module = lower_project(project)?;
        Ok(module)
    }

    /// Run optimization only on an IR module.
    pub fn optimize_only(&self, module: &mut ir::IrModule) -> OptimizeStats {
        if !self.config.opt_level.is_debug() {
            optimize::optimize(module)
        } else {
            OptimizeStats::default()
        }
    }
}

// Re-export key types
pub use codegen::{NativeCode, Target as CodegenTarget};
pub use ir::{
    BasicBlock, BinaryOp, ConstValue, IrFunction, IrGlobal, IrOp, IrParam, IrType, UnaryOp,
};
pub use lower::{BlockData, InputValue, ProjectData as LowerProjectData, TargetData};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::*;

    // ===== CompileConfig tests =====

    #[test]
    fn test_compile_config_default() {
        let config = CompileConfig::default();
        assert_eq!(config.target, Target::LinuxX64);
        assert_eq!(config.opt_level, OptLevel::Debug);
        assert!(!config.features.headless);
    }

    #[test]
    fn test_compile_config_new() {
        let config = CompileConfig::new(Target::MacosArm64, OptLevel::Release);
        assert_eq!(config.target, Target::MacosArm64);
        assert_eq!(config.opt_level, OptLevel::Release);
    }

    #[test]
    fn test_compile_config_with_features() {
        let features = FeatureFlags::new().with_headless().with_no_network();
        let config = CompileConfig::new(Target::LinuxX64, OptLevel::Size).with_features(features);
        assert!(config.features.headless);
        assert!(config.features.no_network);
        assert_eq!(config.opt_level, OptLevel::Size);
    }

    // ===== OptLevel tests =====

    #[test]
    fn test_opt_level_is_debug() {
        assert!(OptLevel::Debug.is_debug());
        assert!(!OptLevel::Release.is_debug());
        assert!(!OptLevel::Size.is_debug());
    }

    #[test]
    fn test_opt_level_is_release() {
        assert!(OptLevel::Release.is_release());
        assert!(!OptLevel::Debug.is_release());
    }

    #[test]
    fn test_opt_level_as_str() {
        assert_eq!(OptLevel::Debug.as_str(), "debug");
        assert_eq!(OptLevel::Release.as_str(), "release");
        assert_eq!(OptLevel::Size.as_str(), "size");
    }

    #[test]
    fn test_opt_level_display() {
        assert_eq!(OptLevel::Debug.to_string(), "debug");
        assert_eq!(OptLevel::Release.to_string(), "release");
    }

    #[test]
    fn test_opt_level_all() {
        let all = OptLevel::all();
        assert_eq!(all.len(), 3);
    }

    // ===== FeatureFlags tests =====

    #[test]
    fn test_feature_flags_default() {
        let flags = FeatureFlags::default();
        assert!(!flags.headless);
        assert!(!flags.no_network);
        assert!(!flags.debug_symbols);
        assert!(!flags.static_link);
        assert!(flags.custom.is_empty());
    }

    #[test]
    fn test_feature_flags_builder() {
        let flags = FeatureFlags::new()
            .with_headless()
            .with_no_network()
            .with_debug_symbols()
            .with_static_link();
        assert!(flags.headless);
        assert!(flags.no_network);
        assert!(flags.debug_symbols);
        assert!(flags.static_link);
    }

    // ===== CompileWarning tests =====

    #[test]
    fn test_compile_warning() {
        let w = CompileWarning {
            message: "test".into(),
            kind: WarningKind::UnknownOpcode,
        };
        assert_eq!(w.message, "test");
        assert_eq!(w.kind, WarningKind::UnknownOpcode);
    }

    #[test]
    fn test_warning_kinds() {
        assert_ne!(WarningKind::UnknownOpcode, WarningKind::UnreachableCode);
        assert_ne!(WarningKind::UnusedVariable, WarningKind::UnusedFunction);
    }

    // ===== AotCompiler tests =====

    #[test]
    fn test_aot_compiler_new() {
        let config = CompileConfig::new(Target::LinuxX64, OptLevel::Release);
        let compiler = AotCompiler::new(config);
        assert_eq!(compiler.config().target, Target::LinuxX64);
        assert_eq!(compiler.config().opt_level, OptLevel::Release);
    }

    #[test]
    fn test_aot_compiler_compile_empty() {
        let config = CompileConfig::new(Target::LinuxX64, OptLevel::Debug);
        let compiler = AotCompiler::new(config);
        let result = compiler.compile("test.sf", "output");
        assert!(result.is_ok());
        let compile_result = result.unwrap();
        assert_eq!(compile_result.output_path, "output");
    }

    #[test]
    fn test_aot_compiler_compile_project_data() {
        let config = CompileConfig::new(Target::LinuxX64, OptLevel::Release);
        let compiler = AotCompiler::new(config);
        let project = lower::make_simple_project("test");
        let result = compiler.compile_project_data(&project);
        assert!(result.is_ok());
        let (module, _stats) = result.unwrap();
        assert!(module.get_function("main").is_some());
    }

    #[test]
    fn test_aot_compiler_lower_only() {
        let config = CompileConfig::new(Target::LinuxX64, OptLevel::Debug);
        let compiler = AotCompiler::new(config);
        let project = lower::make_simple_project("test");
        let result = compiler.lower_only(&project);
        assert!(result.is_ok());
        let module = result.unwrap();
        assert!(module.get_function("main").is_some());
    }

    #[test]
    fn test_aot_compiler_optimize_only_debug() {
        let config = CompileConfig::new(Target::LinuxX64, OptLevel::Debug);
        let compiler = AotCompiler::new(config);
        let mut module = ir::IrModule::new("main");
        let func = IrFunction::new("main", IrType::Void);
        module.add_function(func);
        let stats = compiler.optimize_only(&mut module);
        assert_eq!(stats.constants_folded, 0);
    }

    #[test]
    fn test_aot_compiler_optimize_only_release() {
        let config = CompileConfig::new(Target::LinuxX64, OptLevel::Release);
        let compiler = AotCompiler::new(config);
        let mut module = ir::IrModule::new("main");
        let mut func = IrFunction::new("main", IrType::I32);
        func.push_op(IrOp::LoadConst {
            dest: 0,
            value: ConstValue::I32(3),
        });
        func.push_op(IrOp::LoadConst {
            dest: 1,
            value: ConstValue::I32(4),
        });
        func.push_op(IrOp::BinaryOp {
            dest: 2,
            op: BinaryOp::Add,
            lhs: 0,
            rhs: 1,
        });
        func.push_op(IrOp::Return { value: Some(2) });
        module.add_function(func);
        let stats = compiler.optimize_only(&mut module);
        assert!(stats.constants_folded > 0);
    }

    // ===== Full pipeline tests =====

    #[test]
    fn test_full_pipeline_motion() {
        let config = CompileConfig::new(Target::LinuxX64, OptLevel::Release);
        let compiler = AotCompiler::new(config);
        let project = lower::make_motion_project();
        let result = compiler.compile_project_data(&project);
        assert!(result.is_ok());
    }

    #[test]
    fn test_full_pipeline_arithmetic() {
        let config = CompileConfig::new(Target::LinuxX64, OptLevel::Release);
        let compiler = AotCompiler::new(config);
        let project = lower::make_arithmetic_project();
        let result = compiler.compile_project_data(&project);
        assert!(result.is_ok());
        let (_, stats) = result.unwrap();
        assert!(stats.constants_folded > 0);
    }

    #[test]
    fn test_full_pipeline_if_else() {
        let config = CompileConfig::new(Target::MacosArm64, OptLevel::Release);
        let compiler = AotCompiler::new(config);
        let project = lower::make_if_else_project();
        let result = compiler.compile_project_data(&project);
        assert!(result.is_ok());
    }

    #[test]
    fn test_full_pipeline_variables() {
        let config = CompileConfig::new(Target::WindowsX64, OptLevel::Size);
        let compiler = AotCompiler::new(config);
        let project = lower::make_variable_project();
        let result = compiler.compile_project_data(&project);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pipeline_debug_no_optimize() {
        let config = CompileConfig::new(Target::LinuxX64, OptLevel::Debug);
        let compiler = AotCompiler::new(config);
        let project = lower::make_arithmetic_project();
        let result = compiler.compile_project_data(&project);
        assert!(result.is_ok());
        let (_, stats) = result.unwrap();
        assert_eq!(stats.constants_folded, 0);
    }

    #[test]
    fn test_pipeline_release_with_folding() {
        let config = CompileConfig::new(Target::LinuxX64, OptLevel::Release);
        let compiler = AotCompiler::new(config);
        let project = lower::make_arithmetic_project();
        let result = compiler.compile_project_data(&project);
        assert!(result.is_ok());
        let (_, stats) = result.unwrap();
        assert!(stats.constants_folded > 0);
    }

    // ===== CompilationOutput tests =====

    #[test]
    fn test_compile_result_output_path() {
        let config = CompileConfig::new(Target::LinuxX64, OptLevel::Debug);
        let compiler = AotCompiler::new(config);
        let result = compiler.compile("test.sf", "my_output").unwrap();
        assert_eq!(result.output_path, "my_output");
    }

    #[test]
    fn test_compile_result_has_compile_time() {
        let config = CompileConfig::new(Target::LinuxX64, OptLevel::Debug);
        let compiler = AotCompiler::new(config);
        let result = compiler.compile("test.sf", "output").unwrap();
        assert!(result.compile_time_ms < 10000);
    }

    // ===== Different targets =====

    #[test]
    fn test_compile_target_windows() {
        let config = CompileConfig::new(Target::WindowsX64, OptLevel::Release);
        let compiler = AotCompiler::new(config);
        let project = lower::make_simple_project("test");
        let result = compiler.compile_project_data(&project);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_target_macos_arm() {
        let config = CompileConfig::new(Target::MacosArm64, OptLevel::Release);
        let compiler = AotCompiler::new(config);
        let project = lower::make_simple_project("test");
        let result = compiler.compile_project_data(&project);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_target_macos_x64() {
        let config = CompileConfig::new(Target::MacosX64, OptLevel::Release);
        let compiler = AotCompiler::new(config);
        let project = lower::make_simple_project("test");
        let result = compiler.compile_project_data(&project);
        assert!(result.is_ok());
    }

    // ===== Round-trip tests =====

    #[test]
    fn test_roundtrip_lower_optimize_validate() {
        let project = lower::make_motion_project();
        let mut module = lower_project(&project).unwrap();
        let _stats = optimize::optimize(&mut module);
        assert!(module.validate().is_ok());
    }

    #[test]
    fn test_roundtrip_arithmetic_optimize() {
        let project = lower::make_arithmetic_project();
        let mut module = lower_project(&project).unwrap();
        let stats = optimize::optimize(&mut module);
        assert!(module.validate().is_ok());
        assert!(stats.constants_folded > 0);
    }

    #[test]
    fn test_roundtrip_variable_optimize() {
        let project = lower::make_variable_project();
        let mut module = lower_project(&project).unwrap();
        let _stats = optimize::optimize(&mut module);
        assert!(module.validate().is_ok());
    }

    // ===== Error handling tests =====

    #[test]
    fn test_error_missing_entry_point() {
        let module = ir::IrModule::new("nonexistent");
        let result = module.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_error_undefined_label() {
        let mut module = ir::IrModule::new("main");
        let mut func = IrFunction::new("main", IrType::Void);
        func.push_op(IrOp::Jump {
            target: "nonexistent".into(),
        });
        module.add_function(func);
        assert!(module.validate().is_err());
    }

    #[test]
    fn test_error_undefined_function_call() {
        let mut module = ir::IrModule::new("main");
        let mut func = IrFunction::new("main", IrType::Void);
        func.push_op(IrOp::Call {
            dest: None,
            func: "missing".into(),
            args: vec![],
        });
        func.push_op(IrOp::Return { value: None });
        module.add_function(func);
        assert!(module.validate().is_err());
    }

    // ===== Integration tests =====

    #[test]
    fn test_integration_complex_project() {
        let mut vars = std::collections::HashMap::new();
        vars.insert("counter".to_string(), ConstValue::I32(0));
        vars.insert("max".to_string(), ConstValue::I32(10));

        let project = ProjectData {
            name: "complex".to_string(),
            targets: vec![
                lower::TargetData {
                    name: "Stage".to_string(),
                    is_stage: true,
                    variables: vars,
                    blocks: vec![
                        lower::BlockData {
                            opcode: "event_whenflagclicked".to_string(),
                            inputs: std::collections::HashMap::new(),
                            fields: std::collections::HashMap::new(),
                            next: None,
                            substack: None,
                            substack2: None,
                        },
                        lower::BlockData {
                            opcode: "operator_add".to_string(),
                            inputs: {
                                let mut m = std::collections::HashMap::new();
                                m.insert(
                                    "NUM1".to_string(),
                                    InputValue::Literal(ConstValue::I32(3)),
                                );
                                m.insert(
                                    "NUM2".to_string(),
                                    InputValue::Literal(ConstValue::I32(7)),
                                );
                                m
                            },
                            fields: std::collections::HashMap::new(),
                            next: None,
                            substack: None,
                            substack2: None,
                        },
                        lower::BlockData {
                            opcode: "looks_say".to_string(),
                            inputs: {
                                let mut m = std::collections::HashMap::new();
                                m.insert(
                                    "MESSAGE".to_string(),
                                    InputValue::Literal(ConstValue::String("Hello!".into())),
                                );
                                m
                            },
                            fields: std::collections::HashMap::new(),
                            next: None,
                            substack: None,
                            substack2: None,
                        },
                    ],
                },
                lower::TargetData {
                    name: "Sprite1".to_string(),
                    is_stage: false,
                    variables: std::collections::HashMap::new(),
                    blocks: vec![
                        lower::BlockData {
                            opcode: "motion_forward".to_string(),
                            inputs: {
                                let mut m = std::collections::HashMap::new();
                                m.insert(
                                    "STEPS".to_string(),
                                    InputValue::Literal(ConstValue::F64(10.0)),
                                );
                                m
                            },
                            fields: std::collections::HashMap::new(),
                            next: None,
                            substack: None,
                            substack2: None,
                        },
                        lower::BlockData {
                            opcode: "pen_penDown".to_string(),
                            inputs: std::collections::HashMap::new(),
                            fields: std::collections::HashMap::new(),
                            next: None,
                            substack: None,
                            substack2: None,
                        },
                    ],
                },
            ],
        };

        let config = CompileConfig::new(Target::LinuxX64, OptLevel::Release);
        let compiler = AotCompiler::new(config);
        let result = compiler.compile_project_data(&project);
        assert!(result.is_ok());
        let (module, stats) = result.unwrap();
        assert!(module.get_function("stage_main").is_some());
        assert!(module.get_function("Sprite1_main").is_some());
        assert!(module.get_function("main").is_some());
        assert!(stats.constants_folded > 0);
    }

    #[test]
    fn test_integration_all_op_levels() {
        let project = lower::make_simple_project("test");
        for opt_level in OptLevel::all() {
            let config = CompileConfig::new(Target::LinuxX64, opt_level);
            let compiler = AotCompiler::new(config);
            let result = compiler.compile_project_data(&project);
            assert!(result.is_ok(), "Failed for opt level {:?}", opt_level);
        }
    }

    #[test]
    fn test_integration_all_targets() {
        let project = lower::make_simple_project("test");
        for target in Target::all() {
            let config = CompileConfig::new(target, OptLevel::Release);
            let compiler = AotCompiler::new(config);
            let result = compiler.compile_project_data(&project);
            assert!(result.is_ok(), "Failed for target {:?}", target);
        }
    }

    // ===== Serialization tests =====

    #[test]
    fn test_ir_module_serialization() {
        let mut module = ir::IrModule::new("main");
        let mut func = IrFunction::new("main", IrType::Void);
        func.push_op(IrOp::Return { value: None });
        module.add_function(func);

        let json = serde_json::to_string(&module).unwrap();
        let deserialized: ir::IrModule = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.entry_point, "main");
        assert_eq!(deserialized.functions.len(), 1);
    }

    #[test]
    fn test_const_value_serialization() {
        let values = vec![
            ConstValue::I32(42),
            ConstValue::I64(1000000),
            ConstValue::F64(3.14),
            ConstValue::Bool(true),
            ConstValue::String("hello".into()),
            ConstValue::Null,
            ConstValue::Unit,
        ];
        for val in values {
            let json = serde_json::to_string(&val).unwrap();
            let deserialized: ConstValue = serde_json::from_str(&json).unwrap();
            assert_eq!(val, deserialized);
        }
    }

    #[test]
    fn test_ir_type_serialization() {
        let types = vec![
            IrType::Void,
            IrType::I32,
            IrType::F64,
            IrType::Bool,
            IrType::String,
        ];
        for ty in types {
            let json = serde_json::to_string(&ty).unwrap();
            let deserialized: IrType = serde_json::from_str(&json).unwrap();
            assert_eq!(ty, deserialized);
        }
    }

    #[test]
    fn test_binary_op_serialization() {
        for op in [BinaryOp::Add, BinaryOp::Sub, BinaryOp::Eq, BinaryOp::And] {
            let json = serde_json::to_string(&op).unwrap();
            let deserialized: BinaryOp = serde_json::from_str(&json).unwrap();
            assert_eq!(op, deserialized);
        }
    }

    #[test]
    fn test_feature_flags_serialization() {
        let flags = FeatureFlags::new().with_headless().with_debug_symbols();
        let json = serde_json::to_string(&flags).unwrap();
        let deserialized: FeatureFlags = serde_json::from_str(&json).unwrap();
        assert!(deserialized.headless);
        assert!(deserialized.debug_symbols);
        assert!(!deserialized.no_network);
    }

    #[test]
    fn test_compile_config_serialization() {
        let config = CompileConfig::new(Target::MacosArm64, OptLevel::Release);
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: CompileConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.target, deserialized.target);
        assert_eq!(config.opt_level, deserialized.opt_level);
    }

    #[test]
    fn test_opt_level_serialization() {
        for level in OptLevel::all() {
            let json = serde_json::to_string(&level).unwrap();
            let deserialized: OptLevel = serde_json::from_str(&json).unwrap();
            assert_eq!(level, deserialized);
        }
    }

    #[test]
    fn test_target_serialization() {
        for target in Target::all() {
            let json = serde_json::to_string(&target).unwrap();
            let deserialized: Target = serde_json::from_str(&json).unwrap();
            assert_eq!(target, deserialized);
        }
    }

    #[test]
    fn test_compilation_output_serialization() {
        let result = CompilationOutput {
            output_path: "out.exe".into(),
            size: 4096,
            compile_time_ms: 100,
            warnings: vec![],
            optimize_stats: OptimizeStats::default(),
        };
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: CompilationOutput = serde_json::from_str(&json).unwrap();
        assert_eq!(result.output_path, deserialized.output_path);
        assert_eq!(result.size, deserialized.size);
    }

    #[test]
    fn test_ir_function_serialization() {
        let mut func = IrFunction::new("test_func", IrType::I32);
        func.add_param("x", IrType::I32);
        func.push_op(IrOp::LoadVar {
            dest: 0,
            name: "x".into(),
        });
        func.push_op(IrOp::Return { value: Some(0) });

        let json = serde_json::to_string(&func).unwrap();
        let deserialized: IrFunction = serde_json::from_str(&json).unwrap();
        assert_eq!(func.name, deserialized.name);
        assert_eq!(func.params.len(), deserialized.params.len());
        assert_eq!(func.ops.len(), deserialized.ops.len());
    }

    #[test]
    fn test_ir_op_serialization() {
        let ops = vec![
            IrOp::LoadConst {
                dest: 0,
                value: ConstValue::I32(42),
            },
            IrOp::LoadVar {
                dest: 1,
                name: "x".into(),
            },
            IrOp::StoreVar {
                name: "y".into(),
                src: 1,
            },
            IrOp::BinaryOp {
                dest: 2,
                op: BinaryOp::Add,
                lhs: 0,
                rhs: 1,
            },
            IrOp::UnaryOp {
                dest: 3,
                op: UnaryOp::Not,
                operand: 2,
            },
            IrOp::Call {
                dest: Some(4),
                func: "foo".into(),
                args: vec![0, 1],
            },
            IrOp::Jump {
                target: "end".into(),
            },
            IrOp::Branch {
                cond: 0,
                then_label: "yes".into(),
                else_label: "no".into(),
            },
            IrOp::Return { value: Some(4) },
            IrOp::Nop,
            IrOp::Label {
                name: "start".into(),
            },
        ];
        for op in ops {
            let json = serde_json::to_string(&op).unwrap();
            let deserialized: IrOp = serde_json::from_str(&json).unwrap();
            assert_eq!(op, deserialized);
        }
    }

    #[test]
    fn test_ir_global() {
        let mut module = ir::IrModule::new("main");
        module.add_global("x", IrType::I32, Some(ConstValue::I32(42)));
        module.add_global("y", IrType::F64, None);
        assert_eq!(module.globals.len(), 2);
        assert_eq!(module.globals[0].name, "x");
        assert_eq!(module.globals[0].initial_value, Some(ConstValue::I32(42)));
        assert_eq!(module.globals[1].initial_value, None);
    }

    #[test]
    fn test_ir_global_serialization() {
        let global = IrGlobal {
            name: "counter".into(),
            ty: IrType::I32,
            initial_value: Some(ConstValue::I32(0)),
        };
        let json = serde_json::to_string(&global).unwrap();
        let deserialized: IrGlobal = serde_json::from_str(&json).unwrap();
        assert_eq!(global.name, deserialized.name);
    }

    // ===== Optimization integration tests =====

    #[test]
    fn test_constant_fold_div_by_zero_preserved() {
        let mut module = ir::IrModule::new("main");
        let mut func = IrFunction::new("main", IrType::I32);
        func.push_op(IrOp::LoadConst {
            dest: 0,
            value: ConstValue::I32(10),
        });
        func.push_op(IrOp::LoadConst {
            dest: 1,
            value: ConstValue::I32(0),
        });
        func.push_op(IrOp::BinaryOp {
            dest: 2,
            op: BinaryOp::Div,
            lhs: 0,
            rhs: 1,
        });
        func.push_op(IrOp::Return { value: Some(2) });
        module.add_function(func);

        let stats = optimize::optimize(&mut module);
        assert_eq!(stats.constants_folded, 0);
    }

    #[test]
    fn test_dead_code_elimination_with_unused_function() {
        let mut module = ir::IrModule::new("main");
        let mut main_func = IrFunction::new("main", IrType::Void);
        main_func.push_op(IrOp::Return { value: None });
        module.add_function(main_func);

        let mut helper = IrFunction::new("helper", IrType::I32);
        helper.push_op(IrOp::LoadConst {
            dest: 0,
            value: ConstValue::I32(42),
        });
        helper.push_op(IrOp::Return { value: Some(0) });
        module.add_function(helper);

        let stats = optimize::optimize(&mut module);
        assert!(stats.unused_functions_removed > 0);
        assert!(module.get_function("helper").is_none());
    }

    #[test]
    fn test_inline_small_function_in_module() {
        let mut module = ir::IrModule::new("main");
        let mut main_func = IrFunction::new("main", IrType::I32);
        main_func.push_op(IrOp::LoadConst {
            dest: 0,
            value: ConstValue::I32(5),
        });
        main_func.push_op(IrOp::Call {
            dest: Some(1),
            func: "double".into(),
            args: vec![0],
        });
        main_func.push_op(IrOp::Return { value: Some(1) });
        module.add_function(main_func);

        let mut double = IrFunction::new("double", IrType::I32);
        double.add_param("x", IrType::I32);
        double.push_op(IrOp::LoadVar {
            dest: 0,
            name: "x".into(),
        });
        double.push_op(IrOp::LoadVar {
            dest: 1,
            name: "x".into(),
        });
        double.push_op(IrOp::BinaryOp {
            dest: 2,
            op: BinaryOp::Add,
            lhs: 0,
            rhs: 1,
        });
        double.push_op(IrOp::Return { value: Some(2) });
        module.add_function(double);

        let stats = optimize::optimize(&mut module);
        assert!(stats.functions_inlined > 0);
    }

    #[test]
    fn test_multiple_optimization_passes() {
        let mut module = ir::IrModule::new("main");
        let mut main_func = IrFunction::new("main", IrType::I32);
        main_func.push_op(IrOp::LoadConst {
            dest: 0,
            value: ConstValue::I32(3),
        });
        main_func.push_op(IrOp::LoadConst {
            dest: 1,
            value: ConstValue::I32(4),
        });
        main_func.push_op(IrOp::BinaryOp {
            dest: 2,
            op: BinaryOp::Add,
            lhs: 0,
            rhs: 1,
        });
        main_func.push_op(IrOp::StoreVar {
            name: "unused".into(),
            src: 2,
        });
        main_func.push_op(IrOp::Return { value: Some(2) });
        module.add_function(main_func);

        let stats = optimize::optimize(&mut module);
        assert!(stats.constants_folded > 0);
        assert!(stats.unused_vars_removed > 0);
    }

    #[test]
    fn test_extract_basic_blocks_from_lowered() {
        let project = lower::make_if_else_project();
        let module = lower_project(&project).unwrap();
        let func = module.get_function("Sprite1_main").unwrap();
        let blocks = ir::extract_basic_blocks(func);
        assert!(!blocks.is_empty());
    }

    #[test]
    fn test_build_call_graph_from_lowered() {
        let project = lower::make_motion_project();
        let module = lower_project(&project).unwrap();
        let graph = ir::build_call_graph(&module);
        assert!(!graph.is_empty());
    }

    #[test]
    fn test_reachable_functions_from_lowered() {
        let project = lower::make_motion_project();
        let module = lower_project(&project).unwrap();
        let reachable = ir::reachable_functions(&module);
        assert!(reachable.contains(&"main".to_string()));
    }

    #[test]
    fn test_linker_with_module() {
        let project = lower::make_simple_project("test");
        let module = lower_project(&project).unwrap();
        let mut linker = Linker::new(Target::LinuxX64);
        let result = linker.link(&module, "main");
        assert!(result.is_ok());
    }

    #[test]
    fn test_codegen_target_selection() {
        for target in Target::all() {
            let gen = codegen::create_codegen(target);
            assert!(gen.is_ok(), "Failed for target {:?}", target);
        }
    }

    #[test]
    fn test_compile_with_headless_feature() {
        let features = FeatureFlags::new().with_headless();
        let config = CompileConfig::new(Target::LinuxX64, OptLevel::Release).with_features(features);
        let compiler = AotCompiler::new(config);
        let project = lower::make_simple_project("test");
        let result = compiler.compile_project_data(&project);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_with_all_features() {
        let features = FeatureFlags::new()
            .with_headless()
            .with_no_network()
            .with_debug_symbols()
            .with_static_link();
        let config = CompileConfig::new(Target::LinuxX64, OptLevel::Release).with_features(features);
        let compiler = AotCompiler::new(config);
        assert!(compiler.config().features.headless);
        assert!(compiler.config().features.no_network);
        assert!(compiler.config().features.debug_symbols);
        assert!(compiler.config().features.static_link);
    }

    #[test]
    fn test_ir_module_function_names() {
        let project = lower::make_simple_project("test");
        let module = lower_project(&project).unwrap();
        let names = module.function_names();
        assert!(names.contains(&"main"));
        assert!(names.contains(&"stage_main"));
    }

    #[test]
    fn test_ir_module_total_ops() {
        let project = lower::make_simple_project("test");
        let module = lower_project(&project).unwrap();
        assert!(module.total_ops() > 0);
    }

    #[test]
    fn test_ir_module_get_function_mut() {
        let mut module = ir::IrModule::new("main");
        let func = IrFunction::new("main", IrType::Void);
        module.add_function(func);
        {
            let func = module.get_function_mut("main").unwrap();
            func.push_op(IrOp::Return { value: None });
        }
        assert_eq!(module.get_function("main").unwrap().ops.len(), 1);
    }
}
