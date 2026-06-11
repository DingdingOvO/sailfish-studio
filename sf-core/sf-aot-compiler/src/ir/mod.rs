//! Intermediate Representation for the Sailfish AOT Compiler.
//!
//! Defines the IR types that represent a lowered program:
//! `IrOp`, `IrFunction`, `IrModule`, `IrType`, etc.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Types in the IR type system.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IrType {
    Void,
    I32,
    I64,
    F64,
    Bool,
    String,
    Ptr,
}

impl fmt::Display for IrType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IrType::Void => write!(f, "void"),
            IrType::I32 => write!(f, "i32"),
            IrType::I64 => write!(f, "i64"),
            IrType::F64 => write!(f, "f64"),
            IrType::Bool => write!(f, "bool"),
            IrType::String => write!(f, "string"),
            IrType::Ptr => write!(f, "ptr"),
        }
    }
}

impl IrType {
    /// Returns true if this type is numeric (I32, I64, F64).
    pub fn is_numeric(&self) -> bool {
        matches!(self, IrType::I32 | IrType::I64 | IrType::F64)
    }

    /// Returns the default value for this type as a ConstValue.
    pub fn default_value(&self) -> ConstValue {
        match self {
            IrType::Void => ConstValue::Unit,
            IrType::I32 => ConstValue::I32(0),
            IrType::I64 => ConstValue::I64(0),
            IrType::F64 => ConstValue::F64(0.0),
            IrType::Bool => ConstValue::Bool(false),
            IrType::String => ConstValue::String(String::new()),
            IrType::Ptr => ConstValue::Null,
        }
    }
}

/// Constant values that can appear in the IR.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConstValue {
    Unit,
    I32(i32),
    I64(i64),
    F64(f64),
    Bool(bool),
    String(String),
    Null,
}

impl fmt::Display for ConstValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConstValue::Unit => write!(f, "()"),
            ConstValue::I32(v) => write!(f, "{}", v),
            ConstValue::I64(v) => write!(f, "{}L", v),
            ConstValue::F64(v) => write!(f, "{:.6}", v),
            ConstValue::Bool(v) => write!(f, "{}", v),
            ConstValue::String(v) => write!(f, "\"{}\"", v),
            ConstValue::Null => write!(f, "null"),
        }
    }
}

impl ConstValue {
    /// Get the IrType of this constant value.
    pub fn ir_type(&self) -> IrType {
        match self {
            ConstValue::Unit => IrType::Void,
            ConstValue::I32(_) => IrType::I32,
            ConstValue::I64(_) => IrType::I64,
            ConstValue::F64(_) => IrType::F64,
            ConstValue::Bool(_) => IrType::Bool,
            ConstValue::String(_) => IrType::String,
            ConstValue::Null => IrType::Ptr,
        }
    }

    /// Try to convert to i32.
    pub fn as_i32(&self) -> Option<i32> {
        match self {
            ConstValue::I32(v) => Some(*v),
            ConstValue::I64(v) => Some(*v as i32),
            ConstValue::F64(v) => Some(*v as i32),
            ConstValue::Bool(v) => Some(if *v { 1 } else { 0 }),
            _ => None,
        }
    }

    /// Try to convert to f64.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            ConstValue::I32(v) => Some(*v as f64),
            ConstValue::I64(v) => Some(*v as f64),
            ConstValue::F64(v) => Some(*v),
            _ => None,
        }
    }

    /// Try to convert to bool.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ConstValue::Bool(v) => Some(*v),
            ConstValue::I32(v) => Some(*v != 0),
            ConstValue::I64(v) => Some(*v != 0),
            ConstValue::F64(v) => Some(*v != 0.0),
            _ => None,
        }
    }

    /// Try to convert to string.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            ConstValue::String(v) => Some(v),
            _ => None,
        }
    }
}

/// Binary operations in the IR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::Sub => write!(f, "-"),
            BinaryOp::Mul => write!(f, "*"),
            BinaryOp::Div => write!(f, "/"),
            BinaryOp::Mod => write!(f, "%"),
            BinaryOp::Eq => write!(f, "=="),
            BinaryOp::Ne => write!(f, "!="),
            BinaryOp::Lt => write!(f, "<"),
            BinaryOp::Gt => write!(f, ">"),
            BinaryOp::Le => write!(f, "<="),
            BinaryOp::Ge => write!(f, ">="),
            BinaryOp::And => write!(f, "&&"),
            BinaryOp::Or => write!(f, "||"),
        }
    }
}

impl BinaryOp {
    /// Returns true if this is an arithmetic binary op.
    pub fn is_arithmetic(&self) -> bool {
        matches!(self, BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod)
    }

    /// Returns true if this is a comparison binary op.
    pub fn is_comparison(&self) -> bool {
        matches!(self, BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge)
    }

    /// Returns true if this is a logical binary op.
    pub fn is_logical(&self) -> bool {
        matches!(self, BinaryOp::And | BinaryOp::Or)
    }

