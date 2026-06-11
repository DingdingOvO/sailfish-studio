//! # sf-vm
//!
//! The core virtual machine for the Sailfish Studio project.
//!
//! This crate provides the building blocks for compiling and executing
//! block-based programs, including:
//!
//! - **project**: Data structures for projects, targets, blocks, and values
//! - **compiler**: Block tree traversal and JavaScript code generation
//! - **runtime**: Execution state management with event queue and timers
//! - **extension**: Plugin system for additional opcode handlers
//! - **settings**: 4-layer priority settings engine
//! - **ops**: Opcode definitions, categories, and execution dispatch

pub mod compiler;
pub mod extension;
pub mod ops;
pub mod project;
pub mod runtime;
pub mod settings;

// Re-export the most commonly used types
pub use project::{
    Block, BlockField, BlockInput, Costume, Project, ProjectError, ProjectSettings, Sound, Target,
    Value,
};

pub use runtime::{
    CloneData, RuntimeError, RuntimeEvent, RuntimeState, TargetState, ThreadState,
};

pub use compiler::{compile, compile_block, compile_target, CompilerError};

pub use extension::{
    BuiltinPenExtension, ExtensionError, ExtensionManager, SfExtension,
};

pub use settings::{
    SettingsEngine, SettingsError, SettingsLayer, SettingsValue,
};

pub use ops::{from_opcode_str, Opcode, OpcodeCategory, OpcodeError};
