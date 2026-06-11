//! Lowering: Convert Sailfish project data structures to compiler IR.
//!
//! Transforms Scratch-like block operations into IR ops suitable
//! for optimization and code generation.

use crate::ir::{BinaryOp, ConstValue, IrFunction, IrModule, IrOp, IrType, UnaryOp};
use std::collections::HashMap;

/// Simplified project data that the lowerer works with.
/// This is decoupled from sf-vm to avoid circular dependencies.
#[derive(Debug, Clone)]
pub struct ProjectData {
    pub name: String,
    pub targets: Vec<TargetData>,
}

/// Simplified target data.
#[derive(Debug, Clone)]
pub struct TargetData {
    pub name: String,
    pub is_stage: bool,
    pub variables: HashMap<String, ConstValue>,
    pub blocks: Vec<BlockData>,
}

/// Simplified block data.
#[derive(Debug, Clone)]
pub struct BlockData {
    pub opcode: String,
    pub inputs: HashMap<String, InputValue>,
    pub fields: HashMap<String, String>,
    pub next: Option<usize>,
    /// For C-shaped blocks: body block index.
    pub substack: Option<usize>,
    /// For if/else: false-branch block index.
    pub substack2: Option<usize>,
}

/// Input values for blocks.
#[derive(Debug, Clone)]
pub enum InputValue {
    /// A literal constant value.
    Literal(ConstValue),
    /// A reference to a variable.
    Variable(String),
    /// A reference to another block (by index).
    Block(usize),
}

/// Errors during lowering.
#[derive(Debug, Clone, thiserror::Error)]
pub enum LowerError {
    #[error("unsupported opcode: {0}")]
    UnsupportedOpcode(String),
    #[error("missing input '{0}' for opcode '{1}'")]
    MissingInput(String, String),
    #[error("missing field '{0}' for opcode '{1}'")]
    MissingField(String, String),
    #[error("circular block reference at index {0}")]
    CircularReference(usize),
    #[error("lowering error: {0}")]
    General(String),
}

/// Result type for lowering.
pub type LowerResult<T> = Result<T, LowerError>;

/// Lower a project to an IR module.
pub fn lower_project(project: &ProjectData) -> LowerResult<IrModule> {
    let mut module = IrModule::new("main");

    // Add global variables from all targets
    for target in &project.targets {
        for (name, value) in &target.variables {
            let ty = value.ir_type();
            module.add_global(
                &format!("{}_{}", target.name, name),
                ty,
                Some(value.clone()),
            );
        }
    }

    // Lower each target into functions
    for target in &project.targets {
        let func_name = if target.is_stage {
            "stage_main".to_string()
        } else {
            format!("{}_main", target.name)
        };

        let mut func = IrFunction::new(&func_name, IrType::Void);
        let mut reg_counter = 0usize;

        // Lower all blocks in the target
        let mut visited = std::collections::HashSet::new();
        for (idx, _block) in target.blocks.iter().enumerate() {
            if visited.contains(&idx) {
                continue;
            }
            let mut stack = vec![idx];
            while let Some(block_idx) = stack.pop() {
                if visited.contains(&block_idx) || block_idx >= target.blocks.len() {
                    continue;
                }
                visited.insert(block_idx);

                let blk = &target.blocks[block_idx];
                lower_block(blk, &target, &mut func, &mut reg_counter, &mut stack)?;
            }
        }

        // Add return if not already terminated
        if !func.ops.iter().any(|op| op.is_terminator()) {
            func.push_op(IrOp::Return { value: None });
        }

        module.add_function(func);
    }

    // Create the main entry function that calls all target mains
    let mut main_func = IrFunction::new("main", IrType::Void);
    for target in &project.targets {
        let func_name = if target.is_stage {
            "stage_main".to_string()
        } else {
            format!("{}_main", target.name)
        };
        main_func.push_op(IrOp::Call {
            dest: None,
            func: func_name,
            args: vec![],
        });
    }
    main_func.push_op(IrOp::Return { value: None });
    module.add_function(main_func);

    Ok(module)
}