    /// Evaluate the binary op on two constant values.
    pub fn eval(&self, lhs: &ConstValue, rhs: &ConstValue) -> Option<ConstValue> {
        match self {
            BinaryOp::Add => {
                match (lhs, rhs) {
                    (ConstValue::I32(a), ConstValue::I32(b)) => Some(ConstValue::I32(a.wrapping_add(*b))),
                    (ConstValue::I64(a), ConstValue::I64(b)) => Some(ConstValue::I64(a.wrapping_add(*b))),
                    (ConstValue::F64(a), ConstValue::F64(b)) => Some(ConstValue::F64(a + b)),
                    (ConstValue::String(a), ConstValue::String(b)) => {
                        let mut s = a.clone();
                        s.push_str(b);
                        Some(ConstValue::String(s))
                    }
                    _ => None,
                }
            }
            BinaryOp::Sub => {
                match (lhs, rhs) {
                    (ConstValue::I32(a), ConstValue::I32(b)) => Some(ConstValue::I32(a.wrapping_sub(*b))),
                    (ConstValue::I64(a), ConstValue::I64(b)) => Some(ConstValue::I64(a.wrapping_sub(*b))),
                    (ConstValue::F64(a), ConstValue::F64(b)) => Some(ConstValue::F64(a - b)),
                    _ => None,
                }
            }
            BinaryOp::Mul => {
                match (lhs, rhs) {
                    (ConstValue::I32(a), ConstValue::I32(b)) => Some(ConstValue::I32(a.wrapping_mul(*b))),
                    (ConstValue::I64(a), ConstValue::I64(b)) => Some(ConstValue::I64(a.wrapping_mul(*b))),
                    (ConstValue::F64(a), ConstValue::F64(b)) => Some(ConstValue::F64(a * b)),
                    _ => None,
                }
            }
            BinaryOp::Div => {
                match (lhs, rhs) {
                    (ConstValue::I32(a), ConstValue::I32(b)) => {
                        if *b != 0 { Some(ConstValue::I32(a / b)) } else { None }
                    }
                    (ConstValue::I64(a), ConstValue::I64(b)) => {
                        if *b != 0 { Some(ConstValue::I64(a / b)) } else { None }
                    }
                    (ConstValue::F64(a), ConstValue::F64(b)) => {
                        if *b != 0.0 { Some(ConstValue::F64(a / b)) } else { None }
                    }
                    _ => None,
                }
            }
            BinaryOp::Mod => {
                match (lhs, rhs) {
                    (ConstValue::I32(a), ConstValue::I32(b)) => {
                        if *b != 0 { Some(ConstValue::I32(a % b)) } else { None }
                    }
                    (ConstValue::I64(a), ConstValue::I64(b)) => {
                        if *b != 0 { Some(ConstValue::I64(a % b)) } else { None }
                    }
                    (ConstValue::F64(a), ConstValue::F64(b)) => {
                        if *b != 0.0 { Some(ConstValue::F64(a % b)) } else { None }
                    }
                    _ => None,
                }
            }
            BinaryOp::Eq => {
                match (lhs, rhs) {
                    (ConstValue::I32(a), ConstValue::I32(b)) => Some(ConstValue::Bool(a == b)),
                    (ConstValue::I64(a), ConstValue::I64(b)) => Some(ConstValue::Bool(a == b)),
                    (ConstValue::F64(a), ConstValue::F64(b)) => Some(ConstValue::Bool(a == b)),
                    (ConstValue::Bool(a), ConstValue::Bool(b)) => Some(ConstValue::Bool(a == b)),
                    (ConstValue::String(a), ConstValue::String(b)) => Some(ConstValue::Bool(a == b)),
                    _ => None,
                }
            }
            BinaryOp::Ne => {
                match (lhs, rhs) {
                    (ConstValue::I32(a), ConstValue::I32(b)) => Some(ConstValue::Bool(a != b)),
                    (ConstValue::I64(a), ConstValue::I64(b)) => Some(ConstValue::Bool(a != b)),
                    (ConstValue::F64(a), ConstValue::F64(b)) => Some(ConstValue::Bool(a != b)),
                    (ConstValue::Bool(a), ConstValue::Bool(b)) => Some(ConstValue::Bool(a != b)),
                    (ConstValue::String(a), ConstValue::String(b)) => Some(ConstValue::Bool(a != b)),
                    _ => None,
                }
            }
            BinaryOp::Lt => {
                match (lhs, rhs) {
                    (ConstValue::I32(a), ConstValue::I32(b)) => Some(ConstValue::Bool(a < b)),
                    (ConstValue::I64(a), ConstValue::I64(b)) => Some(ConstValue::Bool(a < b)),
                    (ConstValue::F64(a), ConstValue::F64(b)) => Some(ConstValue::Bool(a < b)),
                    _ => None,
                }
            }
            BinaryOp::Gt => {
                match (lhs, rhs) {
                    (ConstValue::I32(a), ConstValue::I32(b)) => Some(ConstValue::Bool(a > b)),
                    (ConstValue::I64(a), ConstValue::I64(b)) => Some(ConstValue::Bool(a > b)),
                    (ConstValue::F64(a), ConstValue::F64(b)) => Some(ConstValue::Bool(a > b)),
                    _ => None,
                }
            }
            BinaryOp::Le => {
                match (lhs, rhs) {
                    (ConstValue::I32(a), ConstValue::I32(b)) => Some(ConstValue::Bool(a <= b)),
                    (ConstValue::I64(a), ConstValue::I64(b)) => Some(ConstValue::Bool(a <= b)),
                    (ConstValue::F64(a), ConstValue::F64(b)) => Some(ConstValue::Bool(a <= b)),
                    _ => None,
                }
            }
            BinaryOp::Ge => {
                match (lhs, rhs) {
                    (ConstValue::I32(a), ConstValue::I32(b)) => Some(ConstValue::Bool(a >= b)),
                    (ConstValue::I64(a), ConstValue::I64(b)) => Some(ConstValue::Bool(a >= b)),
                    (ConstValue::F64(a), ConstValue::F64(b)) => Some(ConstValue::Bool(a >= b)),
                    _ => None,
                }
            }
            BinaryOp::And => {
                match (lhs, rhs) {
                    (ConstValue::Bool(a), ConstValue::Bool(b)) => Some(ConstValue::Bool(*a && *b)),
                    _ => None,
                }
            }
            BinaryOp::Or => {
                match (lhs, rhs) {
                    (ConstValue::Bool(a), ConstValue::Bool(b)) => Some(ConstValue::Bool(*a || *b)),
                    _ => None,
                }
            }
        }
    }
}

