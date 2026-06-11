//! Linker for the Sailfish AOT Compiler.
//!
//! Links IR modules together, resolves function references,
//! and adds runtime startup code. Currently a stub with a clear
//! interface for future implementation.

use crate::codegen::{NativeCode, Target};
use crate::ir::IrModule;
use std::collections::HashMap;
use thiserror::Error;

/// Errors during linking.
#[derive(Debug, Error)]
pub enum LinkError {
    #[error("undefined symbol: {0}")]
    UndefinedSymbol(String),
    #[error("duplicate symbol: {0}")]
    DuplicateSymbol(String),
    #[error("missing entry point: {0}")]
    MissingEntryPoint(String),
    #[error("linking error: {0}")]
    General(String),
}

/// Result type for linking.
pub type LinkResult<T> = Result<T, LinkError>;

/// A symbol in the linker's symbol table.
#[derive(Debug, Clone)]
pub struct LinkSymbol {
    pub name: String,
    pub offset: usize,
    pub size: usize,
    pub symbol_type: SymbolType,
}

/// The type of a linker symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolType {
    Function,
    Global,
    External,
}

/// The linker — resolves symbols and produces a final executable image.
pub struct Linker {
    target: Target,
    symbols: HashMap<String, LinkSymbol>,
    object_files: Vec<NativeCode>,
    runtime_code: Vec<u8>,
}

impl Linker {
    /// Create a new linker for the given target.
    pub fn new(target: Target) -> Self {
        Self {
            target,
            symbols: HashMap::new(),
            object_files: Vec::new(),
            runtime_code: Vec::new(),
        }
    }

    /// Add an object file (native code) to the linker.
    pub fn add_object(&mut self, code: NativeCode) {
        self.object_files.push(code);
    }

    /// Add a symbol definition.
    pub fn define_symbol(&mut self, name: &str, offset: usize, size: usize, symbol_type: SymbolType) -> LinkResult<()> {
        if self.symbols.contains_key(name) {
            return Err(LinkError::DuplicateSymbol(name.to_string()));
        }
        self.symbols.insert(name.to_string(), LinkSymbol {
            name: name.to_string(),
            offset,
            size,
            symbol_type,
        });
        Ok(())
    }

    /// Look up a symbol by name.
    pub fn resolve_symbol(&self, name: &str) -> Option<&LinkSymbol> {
        self.symbols.get(name)
    }

    /// Check if a symbol is defined.
    pub fn has_symbol(&self, name: &str) -> bool {
        self.symbols.contains_key(name)
    }

    /// Resolve all function references in the module.
    pub fn resolve_module_references(&self, module: &IrModule) -> LinkResult<Vec<UnresolvedReference>> {
        let mut unresolved = Vec::new();

        for func in &module.functions {
            for called in func.called_functions() {
                if !self.has_symbol(called) && !crate::ir::is_builtin(called) {
                    // Check if it's defined in the module itself
                    if !module.get_function(called).is_some() {
                        unresolved.push(UnresolvedReference {
                            from_function: func.name.clone(),
                            symbol: called.to_string(),
                        });
                    }
                }
            }
        }

        Ok(unresolved)
    }

    /// Add runtime startup code (stub).
    pub fn add_runtime_startup(&mut self) {
        // In a real implementation, this would add:
        // - C runtime startup (crt0)
        // - Sailfish runtime initialization
        // - Standard library functions
        // For now, we just add a placeholder
        self.runtime_code = vec![0x90]; // NOP
    }

    /// Link everything together and produce a final executable image.
    pub fn link(&mut self, module: &IrModule, entry_point: &str) -> LinkResult<NativeCode> {
        // Check entry point exists
        if !module.get_function(entry_point).is_some() && !self.has_symbol(entry_point) {
            return Err(LinkError::MissingEntryPoint(entry_point.to_string()));
        }

        // Resolve all references
        let unresolved = self.resolve_module_references(module)?;
        if !unresolved.is_empty() {
            return Err(LinkError::UndefinedSymbol(unresolved[0].symbol.clone()));
        }

        // Add runtime startup
        self.add_runtime_startup();

        // Define symbols for all module functions
        let mut offset = self.runtime_code.len();
        for func in &module.functions {
            self.define_symbol(&func.name, offset, 0, SymbolType::Function)?;
            offset += 64; // Placeholder function size
        }

        // Combine all code
        let mut combined = self.runtime_code.clone();
        for obj in &self.object_files {
            combined.extend_from_slice(&obj.bytes);
        }

        Ok(NativeCode {
            bytes: combined,
            entry_point_offset: 0,
            symbol_table: self.symbols.iter().map(|(k, v)| (k.clone(), v.offset)).collect(),
            target: self.target,
        })
    }

    /// Get the number of defined symbols.
    pub fn symbol_count(&self) -> usize {
        self.symbols.len()
    }

    /// Get the target this linker is for.
    pub fn target(&self) -> Target {
        self.target
    }
}

/// An unresolved reference found during linking.
#[derive(Debug, Clone)]
pub struct UnresolvedReference {
    pub from_function: String,
    pub symbol: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::*;

    #[test]
    fn test_linker_new() {
        let linker = Linker::new(Target::LinuxX64);
        assert_eq!(linker.target(), Target::LinuxX64);
        assert_eq!(linker.symbol_count(), 0);
    }

