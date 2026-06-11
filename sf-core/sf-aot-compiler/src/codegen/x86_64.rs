//! x86_64 code generation backend (stub).
//!
//! Defines the instruction encoding interface for x86_64.
//! Actual machine code generation is not yet implemented.

use crate::codegen::{CodegenError, CodegenResult, CodeGenerator, NativeCode, Target};
use crate::ir::IrModule;

/// x86_64 code generator.
pub struct X86_64Codegen {
    target: Target,
}

impl X86_64Codegen {
    /// Create a new x86_64 code generator for the given target.
    pub fn new(target: Target) -> Self {
        Self { target }
    }

    /// Encode a RET instruction (0xC3).
    pub fn encode_ret() -> Vec<u8> {
        vec![0xC3]
    }

    /// Encode a NOP instruction (0x90).
    pub fn encode_nop() -> Vec<u8> {
        vec![0x90]
    }

    /// Encode a simple function prologue: push rbp; mov rbp, rsp
    pub fn encode_prologue() -> Vec<u8> {
        vec![0x55, 0x48, 0x89, 0xE5]
    }

    /// Encode a simple function epilogue: pop rbp; ret
    pub fn encode_epilogue() -> Vec<u8> {
        vec![0x5D, 0xC3]
    }

    /// Encode a MOV RAX, imm64 instruction.
    pub fn encode_mov_rax_imm64(val: u64) -> Vec<u8> {
        let mut bytes = vec![0x48, 0xB8];
        bytes.extend_from_slice(&val.to_le_bytes());
        bytes
    }

    /// Encode an ADD RAX, imm32 instruction.
    pub fn encode_add_rax_imm32(val: i32) -> Vec<u8> {
        let mut bytes = vec![0x48, 0x05];
        bytes.extend_from_slice(&val.to_le_bytes());
        bytes
    }

    /// Encode a SUB RAX, imm32 instruction.
    pub fn encode_sub_rax_imm32(val: i32) -> Vec<u8> {
        let mut bytes = vec![0x48, 0x2D];
        bytes.extend_from_slice(&val.to_le_bytes());
        bytes
    }

    /// Get the target this codegen is for.
    pub fn get_target(&self) -> Target {
        self.target
    }
}

impl CodeGenerator for X86_64Codegen {
    fn generate(&mut self, _module: &IrModule) -> CodegenResult<NativeCode> {
        Err(CodegenError::NotYetImplemented(format!(
            "x86_64 code generation for {}",
            self.target
        )))
    }

    fn target(&self) -> Target {
        self.target
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_ret() {
        assert_eq!(X86_64Codegen::encode_ret(), vec![0xC3]);
    }

    #[test]
    fn test_encode_nop() {
        assert_eq!(X86_64Codegen::encode_nop(), vec![0x90]);
    }

    #[test]
    fn test_encode_prologue() {
        let prologue = X86_64Codegen::encode_prologue();
        assert_eq!(prologue, vec![0x55, 0x48, 0x89, 0xE5]);
    }

    #[test]
    fn test_encode_epilogue() {
        let epilogue = X86_64Codegen::encode_epilogue();
        assert_eq!(epilogue, vec![0x5D, 0xC3]);
    }

    #[test]
    fn test_encode_mov_rax_imm64() {
        let encoded = X86_64Codegen::encode_mov_rax_imm64(42);
        assert_eq!(encoded[0], 0x48);
        assert_eq!(encoded[1], 0xB8);
        // 42 as little-endian u64
        assert_eq!(encoded[2..10], 42u64.to_le_bytes());
    }

    #[test]
    fn test_encode_add_rax_imm32() {
        let encoded = X86_64Codegen::encode_add_rax_imm32(10);
        assert_eq!(encoded[0], 0x48);
        assert_eq!(encoded[1], 0x05);
        assert_eq!(encoded[2..6], 10i32.to_le_bytes());
    }

    #[test]
    fn test_encode_sub_rax_imm32() {
        let encoded = X86_64Codegen::encode_sub_rax_imm32(5);
        assert_eq!(encoded[0], 0x48);
        assert_eq!(encoded[1], 0x2D);
        assert_eq!(encoded[2..6], 5i32.to_le_bytes());
    }

    #[test]
    fn test_codegen_new() {
        let gen = X86_64Codegen::new(Target::LinuxX64);
        assert_eq!(gen.get_target(), Target::LinuxX64);
    }

    #[test]
    fn test_codegen_generate_stub() {
        let mut gen = X86_64Codegen::new(Target::LinuxX64);
        let module = crate::ir::IrModule::new("main");
        let result = gen.generate(&module);
        assert!(result.is_err());
    }

    #[test]
    fn test_codegen_windows_target() {
        let gen = X86_64Codegen::new(Target::WindowsX64);
        assert_eq!(gen.get_target(), Target::WindowsX64);
    }

    #[test]
    fn test_codegen_macos_target() {
        let gen = X86_64Codegen::new(Target::MacosX64);
        assert_eq!(gen.get_target(), Target::MacosX64);
    }

    #[test]
    fn test_mov_rax_zero() {
        let encoded = X86_64Codegen::encode_mov_rax_imm64(0);
        assert_eq!(encoded.len(), 10);
    }

    #[test]
    fn test_add_rax_negative() {
        let encoded = X86_64Codegen::encode_add_rax_imm32(-1);
        assert_eq!(encoded.len(), 6);
    }
}