/// Unary operations in the IR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnaryOp {
    Neg,
    Not,
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOp::Neg => write!(f, "-"),
            UnaryOp::Not => write!(f, "!"),
        }
    }
}

impl UnaryOp {
    /// Evaluate the unary op on a constant value.
    pub fn eval(&self, val: &ConstValue) -> Option<ConstValue> {
        match self {
            UnaryOp::Neg => match val {
                ConstValue::I32(v) => Some(ConstValue::I32(-v)),
                ConstValue::I64(v) => Some(ConstValue::I64(-v)),
                ConstValue::F64(v) => Some(ConstValue::F64(-v)),
                _ => None,
            },
            UnaryOp::Not => match val {
                ConstValue::Bool(v) => Some(ConstValue::Bool(!v)),
                ConstValue::I32(v) => Some(ConstValue::I32(!v)),
                _ => None,
            },
        }
    }
}

/// IR Operations — the fundamental instructions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IrOp {
    /// Load a constant value. Result stored in virtual register `dest`.
    LoadConst { dest: usize, value: ConstValue },
    /// Load a variable by name into `dest`.
    LoadVar { dest: usize, name: String },
    /// Store the value in `src` into variable `name`.
    StoreVar { name: String, src: usize },
    /// Binary operation: `dest = lhs op rhs`.
    BinaryOp { dest: usize, op: BinaryOp, lhs: usize, rhs: usize },
    /// Unary operation: `dest = op operand`.
    UnaryOp { dest: usize, op: UnaryOp, operand: usize },
    /// Call a function by name with the given argument registers, result in `dest`.
    Call { dest: Option<usize>, func: String, args: Vec<usize> },
    /// Unconditional jump to label.
    Jump { target: String },
    /// Conditional branch: jump to `then_label` if `cond` is truthy, else `else_label`.
    Branch { cond: usize, then_label: String, else_label: String },
    /// Return a value from a function.
    Return { value: Option<usize> },
    /// No-op (placeholder / padding).
    Nop,
    /// Label (target for jumps).
    Label { name: String },
    /// Phi node: select value based on which predecessor block was executed.
    Phi { dest: usize, inputs: Vec<(String, usize)> },
}

impl fmt::Display for IrOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IrOp::LoadConst { dest, value } => write!(f, "  v{} = const {}", dest, value),
            IrOp::LoadVar { dest, name } => write!(f, "  v{} = load {}", dest, name),
            IrOp::StoreVar { name, src } => write!(f, "  store {} <- v{}", name, src),
            IrOp::BinaryOp { dest, op, lhs, rhs } => {
                write!(f, "  v{} = v{} {} v{}", dest, lhs, op, rhs)
            }
            IrOp::UnaryOp { dest, op, operand } => write!(f, "  v{} = {} v{}", dest, op, operand),
            IrOp::Call { dest, func, args } => {
                let args_str = args.iter().map(|a| format!("v{}", a)).collect::<Vec<_>>().join(", ");
                match dest {
                    Some(d) => write!(f, "  v{} = call {}({})", d, func, args_str),
                    None => write!(f, "  call {}({})", func, args_str),
                }
            }
            IrOp::Jump { target } => write!(f, "  jump {}", target),
            IrOp::Branch { cond, then_label, else_label } => {
                write!(f, "  br v{}, {}, {}", cond, then_label, else_label)
            }
            IrOp::Return { value: Some(v) } => write!(f, "  ret v{}", v),
            IrOp::Return { value: None } => write!(f, "  ret"),
            IrOp::Nop => write!(f, "  nop"),
            IrOp::Label { name } => write!(f, "{}:", name),
            IrOp::Phi { dest, inputs } => {
                let inputs_str = inputs
                    .iter()
                    .map(|(label, reg)| format!("[{}: v{}]", label, reg))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "  v{} = phi {}", dest, inputs_str)
            }
        }
    }
}

impl IrOp {
    /// Returns true if this op is a terminator (ends a basic block).
    pub fn is_terminator(&self) -> bool {
        matches!(self, IrOp::Jump { .. } | IrOp::Branch { .. } | IrOp::Return { .. })
    }

    /// Returns the destination register, if any.
    pub fn dest_reg(&self) -> Option<usize> {
        match self {
            IrOp::LoadConst { dest, .. } => Some(*dest),
            IrOp::LoadVar { dest, .. } => Some(*dest),
            IrOp::BinaryOp { dest, .. } => Some(*dest),
            IrOp::UnaryOp { dest, .. } => Some(*dest),
            IrOp::Call { dest: Some(d), .. } => Some(*d),
            IrOp::Phi { dest, .. } => Some(*dest),
            _ => None,
        }
    }