    #[test]
    fn test_define_symbol() {
        let mut linker = Linker::new(Target::LinuxX64);
        linker.define_symbol("main", 0, 64, SymbolType::Function).unwrap();
        assert!(linker.has_symbol("main"));
        assert_eq!(linker.symbol_count(), 1);
    }

    #[test]
    fn test_duplicate_symbol() {
        let mut linker = Linker::new(Target::LinuxX64);
        linker.define_symbol("main", 0, 64, SymbolType::Function).unwrap();
        let result = linker.define_symbol("main", 100, 64, SymbolType::Function);
        assert!(matches!(result, Err(LinkError::DuplicateSymbol(_))));
    }

    #[test]
    fn test_resolve_symbol() {
        let mut linker = Linker::new(Target::LinuxX64);
        linker.define_symbol("helper", 100, 32, SymbolType::Function).unwrap();
        let sym = linker.resolve_symbol("helper").unwrap();
        assert_eq!(sym.offset, 100);
        assert_eq!(sym.symbol_type, SymbolType::Function);
    }

    #[test]
    fn test_resolve_missing_symbol() {
        let linker = Linker::new(Target::LinuxX64);
        assert!(linker.resolve_symbol("missing").is_none());
    }

    #[test]
    fn test_resolve_module_references_ok() {
        let mut module = IrModule::new("main");
        let mut main_func = IrFunction::new("main", IrType::Void);
        main_func.push_op(IrOp::Call { dest: None, func: "helper".into(), args: vec![] });
        main_func.push_op(IrOp::Return { value: None });
        module.add_function(main_func);

        let helper = IrFunction::new("helper", IrType::Void);
        module.add_function(helper);

        let linker = Linker::new(Target::LinuxX64);
        let unresolved = linker.resolve_module_references(&module).unwrap();
        assert!(unresolved.is_empty());
    }

    #[test]
    fn test_resolve_module_references_missing() {
        let mut module = IrModule::new("main");
        let mut main_func = IrFunction::new("main", IrType::Void);
        main_func.push_op(IrOp::Call { dest: None, func: "missing_func".into(), args: vec![] });
        main_func.push_op(IrOp::Return { value: None });
        module.add_function(main_func);

        let linker = Linker::new(Target::LinuxX64);
        let unresolved = linker.resolve_module_references(&module).unwrap();
        assert_eq!(unresolved.len(), 1);
        assert_eq!(unresolved[0].symbol, "missing_func");
    }

    #[test]
    fn test_resolve_builtin_symbols() {
        let mut module = IrModule::new("main");
        let mut main_func = IrFunction::new("main", IrType::Void);
        main_func.push_op(IrOp::Call { dest: None, func: "print".into(), args: vec![0] });
        main_func.push_op(IrOp::Return { value: None });
        module.add_function(main_func);

        let linker = Linker::new(Target::LinuxX64);
        let unresolved = linker.resolve_module_references(&module).unwrap();
        assert!(unresolved.is_empty()); // print is a builtin
    }

    #[test]
    fn test_link_missing_entry_point() {
        let mut linker = Linker::new(Target::LinuxX64);
        let module = IrModule::new("main");
        let result = linker.link(&module, "main");
        assert!(matches!(result, Err(LinkError::MissingEntryPoint(_))));
    }

    #[test]
    fn test_link_success() {
        let mut linker = Linker::new(Target::LinuxX64);
        let mut module = IrModule::new("main");
        let func = IrFunction::new("main", IrType::Void);
        module.add_function(func);
        let result = linker.link(&module, "main");
        assert!(result.is_ok());
        let native = result.unwrap();
        assert!(!native.bytes.is_empty());
    }

    #[test]
    fn test_link_adds_runtime() {
        let mut linker = Linker::new(Target::LinuxX64);
        let mut module = IrModule::new("main");
        let func = IrFunction::new("main", IrType::Void);
        module.add_function(func);
        let native = linker.link(&module, "main").unwrap();
        // Runtime code should be at the start
        assert!(native.bytes.len() > 0);
    }

    #[test]
    fn test_add_object() {
        let mut linker = Linker::new(Target::LinuxX64);
        let obj = NativeCode::empty(Target::LinuxX64);
        linker.add_object(obj);
    }

    #[test]
    fn test_symbol_types() {
        let mut linker = Linker::new(Target::LinuxX64);
        linker.define_symbol("func", 0, 64, SymbolType::Function).unwrap();
        linker.define_symbol("global_var", 64, 8, SymbolType::Global).unwrap();
        linker.define_symbol("ext_func", 0, 0, SymbolType::External).unwrap();

        assert_eq!(linker.resolve_symbol("func").unwrap().symbol_type, SymbolType::Function);
        assert_eq!(linker.resolve_symbol("global_var").unwrap().symbol_type, SymbolType::Global);
        assert_eq!(linker.resolve_symbol("ext_func").unwrap().symbol_type, SymbolType::External);
    }

    #[test]
    fn test_unresolved_reference() {
        let r = UnresolvedReference {
            from_function: "main".into(),
            symbol: "missing".into(),
        };
        assert_eq!(r.from_function, "main");
        assert_eq!(r.symbol, "missing");
    }
}