/// Lower a single block to IR ops.
fn lower_block(
    block: &BlockData,
    _target: &TargetData,
    func: &mut IrFunction,
    reg: &mut usize,
    stack: &mut Vec<usize>,
) -> LowerResult<()> {
    let next_reg = |r: &mut usize| {
        let v = *r;
        *r += 1;
        v
    };

    match block.opcode.as_str() {
        // Motion ops
        "motion_forward" | "motion_movesteps" => {
            let steps_reg = lower_input(&block.inputs, "STEPS", reg, func)?;
            let dest = next_reg(reg);
            func.push_op(IrOp::Call {
                dest: Some(dest),
                func: "move_forward".to_string(),
                args: vec![steps_reg],
            });
        }
        "motion_turnright" => {
            let deg_reg = lower_input(&block.inputs, "DEGREES", reg, func)?;
            let dest = next_reg(reg);
            func.push_op(IrOp::Call {
                dest: Some(dest),
                func: "turn_right".to_string(),
                args: vec![deg_reg],
            });
        }
        "motion_turnleft" => {
            let deg_reg = lower_input(&block.inputs, "DEGREES", reg, func)?;
            let dest = next_reg(reg);
            func.push_op(IrOp::Call {
                dest: Some(dest),
                func: "turn_left".to_string(),
                args: vec![deg_reg],
            });
        }
        "motion_gotoxy" => {
            let x_reg = lower_input(&block.inputs, "X", reg, func)?;
            let y_reg = lower_input(&block.inputs, "Y", reg, func)?;
            let dest = next_reg(reg);
            func.push_op(IrOp::Call {
                dest: Some(dest),
                func: "goto_xy".to_string(),
                args: vec![x_reg, y_reg],
            });
        }
        "motion_setx" => {
            let x_reg = lower_input(&block.inputs, "X", reg, func)?;
            let dest = next_reg(reg);
            func.push_op(IrOp::Call {
                dest: Some(dest),
                func: "set_x".to_string(),
                args: vec![x_reg],
            });
        }
        "motion_sety" => {
            let y_reg = lower_input(&block.inputs, "Y", reg, func)?;
            let dest = next_reg(reg);
            func.push_op(IrOp::Call {
                dest: Some(dest),
                func: "set_y".to_string(),
                args: vec![y_reg],
            });
        }
        "motion_changexby" => {
            let dx_reg = lower_input(&block.inputs, "DX", reg, func)?;
            let dest = next_reg(reg);
            func.push_op(IrOp::Call {
                dest: Some(dest),
                func: "change_x".to_string(),
                args: vec![dx_reg],
            });
        }
        "motion_changeyby" => {
            let dy_reg = lower_input(&block.inputs, "DY", reg, func)?;
            let dest = next_reg(reg);
            func.push_op(IrOp::Call {
                dest: Some(dest),
                func: "change_y".to_string(),
                args: vec![dy_reg],
            });
        }

        // Looks ops
        "looks_say" => {
            let msg_reg = lower_input(&block.inputs, "MESSAGE", reg, func)?;
            func.push_op(IrOp::Call {
                dest: None,
                func: "say".to_string(),
                args: vec![msg_reg],
            });
        }
        "looks_think" => {
            let msg_reg = lower_input(&block.inputs, "MESSAGE", reg, func)?;
            func.push_op(IrOp::Call {
                dest: None,
                func: "think".to_string(),
                args: vec![msg_reg],
            });
        }
        "looks_show" => {
            func.push_op(IrOp::Call {
                dest: None,
                func: "show".to_string(),
                args: vec![],
            });
        }
        "looks_hide" => {
            func.push_op(IrOp::Call {
                dest: None,
                func: "hide".to_string(),
                args: vec![],
            });
        }

        // Sound ops
        "sound_playuntildone" | "sound_play" => {
            let snd_reg = lower_input(&block.inputs, "SOUND_MENU", reg, func)?;
            func.push_op(IrOp::Call {
                dest: None,
                func: "play_sound".to_string(),
                args: vec![snd_reg],
            });
        }
        "sound_stopallsounds" => {
            func.push_op(IrOp::Call {
                dest: None,
                func: "stop_sounds".to_string(),
                args: vec![],
            });
        }
        "sound_setvolumeto" => {
            let vol_reg = lower_input(&block.inputs, "VOLUME", reg, func)?;
            func.push_op(IrOp::Call {
                dest: None,
                func: "set_volume".to_string(),
                args: vec![vol_reg],
            });
        }

        // Event ops
        "event_whenflagclicked" | "event_whenstageclicked" => {
            // Hat block — just a marker, no IR op needed
        }
        "event_broadcast" => {
            let msg_reg = lower_input(&block.inputs, "BROADCAST_INPUT", reg, func)?;
            func.push_op(IrOp::Call {
                dest: None,
                func: "broadcast".to_string(),
                args: vec![msg_reg],
            });
        }

        // Control ops
        "control_wait" => {
            let dur_reg = lower_input(&block.inputs, "DURATION", reg, func)?;
            func.push_op(IrOp::Call {
                dest: None,
                func: "wait".to_string(),
                args: vec![dur_reg],
            });
        }
        "control_repeat" => {
            let times_reg = lower_input(&block.inputs, "TIMES", reg, func)?;
            let loop_label = format!("loop_{}", uuid::Uuid::new_v4().as_simple());
            let end_label = format!("end_{}", uuid::Uuid::new_v4().as_simple());

            // Counter variable
            let counter_reg = *reg;
            *reg += 1;
            func.push_op(IrOp::LoadConst { dest: counter_reg, value: ConstValue::I32(0) });

            func.push_op(IrOp::Label { name: loop_label.clone() });
            // Compare counter < times
            let cond_reg = *reg;
            *reg += 1;
            func.push_op(IrOp::BinaryOp {
                dest: cond_reg,
                op: BinaryOp::Lt,
                lhs: counter_reg,
                rhs: times_reg,
            });
            func.push_op(IrOp::Branch {
                cond: cond_reg,
                then_label: loop_label.clone(),
                else_label: end_label.clone(),
            });

            // Lower body
            if let Some(body_idx) = block.substack {
                stack.push(body_idx);
            }

            // Increment counter
            let one_reg = *reg;
            *reg += 1;
            func.push_op(IrOp::LoadConst { dest: one_reg, value: ConstValue::I32(1) });
            let new_counter = *reg;
            *reg += 1;
            func.push_op(IrOp::BinaryOp {
                dest: new_counter,
                op: BinaryOp::Add,
                lhs: counter_reg,
                rhs: one_reg,
            });

            func.push_op(IrOp::Jump { target: loop_label });
            func.push_op(IrOp::Label { name: end_label });
        }
        "control_forever" => {
            let loop_label = format!("forever_{}", uuid::Uuid::new_v4().as_simple());
            func.push_op(IrOp::Label { name: loop_label.clone() });

            if let Some(body_idx) = block.substack {
                stack.push(body_idx);
            }

            func.push_op(IrOp::Jump { target: loop_label });
        }
        "control_if" => {
            let cond_reg = lower_input(&block.inputs, "CONDITION", reg, func)?;
            let then_label = format!("then_{}", uuid::Uuid::new_v4().as_simple());
            let end_label = format!("endif_{}", uuid::Uuid::new_v4().as_simple());

            func.push_op(IrOp::Branch {
                cond: cond_reg,
                then_label: then_label.clone(),
                else_label: end_label.clone(),
            });
            func.push_op(IrOp::Label { name: then_label });

            if let Some(body_idx) = block.substack {
                stack.push(body_idx);
            }

            func.push_op(IrOp::Label { name: end_label });
        }
        "control_if_else" => {
            let cond_reg = lower_input(&block.inputs, "CONDITION", reg, func)?;
            let then_label = format!("then_{}", uuid::Uuid::new_v4().as_simple());
            let else_label = format!("else_{}", uuid::Uuid::new_v4().as_simple());
            let end_label = format!("endif_{}", uuid::Uuid::new_v4().as_simple());

            func.push_op(IrOp::Branch {
                cond: cond_reg,
                then_label: then_label.clone(),
                else_label: else_label.clone(),
            });
            func.push_op(IrOp::Label { name: then_label });

            if let Some(body_idx) = block.substack {
                stack.push(body_idx);
            }

            func.push_op(IrOp::Jump { target: end_label.clone() });
            func.push_op(IrOp::Label { name: else_label });

            if let Some(else_idx) = block.substack2 {
                stack.push(else_idx);
            }

            func.push_op(IrOp::Label { name: end_label });
        }
        "control_stop" => {
            func.push_op(IrOp::Return { value: None });
        }
        "control_create_clone_of" => {
            let clone_reg = lower_input(&block.inputs, "CLONE_OPTION", reg, func)?;
            func.push_op(IrOp::Call {
                dest: None,
                func: "create_clone".to_string(),
                args: vec![clone_reg],
            });
        }

        // Sensing ops
        "sensing_timer" => {
            let dest = *reg;
            *reg += 1;
            func.push_op(IrOp::Call {
                dest: Some(dest),
                func: "timer".to_string(),
                args: vec![],
            });
        }
        "sensing_keypressed" => {
            let key_reg = lower_input(&block.inputs, "KEY_OPTION", reg, func)?;
            let dest = *reg;
            *reg += 1;
            func.push_op(IrOp::Call {
                dest: Some(dest),
                func: "key_pressed".to_string(),
                args: vec![key_reg],
            });
        }
        "sensing_mousedown" => {
            let dest = *reg;
            *reg += 1;
            func.push_op(IrOp::Call {
                dest: Some(dest),
                func: "mouse_down".to_string(),
                args: vec![],
            });
        }
        "sensing_mousex" => {
            let dest = *reg;
            *reg += 1;
            func.push_op(IrOp::Call {
                dest: Some(dest),
                func: "mouse_x".to_string(),
                args: vec![],
            });
        }
        "sensing_mousey" => {
            let dest = *reg;
            *reg += 1;
            func.push_op(IrOp::Call {
                dest: Some(dest),
                func: "mouse_y".to_string(),
                args: vec![],
            });
        }

        // Operator ops
        "operator_add" => {
            let lhs = lower_input(&block.inputs, "NUM1", reg, func)?;
            let rhs = lower_input(&block.inputs, "NUM2", reg, func)?;
            let dest = *reg;
            *reg += 1;
            func.push_op(IrOp::BinaryOp { dest, op: BinaryOp::Add, lhs, rhs });
        }
        "operator_subtract" => {
            let lhs = lower_input(&block.inputs, "NUM1", reg, func)?;
            let rhs = lower_input(&block.inputs, "NUM2", reg, func)?;
            let dest = *reg;
            *reg += 1;
            func.push_op(IrOp::BinaryOp { dest, op: BinaryOp::Sub, lhs, rhs });
        }
        "operator_multiply" => {
            let lhs = lower_input(&block.inputs, "NUM1", reg, func)?;
            let rhs = lower_input(&block.inputs, "NUM2", reg, func)?;
            let dest = *reg;
            *reg += 1;
            func.push_op(IrOp::BinaryOp { dest, op: BinaryOp::Mul, lhs, rhs });
        }
        "operator_divide" => {
            let lhs = lower_input(&block.inputs, "NUM1", reg, func)?;
            let rhs = lower_input(&block.inputs, "NUM2", reg, func)?;
            let dest = *reg;
            *reg += 1;
            func.push_op(IrOp::BinaryOp { dest, op: BinaryOp::Div, lhs, rhs });
        }
        "operator_mod" => {
            let lhs = lower_input(&block.inputs, "NUM1", reg, func)?;
            let rhs = lower_input(&block.inputs, "NUM2", reg, func)?;
            let dest = *reg;
            *reg += 1;
            func.push_op(IrOp::BinaryOp { dest, op: BinaryOp::Mod, lhs, rhs });
        }
        "operator_gt" => {
            let lhs = lower_input(&block.inputs, "OPERAND1", reg, func)?;
            let rhs = lower_input(&block.inputs, "OPERAND2", reg, func)?;
            let dest = *reg;
            *reg += 1;
            func.push_op(IrOp::BinaryOp { dest, op: BinaryOp::Gt, lhs, rhs });
        }
        "operator_lt" => {
            let lhs = lower_input(&block.inputs, "OPERAND1", reg, func)?;
            let rhs = lower_input(&block.inputs, "OPERAND2", reg, func)?;
            let dest = *reg;
            *reg += 1;
            func.push_op(IrOp::BinaryOp { dest, op: BinaryOp::Lt, lhs, rhs });
        }
        "operator_equals" => {
            let lhs = lower_input(&block.inputs, "OPERAND1", reg, func)?;
            let rhs = lower_input(&block.inputs, "OPERAND2", reg, func)?;
            let dest = *reg;
            *reg += 1;
            func.push_op(IrOp::BinaryOp { dest, op: BinaryOp::Eq, lhs, rhs });
        }
        "operator_and" => {
            let lhs = lower_input(&block.inputs, "OPERAND1", reg, func)?;
            let rhs = lower_input(&block.inputs, "OPERAND2", reg, func)?;
            let dest = *reg;
            *reg += 1;
            func.push_op(IrOp::BinaryOp { dest, op: BinaryOp::And, lhs, rhs });
        }
        "operator_or" => {
            let lhs = lower_input(&block.inputs, "OPERAND1", reg, func)?;
            let rhs = lower_input(&block.inputs, "OPERAND2", reg, func)?;
            let dest = *reg;
            *reg += 1;
            func.push_op(IrOp::BinaryOp { dest, op: BinaryOp::Or, lhs, rhs });
        }
        "operator_not" => {
            let operand = lower_input(&block.inputs, "OPERAND", reg, func)?;
            let dest = *reg;
            *reg += 1;
            func.push_op(IrOp::UnaryOp { dest, op: UnaryOp::Not, operand });
        }
        "operator_join" => {
            let lhs = lower_input(&block.inputs, "STRING1", reg, func)?;
            let rhs = lower_input(&block.inputs, "STRING2", reg, func)?;
            let dest = *reg;
            *reg += 1;
            func.push_op(IrOp::Call {
                dest: Some(dest),
                func: "concat".to_string(),
                args: vec![lhs, rhs],
            });
        }
        "operator_random" => {
            let from_reg = lower_input(&block.inputs, "FROM", reg, func)?;
            let to_reg = lower_input(&block.inputs, "TO", reg, func)?;
            let dest = *reg;
            *reg += 1;
            func.push_op(IrOp::Call {
                dest: Some(dest),
                func: "random".to_string(),
                args: vec![from_reg, to_reg],
            });
        }
        "operator_round" => {
            let num_reg = lower_input(&block.inputs, "NUM", reg, func)?;
            let dest = *reg;
            *reg += 1;
            func.push_op(IrOp::Call {
                dest: Some(dest),
                func: "round".to_string(),
                args: vec![num_reg],
            });
        }
        "operator_sqrt" => {
            let num_reg = lower_input(&block.inputs, "NUM", reg, func)?;
            let dest = *reg;
            *reg += 1;
            func.push_op(IrOp::Call {
                dest: Some(dest),
                func: "sqrt".to_string(),
                args: vec![num_reg],
            });
        }
        "operator_abs" => {
            let num_reg = lower_input(&block.inputs, "NUM", reg, func)?;
            let dest = *reg;
            *reg += 1;
            func.push_op(IrOp::Call {
                dest: Some(dest),
                func: "abs".to_string(),
                args: vec![num_reg],
            });
        }
        "operator_length" => {
            let str_reg = lower_input(&block.inputs, "STRING", reg, func)?;
            let dest = *reg;
            *reg += 1;
            func.push_op(IrOp::Call {
                dest: Some(dest),
                func: "len".to_string(),
                args: vec![str_reg],
            });
        }

        // Variable ops
        "data_setvariableto" => {
            let var_name = block.fields.get("VARIABLE").cloned().unwrap_or_default();
            let val_reg = lower_input(&block.inputs, "VALUE", reg, func)?;
            func.push_op(IrOp::StoreVar { name: var_name, src: val_reg });
        }
        "data_changevariableby" => {
            let var_name = block.fields.get("VARIABLE").cloned().unwrap_or_default();
            let delta_reg = lower_input(&block.inputs, "VALUE", reg, func)?;
            let current_reg = *reg;
            *reg += 1;
            func.push_op(IrOp::LoadVar { dest: current_reg, name: var_name.clone() });
            let new_val = *reg;
            *reg += 1;
            func.push_op(IrOp::BinaryOp {
                dest: new_val,
                op: BinaryOp::Add,
                lhs: current_reg,
                rhs: delta_reg,
            });
            func.push_op(IrOp::StoreVar { name: var_name, src: new_val });
        }
        "data_variable" => {
            let var_name = block.fields.get("VARIABLE").cloned().unwrap_or_default();
            let dest = *reg;
            *reg += 1;
            func.push_op(IrOp::LoadVar { dest, name: var_name });
        }
        "data_addtolist" => {
            let list_name = block.fields.get("LIST").cloned().unwrap_or_default();
            let item_reg = lower_input(&block.inputs, "ITEM", reg, func)?;
            func.push_op(IrOp::Call {
                dest: None,
                func: format!("list_add_{}", list_name),
                args: vec![item_reg],
            });
        }

        // Pen ops
        "pen_clear" => {
            func.push_op(IrOp::Call {
                dest: None,
                func: "pen_clear".to_string(),
                args: vec![],
            });
        }
        "pen_penDown" => {
            func.push_op(IrOp::Call {
                dest: None,
                func: "pen_down".to_string(),
                args: vec![],
            });
        }
        "pen_penUp" => {
            func.push_op(IrOp::Call {
                dest: None,
                func: "pen_up".to_string(),
                args: vec![],
            });
        }
        "pen_stamp" => {
            func.push_op(IrOp::Call {
                dest: None,
                func: "pen_stamp".to_string(),
                args: vec![],
            });
        }
        "pen_setPenColorToColor" => {
            let color_reg = lower_input(&block.inputs, "COLOR", reg, func)?;
            func.push_op(IrOp::Call {
                dest: None,
                func: "pen_set_color".to_string(),
                args: vec![color_reg],
            });
        }
        "pen_setPenSizeTo" => {
            let size_reg = lower_input(&block.inputs, "SIZE", reg, func)?;
            func.push_op(IrOp::Call {
                dest: None,
                func: "pen_set_size".to_string(),
                args: vec![size_reg],
            });
        }

        _ => {
            // Unknown opcodes become no-ops with a warning
            func.push_op(IrOp::Nop);
        }
    }

    Ok(())
}