    /// Returns all register operands read by this op.
    pub fn src_regs(&self) -> Vec<usize> {
        match self {
            IrOp::StoreVar { src, .. } => vec![*src],
            IrOp::BinaryOp { lhs, rhs, .. } => vec![*lhs, *rhs],
            IrOp::UnaryOp { operand, .. } => vec![*operand],
            IrOp::Call { args, .. } => args.clone(),
            IrOp::Branch { cond, .. } => vec![*cond],
            IrOp::Return { value: Some(v) } => vec![*v],
            IrOp::Phi { inputs, .. } => inputs.iter().map(|(_, r)| *r).collect(),
            _ => Vec::new(),
        }
    }

    /// Returns all labels referenced by this op.
    pub fn labels(&self) -> Vec<&str> {
        match self {
            IrOp::Jump { target } => vec![target],
            IrOp::Branch { then_label, else_label, .. } => vec![then_label, else_label],
            IrOp::Phi { inputs, .. } => inputs.iter().map(|(l, _)| l.as_str()).collect(),
            _ => Vec::new(),
        }
    }
}

/// A function parameter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IrParam {
    pub name: String,
    pub ty: IrType,
}

/// An IR function.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IrFunction {
    pub name: String,
    pub params: Vec<IrParam>,
    pub ops: Vec<IrOp>,
    pub return_type: IrType,
}

impl IrFunction {
    /// Create a new empty IR function.
    pub fn new(name: &str, return_type: IrType) -> Self {
        Self {
            name: name.to_string(),
            params: Vec::new(),
            ops: Vec::new(),
            return_type,
        }
    }

    /// Create a new IR function with parameters.
    pub fn with_params(name: &str, params: Vec<IrParam>, return_type: IrType) -> Self {
        Self {
            name: name.to_string(),
            params,
            ops: Vec::new(),
            return_type,
        }
    }

    /// Add a parameter to this function.
    pub fn add_param(&mut self, name: &str, ty: IrType) {
        self.params.push(IrParam {
            name: name.to_string(),
            ty,
        });
    }

    /// Push an IR op to this function.
    pub fn push_op(&mut self, op: IrOp) {
        self.ops.push(op);
    }

    /// Count the number of ops (excluding labels).
    pub fn count_ops(&self) -> usize {
        self.ops.iter().filter(|op| !matches!(op, IrOp::Label { .. })).count()
    }

    /// Get all label names defined in this function.
    pub fn labels(&self) -> Vec<&str> {
        self.ops
            .iter()
            .filter_map(|op| match op {
                IrOp::Label { name } => Some(name.as_str()),
                _ => None,
            })
            .collect()
    }

    /// Check if this function has a label with the given name.
    pub fn has_label(&self, label: &str) -> bool {
        self.labels().contains(&label)
    }

    /// Get all variable names stored in this function.
    pub fn stored_vars(&self) -> Vec<&str> {
        self.ops
            .iter()
            .filter_map(|op| match op {
                IrOp::StoreVar { name, .. } => Some(name.as_str()),
                _ => None,
            })
            .collect()
    }

    /// Get all variable names loaded in this function.
    pub fn loaded_vars(&self) -> Vec<&str> {
        self.ops
            .iter()
            .filter_map(|op| match op {
                IrOp::LoadVar { name, .. } => Some(name.as_str()),
                _ => None,
            })
            .collect()
    }

    /// Get all function names called by this function.
    pub fn called_functions(&self) -> Vec<&str> {
        self.ops
            .iter()
            .filter_map(|op| match op {
                IrOp::Call { func, .. } => Some(func.as_str()),
                _ => None,
            })
            .collect()
    }

    /// Compute the next available virtual register number.
    pub fn next_reg(&self) -> usize {
        let mut max_reg = 0;
        for op in &self.ops {
            if let Some(dest) = op.dest_reg() {
                max_reg = max_reg.max(dest + 1);
            }
        }
        max_reg
    }
}

impl fmt::Display for IrFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let params = self
            .params
            .iter()
            .map(|p| format!("{}: {}", p.name, p.ty))
            .collect::<Vec<_>>()
            .join(", ");
        writeln!(f, "fn {}({}) -> {} {{", self.name, params, self.return_type)?;
        for op in &self.ops {
            writeln!(f, "{}", op)?;
        }
        write!(f, "}}")
    }
}

/// A global variable in the IR module.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IrGlobal {
    pub name: String,
    pub ty: IrType,
    pub initial_value: Option<ConstValue>,
}

/// An IR module — the top-level compilation unit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IrModule {
    pub functions: Vec<IrFunction>,
    pub globals: Vec<IrGlobal>,
    pub entry_point: String,
}

impl IrModule {
    /// Create a new empty IR module with the given entry point function name.
    pub fn new(entry_point: &str) -> Self {
        Self {
            functions: Vec::new(),
            globals: Vec::new(),
            entry_point: entry_point.to_string(),
        }
    }

    /// Add a function to the module.
    pub fn add_function(&mut self, func: IrFunction) {
        self.functions.push(func);
    }

    /// Add a global variable to the module.
    pub fn add_global(&mut self, name: &str, ty: IrType, initial_value: Option<ConstValue>) {
        self.globals.push(IrGlobal {
            name: name.to_string(),
            ty,
            initial_value,
        });
    }

    /// Get a function by name.
    pub fn get_function(&self, name: &str) -> Option<&IrFunction> {
        self.functions.iter().find(|f| f.name == name)
    }

