//! Code generation interface for the Sailfish AOT Compiler.
//!
//! Defines the `CodeGenerator` trait and target-specific backends.
//! Actual code generation is stubbed with clear interfaces for future
//! LLVM or Cranelift integration.

pub mod aarch64;
pub mod x86_64;

use crate::ir::IrModule;
use std::collections::HashMap;
use thiserror::Error;

/// Errors during code generation.
#[derive(Debug, Error)]
pub enum CodegenError {
    #[error("code generation not yet implemented for target: {0}")]
    NotYetImplemented(String),
    #[error("unsupported target: {0}")]
    UnsupportedTarget(String),
    #[error("code generation error: {0}")]
    General(String),
}

/// Result type for code generation.
pub type CodegenResult<T> = Result<T, CodegenError>;

/// The target platform for code generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Target {
    WindowsX64,
    MacosArm64,
    MacosX64,
    LinuxX64,
}

use serde::{Deserialize, Serialize};

impl Target {
    /// Returns the architecture name.
    pub fn arch(&self) -> &str {
        match self {
            Target::WindowsX64 | Target::MacosX64 | Target::LinuxX64 => "x86_64",
            Target::MacosArm64 => "aarch64",
        }
    }

    /// Returns the OS name.
    pub fn os(&self) -> &str {
        match self {
            Target::WindowsX64 => "windows",
            Target::MacosArm64 | Target::MacosX64 => "macos",
            Target::LinuxX64 => "linux",
        }
    }

    /// Returns the LLVM target triple.
    pub fn llvm_triple(&self) -> &str {
        match self {
            Target::WindowsX64 => "x86_64-pc-windows-msvc",
            Target::MacosArm64 => "aarch64-apple-macos",
            Target::MacosX64 => "x86_64-apple-macos",
            Target::LinuxX64 => "x86_64-unknown-linux-gnu",
        }
    }

    /// Returns the executable file extension for this target.
    pub fn exe_extension(&self) -> &str {
        match self {
            Target::WindowsX64 => ".exe",
            _ => "",
        }
    }

    /// Returns true if this is a 64-bit target.
    pub fn is_64bit(&self) -> bool {
        true // All current targets are 64-bit
    }

    /// Create from arch and os strings.
    pub fn from_arch_os(arch: &str, os: &str) -> Option<Self> {
        match (arch, os) {
            ("x86_64", "windows") => Some(Target::WindowsX64),
            ("aarch64", "macos") => Some(Target::MacosArm64),
            ("x86_64", "macos") => Some(Target::MacosX64),
            ("x86_64", "linux") => Some(Target::LinuxX64),
            _ => None,
        }
    }

    /// All supported targets.
    pub fn all() -> Vec<Target> {
        vec![
            Target::WindowsX64,
            Target::MacosArm64,
            Target::MacosX64,
            Target::LinuxX64,
        ]
    }
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.arch(), self.os())
    }
}

/// Native code output from code generation.
#[derive(Debug, Clone)]
pub struct NativeCode {
    /// The generated machine code bytes.
    pub bytes: Vec<u8>,
    /// Offset of the entry point function in the bytes.
    pub entry_point_offset: usize,
    /// Symbol table mapping function names to offsets.
    pub symbol_table: HashMap<String, usize>,
    /// The target this code was generated for.
    pub target: Target,
}

impl NativeCode {
    /// Create empty native code for a target.
    pub fn empty(target: Target) -> Self {
        Self {
            bytes: Vec::new(),
            entry_point_offset: 0,
            symbol_table: HashMap::new(),
            target,
        }
    }

    /// Returns the size of the generated code in bytes.
    pub fn size(&self) -> usize {
        self.bytes.len()
    }

    /// Add a symbol to the table.
    pub fn add_symbol(&mut self, name: &str, offset: usize) {
        self.symbol_table.insert(name.to_string(), offset);
    }

    /// Look up a symbol by name.
    pub fn get_symbol(&self, name: &str) -> Option<usize> {
        self.symbol_table.get(name).copied()
    }
}

/// The code generator trait.
pub trait CodeGenerator {
    /// Generate native code from an IR module.
    fn generate(&mut self, module: &IrModule) -> CodegenResult<NativeCode>;

