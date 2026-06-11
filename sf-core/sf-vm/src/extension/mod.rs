//! Extension system for the Sailfish VM.
//!
//! Provides a trait-based extension mechanism that allows additional
//! opcode handlers to be registered and dispatched at runtime.

use crate::project::Value;
use crate::runtime::RuntimeState;
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during extension operations.
#[derive(Error, Debug)]
pub enum ExtensionError {
    #[error("extension not found: {0}")]
    ExtensionNotFound(String),
    #[error("opcode not supported by extension: {0}")]
    OpcodeNotSupported(String),
    #[error("extension already registered: {0}")]
    AlreadyRegistered(String),
    #[error("execution error in extension '{extension}': {message}")]
    ExecutionError { extension: String, message: String },
}

/// Trait for Sailfish VM extensions.
///
/// Extensions provide additional opcode handlers beyond the built-in ones.
pub trait SfExtension: std::fmt::Debug {
    /// The name of this extension.
    fn name(&self) -> &str;

    /// The list of opcodes this extension supports.
    fn opcodes(&self) -> &[&str];

    /// Execute an opcode from this extension.
    ///
    /// # Arguments
    /// * `opcode` - The opcode to execute
    /// * `args` - The arguments for the opcode
    /// * `runtime` - Mutable reference to the runtime state
    ///
    /// # Returns
    /// The result value of the operation
    fn execute(
        &self,
        opcode: &str,
        args: &Value,
        runtime: &mut RuntimeState,
    ) -> Result<Value, ExtensionError>;
}

/// Manager for VM extensions.
///
/// Handles registration, unregistration, and dispatch of extension opcodes.
#[derive(Debug)]
pub struct ExtensionManager {
    extensions: HashMap<String, Box<dyn SfExtension>>,
    opcode_map: HashMap<String, String>, // opcode -> extension name
}

impl ExtensionManager {
    /// Create a new empty extension manager.
    pub fn new() -> Self {
        Self {
            extensions: HashMap::new(),
            opcode_map: HashMap::new(),
        }
    }

    /// Register an extension.
    pub fn register(&mut self, extension: Box<dyn SfExtension>) -> Result<(), ExtensionError> {
        let name = extension.name().to_string();
        if self.extensions.contains_key(&name) {
            return Err(ExtensionError::AlreadyRegistered(name));
        }

        // Register all opcodes
        for opcode in extension.opcodes() {
            self.opcode_map
                .insert(opcode.to_string(), name.clone());
        }

        self.extensions.insert(name, extension);
        Ok(())
    }

    /// Unregister an extension by name.
    pub fn unregister(&mut self, name: &str) -> Result<(), ExtensionError> {
        let extension = self
            .extensions
            .remove(name)
            .ok_or_else(|| ExtensionError::ExtensionNotFound(name.to_string()))?;

        // Remove all opcode mappings
        for opcode in extension.opcodes() {
            self.opcode_map.remove(*opcode);
        }

        Ok(())
    }

    /// Execute an opcode through the extension dispatch system.
    pub fn execute(
        &mut self,
        opcode: &str,
        args: &Value,
        runtime: &mut RuntimeState,
    ) -> Result<Value, ExtensionError> {
        let ext_name = self
            .opcode_map
            .get(opcode)
            .ok_or_else(|| ExtensionError::OpcodeNotSupported(opcode.to_string()))?
            .clone();

        // We need to temporarily remove the extension to avoid borrow conflicts
        // This is safe because we immediately put it back
        let extension = self.extensions.remove(&ext_name).unwrap();
        let result = extension.execute(opcode, args, runtime);
        self.extensions.insert(ext_name, extension);
        result
    }

    /// Check if an opcode is supported by any registered extension.
    pub fn supports_opcode(&self, opcode: &str) -> bool {
        self.opcode_map.contains_key(opcode)
    }

    /// Get the name of the extension that handles a given opcode.
    pub fn extension_for_opcode(&self, opcode: &str) -> Option<&str> {
        self.opcode_map.get(opcode).map(|s| s.as_str())
    }

    /// Get a list of all registered extension names.
    pub fn extension_names(&self) -> Vec<&str> {
        self.extensions.keys().map(|s| s.as_str()).collect()
    }