    /// Get a mutable reference to a function by name.
    pub fn get_function_mut(&mut self, name: &str) -> Option<&mut IrFunction> {
        self.functions.iter_mut().find(|f| f.name == name)
    }

    /// Get the entry point function.
    pub fn entry_function(&self) -> Option<&IrFunction> {
        self.get_function(&self.entry_point)
    }

    /// Get all function names in this module.
    pub fn function_names(&self) -> Vec<&str> {
        self.functions.iter().map(|f| f.name.as_str()).collect()
    }

    /// Total number of IR ops across all functions.
    pub fn total_ops(&self) -> usize {
        self.functions.iter().map(|f| f.ops.len()).sum()
    }

    /// Validate the IR module (check labels, references, etc.).
    pub fn validate(&self) -> Result<(), IrValidationError> {
        // Check entry point exists
        if !self.functions.iter().any(|f| f.name == self.entry_point) {
            return Err(IrValidationError::MissingEntryPoint(self.entry_point.clone()));
        }

        for func in &self.functions {
            // Check all referenced labels exist
            for op in &func.ops {
                for label in op.labels() {
                    if !func.has_label(label) {
                        return Err(IrValidationError::UndefinedLabel {
                            function: func.name.clone(),
                            label: label.to_string(),
                        });
                    }
                }
            }

            // Check all called functions exist (or are builtins)
            for called in func.called_functions() {
                // Builtins are allowed
                if is_builtin(called) {
                    continue;
                }
                if !self.functions.iter().any(|f| f.name == called) {
                    return Err(IrValidationError::UndefinedFunction {
                        function: func.name.clone(),
                        callee: called.to_string(),
                    });
                }
            }
        }

        Ok(())
    }
}

/// Check if a function name is a builtin.
pub fn is_builtin(name: &str) -> bool {
    matches!(
        name,
        "print"
            | "println"
            | "sqrt"
            | "abs"
            | "floor"
            | "ceil"
            | "round"
            | "to_string"
            | "to_i32"
            | "to_f64"
            | "len"
            | "concat"
            | "substr"
            | "random"
            | "timer"
            | "key_pressed"
            | "mouse_x"
            | "mouse_y"
            | "mouse_down"
            | "touching"
            | "distance_to"
            | "ask"
            | "answer"
            | "move_forward"
            | "turn_right"
            | "turn_left"
            | "goto_xy"
            | "set_x"
            | "set_y"
            | "change_x"
            | "change_y"
            | "say"
            | "think"
            | "show"
            | "hide"
            | "switch_costume"
            | "next_costume"
            | "set_size"
            | "change_size"
            | "play_sound"
            | "stop_sounds"
            | "set_volume"
            | "change_volume"
            | "broadcast"
            | "wait"
            | "create_clone"
            | "pen_down"
            | "pen_up"
            | "pen_clear"
            | "pen_stamp"
            | "pen_set_color"
            | "pen_set_size"
    )
}

impl fmt::Display for IrModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "// IR Module (entry: {})", self.entry_point)?;
        for global in &self.globals {
            match &global.initial_value {
                Some(v) => writeln!(f, "global {}: {} = {}", global.name, global.ty, v)?,
                None => writeln!(f, "global {}: {}", global.name, global.ty)?,
            }
        }
        for func in &self.functions {
            writeln!(f)?;
            write!(f, "{}", func)?;
        }
        Ok(())
    }
}

/// Validation errors for IR modules.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum IrValidationError {
    #[error("entry point function '{0}' not found")]
    MissingEntryPoint(String),
    #[error("undefined label '{label}' in function '{function}'")]
    UndefinedLabel { function: String, label: String },
    #[error("undefined function '{callee}' called from '{function}'")]
    UndefinedFunction { function: String, callee: String },
}

/// A basic block extracted from an IR function.
#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub label: String,
    pub ops: Vec<IrOp>,
    pub terminator: Option<IrOp>,
}

impl BasicBlock {
    /// Create a new basic block with the given label.
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            ops: Vec::new(),
            terminator: None,
        }
    }

    /// Check if this block is terminated.
    pub fn is_terminated(&self) -> bool {
        self.terminator.is_some()
    }
}

/// Extract basic blocks from an IR function.
pub fn extract_basic_blocks(func: &IrFunction) -> Vec<BasicBlock> {
    let mut blocks: Vec<BasicBlock> = Vec::new();
    let mut current_block = BasicBlock::new(&format!("{}_entry", func.name));

    for op in &func.ops {
        match op {
            IrOp::Label { name } => {
                if !current_block.ops.is_empty() || current_block.is_terminated() {
                    blocks.push(current_block);
                }
                current_block = BasicBlock::new(name);
            }
            IrOp::Jump { .. } | IrOp::Branch { .. } | IrOp::Return { .. } => {
                current_block.terminator = Some(op.clone());
            }
            _ => {
                current_block.ops.push(op.clone());
            }
        }
    }

    if !current_block.ops.is_empty() || current_block.is_terminated() {
        blocks.push(current_block);
    }

    blocks
}

/// Build a call graph from an IR module (function name -> called function names).
pub fn build_call_graph(module: &IrModule) -> HashMap<String, Vec<String>> {
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();
    for func in &module.functions {
        let callees: Vec<String> = func
            .called_functions()
            .into_iter()
            .filter(|name| !is_builtin(name))
            .map(String::from)
            .collect();
        graph.insert(func.name.clone(), callees);
    }
    graph
}