    /// Get the target this generator produces code for.
    fn target(&self) -> Target;
}

/// Create a code generator for the given target.
pub fn create_codegen(target: Target) -> CodegenResult<Box<dyn CodeGenerator>> {
    match target {
        Target::WindowsX64 | Target::MacosX64 | Target::LinuxX64 => {
            Ok(Box::new(x86_64::X86_64Codegen::new(target)))
        }
        Target::MacosArm64 => Ok(Box::new(aarch64::Aarch64Codegen::new(target))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_arch() {
        assert_eq!(Target::WindowsX64.arch(), "x86_64");
        assert_eq!(Target::MacosArm64.arch(), "aarch64");
        assert_eq!(Target::LinuxX64.arch(), "x86_64");
    }

    #[test]
    fn test_target_os() {
        assert_eq!(Target::WindowsX64.os(), "windows");
        assert_eq!(Target::MacosArm64.os(), "macos");
        assert_eq!(Target::LinuxX64.os(), "linux");
    }

    #[test]
    fn test_target_llvm_triple() {
        assert_eq!(Target::WindowsX64.llvm_triple(), "x86_64-pc-windows-msvc");
        assert_eq!(Target::MacosArm64.llvm_triple(), "aarch64-apple-macos");
        assert_eq!(Target::LinuxX64.llvm_triple(), "x86_64-unknown-linux-gnu");
    }

    #[test]
    fn test_target_exe_extension() {
        assert_eq!(Target::WindowsX64.exe_extension(), ".exe");
        assert_eq!(Target::MacosX64.exe_extension(), "");
        assert_eq!(Target::LinuxX64.exe_extension(), "");
    }

    #[test]
    fn test_target_is_64bit() {
        for target in Target::all() {
            assert!(target.is_64bit());
        }
    }

    #[test]
    fn test_target_from_arch_os() {
        assert_eq!(Target::from_arch_os("x86_64", "linux"), Some(Target::LinuxX64));
        assert_eq!(Target::from_arch_os("aarch64", "macos"), Some(Target::MacosArm64));
        assert_eq!(Target::from_arch_os("x86_64", "windows"), Some(Target::WindowsX64));
        assert_eq!(Target::from_arch_os("arm", "windows"), None);
    }

    #[test]
    fn test_target_display() {
        assert_eq!(Target::LinuxX64.to_string(), "x86_64-linux");
        assert_eq!(Target::MacosArm64.to_string(), "aarch64-macos");
    }

    #[test]
    fn test_target_all() {
        let all = Target::all();
        assert_eq!(all.len(), 4);
    }

    #[test]
    fn test_native_code_empty() {
        let code = NativeCode::empty(Target::LinuxX64);
        assert_eq!(code.size(), 0);
        assert_eq!(code.target, Target::LinuxX64);
    }

    #[test]
    fn test_native_code_add_symbol() {
        let mut code = NativeCode::empty(Target::LinuxX64);
        code.add_symbol("main", 0);
        code.add_symbol("helper", 100);
        assert_eq!(code.get_symbol("main"), Some(0));
        assert_eq!(code.get_symbol("helper"), Some(100));
        assert_eq!(code.get_symbol("missing"), None);
    }

    #[test]
    fn test_native_code_with_bytes() {
        let code = NativeCode {
            bytes: vec![0x90, 0xC3], // NOP, RET
            entry_point_offset: 0,
            symbol_table: HashMap::new(),
            target: Target::LinuxX64,
        };
        assert_eq!(code.size(), 2);
    }

    #[test]
    fn test_create_codegen_x86_64() {
        let gen = create_codegen(Target::LinuxX64).unwrap();
        assert_eq!(gen.target(), Target::LinuxX64);
    }

    #[test]
    fn test_create_codegen_aarch64() {
        let gen = create_codegen(Target::MacosArm64).unwrap();
        assert_eq!(gen.target(), Target::MacosArm64);
    }

    #[test]
    fn test_codegen_not_yet_implemented() {
        let mut gen = create_codegen(Target::LinuxX64).unwrap();
        let module = crate::ir::IrModule::new("main");
        let result = gen.generate(&module);
        assert!(result.is_err());
        assert!(matches!(result, Err(CodegenError::NotYetImplemented(_))));
    }
}