/// Lower an input value to a register.
fn lower_input(
    inputs: &HashMap<String, InputValue>,
    name: &str,
    reg: &mut usize,
    func: &mut IrFunction,
) -> LowerResult<usize> {
    let next_reg = |r: &mut usize| {
        let v = *r;
        *r += 1;
        v
    };

    match inputs.get(name) {
        Some(InputValue::Literal(val)) => {
            let dest = next_reg(reg);
            func.push_op(IrOp::LoadConst { dest, value: val.clone() });
            Ok(dest)
        }
        Some(InputValue::Variable(var_name)) => {
            let dest = next_reg(reg);
            func.push_op(IrOp::LoadVar { dest, name: var_name.clone() });
            Ok(dest)
        }
        Some(InputValue::Block(_idx)) => {
            // For reporter blocks, we'd recursively lower them.
            // For now, load a placeholder.
            let dest = next_reg(reg);
            func.push_op(IrOp::LoadConst { dest, value: ConstValue::I32(0) });
            Ok(dest)
        }
        None => {
            // Default to 0 for missing inputs
            let dest = next_reg(reg);
            func.push_op(IrOp::LoadConst { dest, value: ConstValue::I32(0) });
            Ok(dest)
        }
    }
}

/// Create a simple project data for testing.
pub fn make_simple_project(name: &str) -> ProjectData {
    ProjectData {
        name: name.to_string(),
        targets: vec![TargetData {
            name: "Stage".to_string(),
            is_stage: true,
            variables: HashMap::new(),
            blocks: vec![BlockData {
                opcode: "event_whenflagclicked".to_string(),
                inputs: HashMap::new(),
                fields: HashMap::new(),
                next: None,
                substack: None,
                substack2: None,
            }],
        }],
    }
}