/// Find all functions reachable from the entry point.
pub fn reachable_functions(module: &IrModule) -> Vec<String> {
    let call_graph = build_call_graph(module);
    let mut reachable = Vec::new();
    let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut stack = vec![module.entry_point.clone()];

    while let Some(name) = stack.pop() {
        if visited.contains(&name) {
            continue;
        }
        visited.insert(name.clone());
        reachable.push(name.clone());
        if let Some(callees) = call_graph.get(&name) {
            for callee in callees {
                if !visited.contains(callee) {
                    stack.push(callee.clone());
                }
            }
        }
    }

    reachable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ir_type_display() {
        assert_eq!(IrType::I32.to_string(), "i32");
        assert_eq!(IrType::F64.to_string(), "f64");
        assert_eq!(IrType::Bool.to_string(), "bool");
        assert_eq!(IrType::Void.to_string(), "void");
        assert_eq!(IrType::String.to_string(), "string");
        assert_eq!(IrType::Ptr.to_string(), "ptr");
    }

    #[test]
    fn test_ir_type_is_numeric() {
        assert!(IrType::I32.is_numeric());
        assert!(IrType::I64.is_numeric());
        assert!(IrType::F64.is_numeric());
        assert!(!IrType::Bool.is_numeric());
        assert!(!IrType::Void.is_numeric());
    }

    #[test]
    fn test_ir_type_default_value() {
        assert_eq!(IrType::I32.default_value(), ConstValue::I32(0));
        assert_eq!(IrType::F64.default_value(), ConstValue::F64(0.0));
        assert_eq!(IrType::Bool.default_value(), ConstValue::Bool(false));
        assert_eq!(IrType::String.default_value(), ConstValue::String(String::new()));
    }

    #[test]
    fn test_const_value_display() {
        assert_eq!(ConstValue::I32(42).to_string(), "42");
        assert_eq!(ConstValue::F64(3.14).to_string(), "3.140000");
        assert_eq!(ConstValue::Bool(true).to_string(), "true");
        assert_eq!(ConstValue::String("hello".into()).to_string(), "\"hello\"");
        assert_eq!(ConstValue::Null.to_string(), "null");
    }

    #[test]
    fn test_const_value_ir_type() {
        assert_eq!(ConstValue::I32(0).ir_type(), IrType::I32);
        assert_eq!(ConstValue::Bool(true).ir_type(), IrType::Bool);
        assert_eq!(ConstValue::String("".into()).ir_type(), IrType::String);
    }

    #[test]
    fn test_const_value_as_conversions() {
        assert_eq!(ConstValue::I32(42).as_i32(), Some(42));
        assert_eq!(ConstValue::F64(3.14).as_f64(), Some(3.14));
        assert_eq!(ConstValue::Bool(true).as_bool(), Some(true));
        assert_eq!(ConstValue::String("hi".into()).as_str(), Some("hi"));
        assert_eq!(ConstValue::I32(42).as_f64(), Some(42.0));
        assert_eq!(ConstValue::Bool(true).as_i32(), Some(1));
    }

    #[test]
    fn test_binary_op_display() {
        assert_eq!(BinaryOp::Add.to_string(), "+");
        assert_eq!(BinaryOp::Eq.to_string(), "==");
        assert_eq!(BinaryOp::And.to_string(), "&&");
    }

    #[test]
    fn test_binary_op_categories() {
        assert!(BinaryOp::Add.is_arithmetic());
        assert!(BinaryOp::Eq.is_comparison());
        assert!(BinaryOp::And.is_logical());
        assert!(!BinaryOp::Add.is_comparison());
    }

    #[test]
    fn test_binary_op_eval_i32() {
        let a = ConstValue::I32(10);
        let b = ConstValue::I32(3);
        assert_eq!(BinaryOp::Add.eval(&a, &b), Some(ConstValue::I32(13)));
        assert_eq!(BinaryOp::Sub.eval(&a, &b), Some(ConstValue::I32(7)));
        assert_eq!(BinaryOp::Mul.eval(&a, &b), Some(ConstValue::I32(30)));
        assert_eq!(BinaryOp::Div.eval(&a, &b), Some(ConstValue::I32(3)));
        assert_eq!(BinaryOp::Mod.eval(&a, &b), Some(ConstValue::I32(1)));
        assert_eq!(BinaryOp::Lt.eval(&a, &b), Some(ConstValue::Bool(false)));
        assert_eq!(BinaryOp::Gt.eval(&a, &b), Some(ConstValue::Bool(true)));
    }

    #[test]
    fn test_binary_op_eval_f64() {
        let a = ConstValue::F64(10.0);
        let b = ConstValue::F64(4.0);
        assert_eq!(BinaryOp::Add.eval(&a, &b), Some(ConstValue::F64(14.0)));
        assert_eq!(BinaryOp::Div.eval(&a, &b), Some(ConstValue::F64(2.5)));
    }

    #[test]
    fn test_binary_op_eval_string() {
        let a = ConstValue::String("hello".into());
        let b = ConstValue::String(" world".into());
        assert_eq!(BinaryOp::Add.eval(&a, &b), Some(ConstValue::String("hello world".into())));
        assert_eq!(BinaryOp::Eq.eval(&a, &a), Some(ConstValue::Bool(true)));
    }

    #[test]
    fn test_binary_op_div_by_zero() {
        let a = ConstValue::I32(10);
        let z = ConstValue::I32(0);
        assert_eq!(BinaryOp::Div.eval(&a, &z), None);
        assert_eq!(BinaryOp::Mod.eval(&a, &z), None);
    }

    #[test]
    fn test_unary_op_eval() {
        assert_eq!(UnaryOp::Neg.eval(&ConstValue::I32(5)), Some(ConstValue::I32(-5)));
        assert_eq!(UnaryOp::Neg.eval(&ConstValue::F64(3.14)), Some(ConstValue::F64(-3.14)));
        assert_eq!(UnaryOp::Not.eval(&ConstValue::Bool(true)), Some(ConstValue::Bool(false)));
    }

    #[test]
    fn test_ir_op_display() {
        let op = IrOp::LoadConst { dest: 0, value: ConstValue::I32(42) };
        assert_eq!(op.to_string(), "  v0 = const 42");

        let op = IrOp::BinaryOp { dest: 2, op: BinaryOp::Add, lhs: 0, rhs: 1 };
        assert_eq!(op.to_string(), "  v2 = v0 + v1");

        let op = IrOp::Return { value: Some(2) };
        assert_eq!(op.to_string(), "  ret v2");
    }

    #[test]
    fn test_ir_op_is_terminator() {
        assert!(IrOp::Jump { target: "end".into() }.is_terminator());
        assert!(IrOp::Return { value: None }.is_terminator());
        assert!(!IrOp::LoadConst { dest: 0, value: ConstValue::I32(1) }.is_terminator());
    }

    #[test]
    fn test_ir_op_dest_reg() {
        assert_eq!(IrOp::LoadConst { dest: 5, value: ConstValue::I32(1) }.dest_reg(), Some(5));
        assert_eq!(IrOp::StoreVar { name: "x".into(), src: 1 }.dest_reg(), None);
        assert_eq!(IrOp::Jump { target: "end".into() }.dest_reg(), None);
    }

    #[test]
    fn test_ir_op_src_regs() {
        let op = IrOp::BinaryOp { dest: 2, op: BinaryOp::Add, lhs: 0, rhs: 1 };
        assert_eq!(op.src_regs(), vec![0, 1]);

        let op = IrOp::StoreVar { name: "x".into(), src: 3 };
        assert_eq!(op.src_regs(), vec![3]);

        let op = IrOp::Nop;
        assert!(op.src_regs().is_empty());
    }

    #[test]
    fn test_ir_op_labels() {
        let op = IrOp::Jump { target: "loop".into() };
        assert_eq!(op.labels(), vec!["loop"]);

        let op = IrOp::Branch { cond: 0, then_label: "yes".into(), else_label: "no".into() };
        assert_eq!(op.labels(), vec!["yes", "no"]);
    }

    #[test]
    fn test_ir_function_construction() {
        let mut func = IrFunction::new("main", IrType::Void);
        func.add_param("argc", IrType::I32);
        func.push_op(IrOp::LoadConst { dest: 0, value: ConstValue::I32(0) });
        func.push_op(IrOp::Return { value: Some(0) });
        assert_eq!(func.name, "main");
        assert_eq!(func.params.len(), 1);
        assert_eq!(func.ops.len(), 2);
        assert_eq!(func.count_ops(), 2);
    }

    #[test]
    fn test_ir_function_next_reg() {
        let mut func = IrFunction::new("f", IrType::I32);
        assert_eq!(func.next_reg(), 0);
        func.push_op(IrOp::LoadConst { dest: 0, value: ConstValue::I32(1) });
        func.push_op(IrOp::LoadConst { dest: 1, value: ConstValue::I32(2) });
        assert_eq!(func.next_reg(), 2);
    }

    #[test]
    fn test_ir_function_called_functions() {
        let mut func = IrFunction::new("main", IrType::Void);
        func.push_op(IrOp::Call { dest: None, func: "print".into(), args: vec![0] });
        func.push_op(IrOp::Call { dest: Some(1), func: "helper".into(), args: vec![] });
        assert_eq!(func.called_functions(), vec!["print", "helper"]);
    }

    #[test]
    fn test_ir_function_stored_and_loaded_vars() {
        let mut func = IrFunction::new("f", IrType::Void);
        func.push_op(IrOp::LoadVar { dest: 0, name: "x".into() });
        func.push_op(IrOp::StoreVar { name: "y".into(), src: 0 });
        assert_eq!(func.loaded_vars(), vec!["x"]);
        assert_eq!(func.stored_vars(), vec!["y"]);
    }

    #[test]
    fn test_ir_function_display() {
        let mut func = IrFunction::new("add", IrType::I32);
        func.add_param("a", IrType::I32);
        func.add_param("b", IrType::I32);
        func.push_op(IrOp::LoadVar { dest: 0, name: "a".into() });
        func.push_op(IrOp::LoadVar { dest: 1, name: "b".into() });
        func.push_op(IrOp::BinaryOp { dest: 2, op: BinaryOp::Add, lhs: 0, rhs: 1 });
        func.push_op(IrOp::Return { value: Some(2) });
        let s = func.to_string();
        assert!(s.contains("fn add(a: i32, b: i32) -> i32"));
        assert!(s.contains("v2 = v0 + v1"));
    }

    #[test]
    fn test_ir_module_construction() {
        let mut module = IrModule::new("main");
        let func = IrFunction::new("main", IrType::Void);
        module.add_function(func);
        module.add_global("counter", IrType::I32, Some(ConstValue::I32(0)));
        assert_eq!(module.functions.len(), 1);
        assert_eq!(module.globals.len(), 1);
        assert!(module.get_function("main").is_some());
        assert!(module.get_function("nonexistent").is_none());
    }

    #[test]
    fn test_ir_module_validate_success() {
        let mut module = IrModule::new("main");
        let mut func = IrFunction::new("main", IrType::Void);
        func.push_op(IrOp::Return { value: None });
        module.add_function(func);
        assert!(module.validate().is_ok());
    }

    #[test]
    fn test_ir_module_validate_missing_entry() {
        let module = IrModule::new("main");
        assert!(matches!(module.validate(), Err(IrValidationError::MissingEntryPoint(_))));
    }

    #[test]
    fn test_ir_module_validate_undefined_label() {
        let mut module = IrModule::new("main");
        let mut func = IrFunction::new("main", IrType::Void);
        func.push_op(IrOp::Jump { target: "nonexistent".into() });
        module.add_function(func);
        assert!(matches!(module.validate(), Err(IrValidationError::UndefinedLabel { .. })));
    }

    #[test]
    fn test_ir_module_validate_undefined_function() {
        let mut module = IrModule::new("main");
        let mut func = IrFunction::new("main", IrType::Void);
        func.push_op(IrOp::Call { dest: None, func: "missing_func".into(), args: vec![] });
        func.push_op(IrOp::Return { value: None });
        module.add_function(func);
        assert!(matches!(module.validate(), Err(IrValidationError::UndefinedFunction { .. })));
    }

    #[test]
    fn test_ir_module_validate_builtin_allowed() {
        let mut module = IrModule::new("main");
        let mut func = IrFunction::new("main", IrType::Void);
        func.push_op(IrOp::Call { dest: None, func: "print".into(), args: vec![0] });
        func.push_op(IrOp::Return { value: None });
        module.add_function(func);
        assert!(module.validate().is_ok());
    }

    #[test]
    fn test_extract_basic_blocks() {
        let mut func = IrFunction::new("f", IrType::I32);
        func.push_op(IrOp::LoadConst { dest: 0, value: ConstValue::I32(1) });
        func.push_op(IrOp::Label { name: "loop".into() });
        func.push_op(IrOp::LoadConst { dest: 1, value: ConstValue::I32(2) });
        func.push_op(IrOp::Branch { cond: 1, then_label: "loop".into(), else_label: "end".into() });
        func.push_op(IrOp::Label { name: "end".into() });
        func.push_op(IrOp::Return { value: Some(0) });
        let blocks = extract_basic_blocks(&func);
        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[1].label, "loop");
        assert!(blocks[1].is_terminated());
    }

    #[test]
    fn test_build_call_graph() {
        let mut module = IrModule::new("main");
        let mut main_func = IrFunction::new("main", IrType::Void);
        main_func.push_op(IrOp::Call { dest: None, func: "helper".into(), args: vec![] });
        main_func.push_op(IrOp::Return { value: None });
        module.add_function(main_func);

        let helper = IrFunction::new("helper", IrType::Void);
        module.add_function(helper);

        let graph = build_call_graph(&module);
        assert_eq!(graph.get("main").unwrap().len(), 1);
        assert_eq!(graph.get("main").unwrap()[0], "helper");
        assert!(graph.get("helper").unwrap().is_empty());
    }

    #[test]
    fn test_reachable_functions() {
        let mut module = IrModule::new("main");
        let mut main_func = IrFunction::new("main", IrType::Void);
        main_func.push_op(IrOp::Call { dest: None, func: "helper".into(), args: vec![] });
        main_func.push_op(IrOp::Return { value: None });
        module.add_function(main_func);

        let helper = IrFunction::new("helper", IrType::Void);
        module.add_function(helper);

        let orphan = IrFunction::new("orphan", IrType::Void);
        module.add_function(orphan);

        let reachable = reachable_functions(&module);
        assert!(reachable.contains(&"main".to_string()));
        assert!(reachable.contains(&"helper".to_string()));
        assert!(!reachable.contains(&"orphan".to_string()));
    }

    #[test]
    fn test_ir_module_display() {
        let mut module = IrModule::new("main");
        module.add_global("x", IrType::I32, Some(ConstValue::I32(42)));
        let func = IrFunction::new("main", IrType::Void);
        module.add_function(func);
        let s = module.to_string();
        assert!(s.contains("// IR Module"));
        assert!(s.contains("global x: i32 = 42"));
    }

    #[test]
    fn test_ir_module_total_ops() {
        let mut module = IrModule::new("main");
        let mut f1 = IrFunction::new("main", IrType::Void);
        f1.push_op(IrOp::Nop);
        f1.push_op(IrOp::Return { value: None });
        module.add_function(f1);
        let mut f2 = IrFunction::new("helper", IrType::I32);
        f2.push_op(IrOp::LoadConst { dest: 0, value: ConstValue::I32(1) });
        f2.push_op(IrOp::Return { value: Some(0) });
        module.add_function(f2);
        assert_eq!(module.total_ops(), 4);
    }

    #[test]
    fn test_basic_block_new() {
        let bb = BasicBlock::new("entry");
        assert_eq!(bb.label, "entry");
        assert!(!bb.is_terminated());
        assert!(bb.ops.is_empty());
    }
}