    /// Check if an extension is registered.
    pub fn is_registered(&self, name: &str) -> bool {
        self.extensions.contains_key(name)
    }
}

impl Default for ExtensionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Built-in Pen extension for testing.
#[derive(Debug, Clone)]
pub struct BuiltinPenExtension;

impl BuiltinPenExtension {
    /// Create a new built-in pen extension.
    pub fn new() -> Self {
        Self
    }
}

impl Default for BuiltinPenExtension {
    fn default() -> Self {
        Self::new()
    }
}

const PEN_OPCODES: &[&str] = &[
    "pen_clear",
    "pen_stamp",
    "pen_down",
    "pen_up",
    "pen_setColor",
];

impl SfExtension for BuiltinPenExtension {
    fn name(&self) -> &str {
        "pen"
    }

    fn opcodes(&self) -> &[&str] {
        PEN_OPCODES
    }

    fn execute(
        &self,
        opcode: &str,
        args: &Value,
        runtime: &mut RuntimeState,
    ) -> Result<Value, ExtensionError> {
        match opcode {
            "pen_clear" => {
                // Clear all pen trails
                runtime.push_event(crate::runtime::RuntimeEvent::Broadcast {
                    name: "__pen_clear".to_string(),
                });
                Ok(Value::Null)
            }
            "pen_stamp" => {
                // Stamp current costume at current position
                runtime.push_event(crate::runtime::RuntimeEvent::Broadcast {
                    name: "__pen_stamp".to_string(),
                });
                Ok(Value::Null)
            }
            "pen_down" => {
                if let Some(target) = runtime.current_target_state_mut() {
                    target.pen_down = true;
                }
                Ok(Value::Null)
            }
            "pen_up" => {
                if let Some(target) = runtime.current_target_state_mut() {
                    target.pen_down = false;
                }
                Ok(Value::Null)
            }
            "pen_setColor" => {
                if let Some(target) = runtime.current_target_state_mut() {
                    if let Some(color) = args.as_string() {
                        target.pen_color = color;
                    }
                }
                Ok(Value::Null)
            }
            _ => Err(ExtensionError::OpcodeNotSupported(opcode.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_manager_new() {
        let manager = ExtensionManager::new();
        assert!(manager.extension_names().is_empty());
        assert!(!manager.supports_opcode("pen_clear"));
    }

    #[test]
    fn test_register_extension() {
        let mut manager = ExtensionManager::new();
        manager
            .register(Box::new(BuiltinPenExtension::new()))
            .expect("should register");

        assert!(manager.is_registered("pen"));
        assert!(manager.supports_opcode("pen_clear"));
        assert!(manager.supports_opcode("pen_stamp"));
        assert!(manager.supports_opcode("pen_down"));
        assert!(manager.supports_opcode("pen_up"));
        assert!(manager.supports_opcode("pen_setColor"));
        assert_eq!(manager.extension_names(), vec!["pen"]);
    }

    #[test]
    fn test_register_duplicate_extension() {
        let mut manager = ExtensionManager::new();
        manager
            .register(Box::new(BuiltinPenExtension::new()))
            .expect("should register");
        let result = manager.register(Box::new(BuiltinPenExtension::new()));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already registered"));
    }

    #[test]
    fn test_unregister_extension() {
        let mut manager = ExtensionManager::new();
        manager
            .register(Box::new(BuiltinPenExtension::new()))
            .expect("should register");

        manager.unregister("pen").expect("should unregister");
        assert!(!manager.is_registered("pen"));
        assert!(!manager.supports_opcode("pen_clear"));
    }

    #[test]
    fn test_unregister_nonexistent() {
        let mut manager = ExtensionManager::new();
        let result = manager.unregister("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_pen_down() {
        let mut manager = ExtensionManager::new();
        manager
            .register(Box::new(BuiltinPenExtension::new()))
            .expect("should register");

        let mut runtime = RuntimeState::new();
        runtime.current_target = "Cat".to_string();
        runtime.add_target(crate::runtime::TargetState::new_sprite("Cat"));

        let result = manager
            .execute("pen_down", &Value::Null, &mut runtime)
            .expect("should execute");
        assert!(result.is_null());
        assert!(runtime.current_target_state().unwrap().pen_down);
    }

    #[test]
    fn test_execute_pen_up() {
        let mut manager = ExtensionManager::new();
        manager
            .register(Box::new(BuiltinPenExtension::new()))
            .expect("should register");

        let mut runtime = RuntimeState::new();
        runtime.current_target = "Cat".to_string();
        let mut target = crate::runtime::TargetState::new_sprite("Cat");
        target.pen_down = true;
        runtime.add_target(target);

        let result = manager
            .execute("pen_up", &Value::Null, &mut runtime)
            .expect("should execute");
        assert!(result.is_null());
        assert!(!runtime.current_target_state().unwrap().pen_down);
    }

    #[test]
    fn test_execute_pen_set_color() {
        let mut manager = ExtensionManager::new();
        manager
            .register(Box::new(BuiltinPenExtension::new()))
            .expect("should register");

        let mut runtime = RuntimeState::new();
        runtime.current_target = "Cat".to_string();
        runtime.add_target(crate::runtime::TargetState::new_sprite("Cat"));

        let result = manager
            .execute("pen_setColor", &Value::String("#ff0000".to_string()), &mut runtime)
            .expect("should execute");
        assert!(result.is_null());
        assert_eq!(runtime.current_target_state().unwrap().pen_color, "#ff0000");
    }

    #[test]
    fn test_execute_pen_clear() {
        let mut manager = ExtensionManager::new();
        manager
            .register(Box::new(BuiltinPenExtension::new()))
            .expect("should register");

        let mut runtime = RuntimeState::new();
        let result = manager
            .execute("pen_clear", &Value::Null, &mut runtime)
            .expect("should execute");
        assert!(result.is_null());
    }

    #[test]
    fn test_execute_pen_stamp() {
        let mut manager = ExtensionManager::new();
        manager
            .register(Box::new(BuiltinPenExtension::new()))
            .expect("should register");

        let mut runtime = RuntimeState::new();
        let result = manager
            .execute("pen_stamp", &Value::Null, &mut runtime)
            .expect("should execute");
        assert!(result.is_null());
    }

    #[test]
    fn test_execute_unsupported_opcode() {
        let mut manager = ExtensionManager::new();
        let mut runtime = RuntimeState::new();
        let result = manager.execute("nonexistent_opcode", &Value::Null, &mut runtime);
        assert!(result.is_err());
    }

    #[test]
    fn test_extension_for_opcode() {
        let mut manager = ExtensionManager::new();
        manager
            .register(Box::new(BuiltinPenExtension::new()))
            .expect("should register");

        assert_eq!(manager.extension_for_opcode("pen_clear"), Some("pen"));
        assert_eq!(manager.extension_for_opcode("nonexistent"), None);
    }

    #[test]
    fn test_custom_extension() {
        #[derive(Debug)]
        struct TestExtension;

        impl SfExtension for TestExtension {
            fn name(&self) -> &str {
                "test"
            }
            fn opcodes(&self) -> &[&str] {
                &["test_hello", "test_world"]
            }
            fn execute(
                &self,
                opcode: &str,
                _args: &Value,
                _runtime: &mut RuntimeState,
            ) -> Result<Value, ExtensionError> {
                match opcode {
                    "test_hello" => Ok(Value::String("hello".to_string())),
                    "test_world" => Ok(Value::String("world".to_string())),
                    _ => Err(ExtensionError::OpcodeNotSupported(opcode.to_string())),
                }
            }
        }

        let mut manager = ExtensionManager::new();
        manager
            .register(Box::new(TestExtension))
            .expect("should register");

        assert!(manager.is_registered("test"));
        assert!(manager.supports_opcode("test_hello"));
        assert!(manager.supports_opcode("test_world"));

        let mut runtime = RuntimeState::new();
        let result = manager
            .execute("test_hello", &Value::Null, &mut runtime)
            .expect("should execute");
        assert_eq!(result, Value::String("hello".to_string()));

        let result = manager
            .execute("test_world", &Value::Null, &mut runtime)
            .expect("should execute");
        assert_eq!(result, Value::String("world".to_string()));
    }
}