/// Create a project with a variable set and motion.
pub fn make_motion_project() -> ProjectData {
    let mut vars = HashMap::new();
    vars.insert("speed".to_string(), ConstValue::F64(10.0));

    ProjectData {
        name: "motion_test".to_string(),
        targets: vec![TargetData {
            name: "Sprite1".to_string(),
            is_stage: false,
            variables: vars,
            blocks: vec![
                BlockData {
                    opcode: "event_whenflagclicked".to_string(),
                    inputs: HashMap::new(),
                    fields: HashMap::new(),
                    next: None,
                    substack: None,
                    substack2: None,
                },
                BlockData {
                    opcode: "motion_forward".to_string(),
                    inputs: {
                        let mut m = HashMap::new();
                        m.insert("STEPS".to_string(), InputValue::Literal(ConstValue::F64(10.0)));
                        m
                    },
                    fields: HashMap::new(),
                    next: None,
                    substack: None,
                    substack2: None,
                },
            ],
        }],
    }
}

/// Create a project with if/else control flow.
pub fn make_if_else_project() -> ProjectData {
    ProjectData {
        name: "if_else_test".to_string(),
        targets: vec![TargetData {
            name: "Sprite1".to_string(),
            is_stage: false,
            variables: HashMap::new(),
            blocks: vec![
                BlockData {
                    opcode: "control_if_else".to_string(),
                    inputs: {
                        let mut m = HashMap::new();
                        m.insert("CONDITION".to_string(), InputValue::Literal(ConstValue::Bool(true)));
                        m
                    },
                    fields: HashMap::new(),
                    next: None,
                    substack: None,
                    substack2: None,
                },
            ],
        }],
    }
}

/// Create a project with arithmetic operators.
pub fn make_arithmetic_project() -> ProjectData {
    ProjectData {
        name: "arith_test".to_string(),
        targets: vec![TargetData {
            name: "Stage".to_string(),
            is_stage: true,
            variables: HashMap::new(),
            blocks: vec![
                BlockData {
                    opcode: "operator_add".to_string(),
                    inputs: {
                        let mut m = HashMap::new();
                        m.insert("NUM1".to_string(), InputValue::Literal(ConstValue::I32(5)));
                        m.insert("NUM2".to_string(), InputValue::Literal(ConstValue::I32(3)));
                        m
                    },
                    fields: HashMap::new(),
                    next: None,
                    substack: None,
                    substack2: None,
                },
            ],
        }],
    }
}

/// Create a project with variables.
pub fn make_variable_project() -> ProjectData {
    let mut vars = HashMap::new();
    vars.insert("x".to_string(), ConstValue::I32(0));

    ProjectData {
        name: "var_test".to_string(),
        targets: vec![TargetData {
            name: "Stage".to_string(),
            is_stage: true,
            variables: vars,
            blocks: vec![
                BlockData {
                    opcode: "data_setvariableto".to_string(),
                    inputs: {
                        let mut m = HashMap::new();
                        m.insert("VALUE".to_string(), InputValue::Literal(ConstValue::I32(42)));
                        m
                    },
                    fields: {
                        let mut m = HashMap::new();
                        m.insert("VARIABLE".to_string(), "x".to_string());
                        m
                    },
                    next: None,
                    substack: None,
                    substack2: None,
                },
                BlockData {
                    opcode: "data_changevariableby".to_string(),
                    inputs: {
                        let mut m = HashMap::new();
                        m.insert("VALUE".to_string(), InputValue::Literal(ConstValue::I32(1)));
                        m
                    },
                    fields: {
                        let mut m = HashMap::new();
                        m.insert("VARIABLE".to_string(), "x".to_string());
                        m
                    },
                    next: None,
                    substack: None,
                    substack2: None,
                },
            ],
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lower_simple_project() {
        let project = make_simple_project("test");
        let module = lower_project(&project).unwrap();
        assert!(module.get_function("main").is_some());
        assert!(module.get_function("stage_main").is_some());
        assert!(module.validate().is_ok());
    }

    #[test]
    fn test_lower_motion_project() {
        let project = make_motion_project();
        let module = lower_project(&project).unwrap();
        let func = module.get_function("Sprite1_main").unwrap();
        // Should contain a call to move_forward
        assert!(func.ops.iter().any(|op| matches!(op, IrOp::Call { func, .. } if func == "move_forward")));
    }

    #[test]
    fn test_lower_if_else() {
        let project = make_if_else_project();
        let module = lower_project(&project).unwrap();
        let func = module.get_function("Sprite1_main").unwrap();
        // Should contain a branch
        assert!(func.ops.iter().any(|op| matches!(op, IrOp::Branch { .. })));
    }

    #[test]
    fn test_lower_arithmetic() {
        let project = make_arithmetic_project();
        let module = lower_project(&project).unwrap();
        let func = module.get_function("stage_main").unwrap();
        // Should contain a binary add op
        assert!(func.ops.iter().any(|op| matches!(op, IrOp::BinaryOp { op: BinaryOp::Add, .. })));
    }

    #[test]
    fn test_lower_variables() {
        let project = make_variable_project();
        let module = lower_project(&project).unwrap();
        // Should have the variable as a global
        assert!(module.globals.iter().any(|g| g.name.contains("x")));
        let func = module.get_function("stage_main").unwrap();
        // Should contain store and load ops
        assert!(func.ops.iter().any(|op| matches!(op, IrOp::StoreVar { .. })));
        assert!(func.ops.iter().any(|op| matches!(op, IrOp::LoadVar { .. })));
    }

    #[test]
    fn test_lower_project_has_entry() {
        let project = make_simple_project("test");
        let module = lower_project(&project).unwrap();
        assert_eq!(module.entry_point, "main");
        assert!(module.entry_function().is_some());
    }

    #[test]
    fn test_lower_main_calls_targets() {
        let project = make_simple_project("test");
        let module = lower_project(&project).unwrap();
        let main_func = module.get_function("main").unwrap();
        // Main should call stage_main
        assert!(main_func.called_functions().contains(&"stage_main"));
    }

    #[test]
    fn test_lower_repeat_generates_loop() {
        let project = ProjectData {
            name: "repeat_test".to_string(),
            targets: vec![TargetData {
                name: "Sprite1".to_string(),
                is_stage: false,
                variables: HashMap::new(),
                blocks: vec![BlockData {
                    opcode: "control_repeat".to_string(),
                    inputs: {
                        let mut m = HashMap::new();
                        m.insert("TIMES".to_string(), InputValue::Literal(ConstValue::I32(10)));
                        m
                    },
                    fields: HashMap::new(),
                    next: None,
                    substack: None,
                    substack2: None,
                }],
            }],
        };
        let module = lower_project(&project).unwrap();
        let func = module.get_function("Sprite1_main").unwrap();
        // Should contain a label and a branch (loop structure)
        assert!(func.ops.iter().any(|op| matches!(op, IrOp::Label { .. })));
        assert!(func.ops.iter().any(|op| matches!(op, IrOp::Branch { .. })));
        assert!(func.ops.iter().any(|op| matches!(op, IrOp::Jump { .. })));
    }

    #[test]
    fn test_lower_forever_generates_loop() {
        let project = ProjectData {
            name: "forever_test".to_string(),
            targets: vec![TargetData {
                name: "Sprite1".to_string(),
                is_stage: false,
                variables: HashMap::new(),
                blocks: vec![BlockData {
                    opcode: "control_forever".to_string(),
                    inputs: HashMap::new(),
                    fields: HashMap::new(),
                    next: None,
                    substack: None,
                    substack2: None,
                }],
            }],
        };
        let module = lower_project(&project).unwrap();
        let func = module.get_function("Sprite1_main").unwrap();
        assert!(func.ops.iter().any(|op| matches!(op, IrOp::Label { .. })));
        assert!(func.ops.iter().any(|op| matches!(op, IrOp::Jump { .. })));
    }

    #[test]
    fn test_lower_say() {
        let project = ProjectData {
            name: "say_test".to_string(),
            targets: vec![TargetData {
                name: "Sprite1".to_string(),
                is_stage: false,
                variables: HashMap::new(),
                blocks: vec![BlockData {
                    opcode: "looks_say".to_string(),
                    inputs: {
                        let mut m = HashMap::new();
                        m.insert("MESSAGE".to_string(), InputValue::Literal(ConstValue::String("Hello!".into())));
                        m
                    },
                    fields: HashMap::new(),
                    next: None,
                    substack: None,
                    substack2: None,
                }],
            }],
        };
        let module = lower_project(&project).unwrap();
        let func = module.get_function("Sprite1_main").unwrap();
        assert!(func.ops.iter().any(|op| matches!(op, IrOp::Call { func, .. } if func == "say")));
    }

    #[test]
    fn test_lower_unknown_opcode() {
        let project = ProjectData {
            name: "unknown_test".to_string(),
            targets: vec![TargetData {
                name: "Stage".to_string(),
                is_stage: true,
                variables: HashMap::new(),
                blocks: vec![BlockData {
                    opcode: "custom_unknown_opcode".to_string(),
                    inputs: HashMap::new(),
                    fields: HashMap::new(),
                    next: None,
                    substack: None,
                    substack2: None,
                }],
            }],
        };
        let module = lower_project(&project).unwrap();
        let func = module.get_function("stage_main").unwrap();
        assert!(func.ops.iter().any(|op| matches!(op, IrOp::Nop)));
    }

    #[test]
    fn test_lower_pen_ops() {
        let project = ProjectData {
            name: "pen_test".to_string(),
            targets: vec![TargetData {
                name: "Sprite1".to_string(),
                is_stage: false,
                variables: HashMap::new(),
                blocks: vec![
                    BlockData {
                        opcode: "pen_penDown".to_string(),
                        inputs: HashMap::new(),
                        fields: HashMap::new(),
                        next: None,
                        substack: None,
                        substack2: None,
                    },
                    BlockData {
                        opcode: "pen_penUp".to_string(),
                        inputs: HashMap::new(),
                        fields: HashMap::new(),
                        next: None,
                        substack: None,
                        substack2: None,
                    },
                ],
            }],
        };
        let module = lower_project(&project).unwrap();
        let func = module.get_function("Sprite1_main").unwrap();
        assert!(func.ops.iter().any(|op| matches!(op, IrOp::Call { func, .. } if func == "pen_down")));
        assert!(func.ops.iter().any(|op| matches!(op, IrOp::Call { func, .. } if func == "pen_up")));
    }

    #[test]
    fn test_lower_multiple_targets() {
        let project = ProjectData {
            name: "multi_target".to_string(),
            targets: vec![
                TargetData {
                    name: "Stage".to_string(),
                    is_stage: true,
                    variables: HashMap::new(),
                    blocks: vec![],
                },
                TargetData {
                    name: "Cat".to_string(),
                    is_stage: false,
                    variables: HashMap::new(),
                    blocks: vec![],
                },
                TargetData {
                    name: "Dog".to_string(),
                    is_stage: false,
                    variables: HashMap::new(),
                    blocks: vec![],
                },
            ],
        };
        let module = lower_project(&project).unwrap();
        assert!(module.get_function("stage_main").is_some());
        assert!(module.get_function("Cat_main").is_some());
        assert!(module.get_function("Dog_main").is_some());
    }

    #[test]
    fn test_lower_operator_not() {
        let project = ProjectData {
            name: "not_test".to_string(),
            targets: vec![TargetData {
                name: "Stage".to_string(),
                is_stage: true,
                variables: HashMap::new(),
                blocks: vec![BlockData {
                    opcode: "operator_not".to_string(),
                    inputs: {
                        let mut m = HashMap::new();
                        m.insert("OPERAND".to_string(), InputValue::Literal(ConstValue::Bool(true)));
                        m
                    },
                    fields: HashMap::new(),
                    next: None,
                    substack: None,
                    substack2: None,
                }],
            }],
        };
        let module = lower_project(&project).unwrap();
        let func = module.get_function("stage_main").unwrap();
        assert!(func.ops.iter().any(|op| matches!(op, IrOp::UnaryOp { op: UnaryOp::Not, .. })));
    }

    #[test]
    fn test_lower_variable_input() {
        let mut vars = HashMap::new();
        vars.insert("my_var".to_string(), ConstValue::I32(5));

        let project = ProjectData {
            name: "var_input_test".to_string(),
            targets: vec![TargetData {
                name: "Stage".to_string(),
                is_stage: true,
                variables: vars,
                blocks: vec![BlockData {
                    opcode: "motion_forward".to_string(),
                    inputs: {
                        let mut m = HashMap::new();
                        m.insert("STEPS".to_string(), InputValue::Variable("my_var".to_string()));
                        m
                    },
                    fields: HashMap::new(),
                    next: None,
                    substack: None,
                    substack2: None,
                }],
            }],
        };
        let module = lower_project(&project).unwrap();
        let func = module.get_function("stage_main").unwrap();
        assert!(func.ops.iter().any(|op| matches!(op, IrOp::LoadVar { name, .. } if name == "my_var")));
    }

    #[test]
    fn test_lower_control_stop() {
        let project = ProjectData {
            name: "stop_test".to_string(),
            targets: vec![TargetData {
                name: "Stage".to_string(),
                is_stage: true,
                variables: HashMap::new(),
                blocks: vec![BlockData {
                    opcode: "control_stop".to_string(),
                    inputs: HashMap::new(),
                    fields: HashMap::new(),
                    next: None,
                    substack: None,
                    substack2: None,
                }],
            }],
        };
        let module = lower_project(&project).unwrap();
        let func = module.get_function("stage_main").unwrap();
        assert!(func.ops.iter().any(|op| matches!(op, IrOp::Return { .. })));
    }
}
