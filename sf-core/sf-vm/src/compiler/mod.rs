//! Block tree traversal and JS code generation for the Sailfish VM.
//!
//! Compiles block-based programs into JavaScript code that can be
//! executed by the runtime.

use crate::project::{Block, Project, Target, Value};
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during compilation.
#[derive(Error, Debug)]
pub enum CompilerError {
    #[error("unknown opcode: {0}")]
    UnknownOpcode(String),
    #[error("missing input '{0}' for opcode '{1}'")]
    MissingInput(String, String),
    #[error("missing field '{0}' for opcode '{1}'")]
    MissingField(String, String),
    #[error("circular block reference at '{0}'")]
    CircularReference(String),
    #[error("compilation error: {0}")]
    General(String),
}

/// Compile an entire project to JavaScript.
pub fn compile(project: &Project) -> Result<String, CompilerError> {
    let mut output = String::new();
    output.push_str("// Sailfish VM - Compiled Project\n");
    output.push_str(&format!("// Project: {}\n", project.name));
    output.push_str("(async function() {\n");
    output.push_str("const runtime = sf_runtime;\n");

    for target in &project.targets {
        let target_code = compile_target(target)?;
        output.push_str(&target_code);
    }

    output.push_str("})();\n");
    Ok(output)
}

/// Compile a single target to JavaScript.
pub fn compile_target(target: &Target) -> Result<String, CompilerError> {
    let mut output = String::new();
    output.push_str(&format!("// Target: {}\n", target.name));

    // Find all top-level blocks (hat blocks)
    let top_level_blocks: Vec<&Block> = target
        .blocks
        .values()
        .filter(|b| b.top_level)
        .collect();

    for block in top_level_blocks {
        let code = compile_block_tree(block, &target.blocks)?;
        output.push_str(&code);
        output.push('\n');
    }

    Ok(output)
}

/// Compile a block and all its connected blocks (following the `next` chain).
fn compile_block_tree(
    block: &Block,
    blocks: &HashMap<String, Block>,
) -> Result<String, CompilerError> {
    let mut output = String::new();
    let mut current = Some(block);

    while let Some(b) = current {
        let code = compile_block(b, blocks)?;
        output.push_str(&code);
        output.push('\n');

        current = b
            .next
            .as_ref()
            .and_then(|id| blocks.get(id));
    }

    Ok(output)
}

/// Compile a single block to JavaScript.
pub fn compile_block(
    block: &Block,
    blocks: &HashMap<String, Block>,
) -> Result<String, CompilerError> {
    match block.opcode.as_str() {
        // Motion
        "motion_forward" => compile_motion_forward(block),
        "motion_turnright" => compile_motion_turn_right(block),
        "motion_turnleft" => compile_motion_turn_left(block),
        "motion_goto" => compile_motion_goto(block),
        "motion_gotoxy" => compile_motion_gotoxy(block),
        "motion_setx" => compile_motion_setx(block),
        "motion_sety" => compile_motion_sety(block),
        "motion_changexby" => compile_motion_changexby(block),
        "motion_changeyby" => compile_motion_changeyby(block),
        "motion_pointindirection" => compile_motion_point_direction(block),
        "motion_xposition" => Ok("runtime.getX()".to_string()),
        "motion_yposition" => Ok("runtime.getY()".to_string()),
        "motion_direction" => Ok("runtime.getDirection()".to_string()),

        // Looks
        "looks_say" => compile_looks_say(block),
        "looks_think" => compile_looks_think(block),
        "looks_show" => Ok("runtime.setVisible(true);".to_string()),
        "looks_hide" => Ok("runtime.setVisible(false);".to_string()),
        "looks_switchcostumeto" => compile_looks_switch_costume(block),
        "looks_nextcostume" => Ok("runtime.nextCostume();".to_string()),
        "looks_changesizeby" => compile_looks_change_size(block),
        "looks_setsizeto" => compile_looks_set_size(block),
        "looks_seteffectto" => compile_looks_set_effect(block),
        "looks_changeeffectby" => compile_looks_change_effect(block),
        "looks_gotofrontback" => Ok("runtime.goToFront();".to_string()),
        "looks_goforwardbackwardlayers" => compile_looks_go_layers(block),
        "looks_costumenumbername" => Ok("runtime.getCostumeNumber()".to_string()),
        "looks_size" => Ok("runtime.getSize()".to_string()),

        // Sound
        "sound_playuntildone" => compile_sound_play(block),
        "sound_play" => compile_sound_play(block),
        "sound_stopallsounds" => Ok("runtime.stopAllSounds();".to_string()),
        "sound_seteffectto" => compile_sound_set_effect(block),
        "sound_changeeffectby" => compile_sound_change_effect(block),
        "sound_setvolumeto" => compile_sound_set_volume(block),
        "sound_changevolumeby" => compile_sound_change_volume(block),
        "sound_volume" => Ok("runtime.getVolume()".to_string()),

        // Events
        "event_whenflagclicked" => compile_event_flag_clicked(block, blocks),
        "event_whenkeypressed" => compile_event_key_pressed(block, blocks),
        "event_whenbroadcastreceived" => compile_event_broadcast_received(block, blocks),
        "event_broadcast" => compile_event_broadcast(block),
        "event_broadcastandwait" => compile_event_broadcast_and_wait(block),

        // Control
        "control_wait" => compile_control_wait(block),
        "control_repeat" => compile_control_repeat(block, blocks),
        "control_if" => compile_control_if(block, blocks),
        "control_if_else" => compile_control_if_else(block, blocks),
        "control_forever" => compile_control_forever(block, blocks),
        "control_stop" => Ok("return;".to_string()),
        "control_create_clone_of" => compile_control_create_clone(block),
        "control_delete_this_clone" => Ok("runtime.deleteThisClone();".to_string()),

        // Sensing
        "sensing_askandwait" => compile_sensing_ask(block),
        "sensing_answer" => Ok("runtime.getAnswer()".to_string()),
        "sensing_timer" => Ok("runtime.getTimer()".to_string()),
        "sensing_resettimer" => Ok("runtime.resetTimer();".to_string()),
        "sensing_keypressed" => compile_sensing_key_pressed(block),
        "sensing_mousedown" => Ok("runtime.isMouseDown()".to_string()),
        "sensing_mousex" => Ok("runtime.getMouseX()".to_string()),
        "sensing_mousey" => Ok("runtime.getMouseY()".to_string()),
        "sensing_loudness" => Ok("runtime.getLoudness()".to_string()),
        "sensing_current" => compile_sensing_current(block),
        "sensing_dayssince2000" => Ok("runtime.daysSince2000()".to_string()),
        "sensing_touchingobject" => compile_sensing_touching(block),

        // Operators
        "operator_add" => compile_operator_binary(block, "+"),
        "operator_subtract" => compile_operator_binary(block, "-"),
        "operator_multiply" => compile_operator_binary(block, "*"),
        "operator_divide" => compile_operator_divide(block),
        "operator_random" => compile_operator_random(block),
        "operator_gt" => compile_operator_comparison(block, ">"),
        "operator_lt" => compile_operator_comparison(block, "<"),
        "operator_equals" => compile_operator_comparison(block, "=="),
        "operator_and" => compile_operator_logical(block, "&&"),
        "operator_or" => compile_operator_logical(block, "||"),
        "operator_not" => compile_operator_not(block),
        "operator_join" => compile_operator_join(block),
        "operator_letter_of" => compile_operator_letter_of(block),
        "operator_length" => compile_operator_length(block),
        "operator_contains" => compile_operator_contains(block),
        "operator_mod" => compile_operator_mod(block),
        "operator_round" => compile_operator_round(block),
        "operator_mathop" => compile_operator_mathop(block),

        // Variables
        "data_setvariableto" => compile_data_set_variable(block),
        "data_changevariableby" => compile_data_change_variable(block),
        "data_variable" => compile_data_variable(block),
        "data_showvariable" => compile_data_show_variable(block),
        "data_hidevariable" => compile_data_hide_variable(block),
        "data_addtolist" => compile_data_add_to_list(block),
        "data_deleteoflist" => compile_data_delete_of_list(block),
        "data_insertatlist" => compile_data_insert_at_list(block),
        "data_replaceitemoflist" => compile_data_replace_item_of_list(block),
        "data_itemoflist" => compile_data_item_of_list(block),
        "data_lengthoflist" => compile_data_length_of_list(block),
        "data_listcontainsitem" => compile_data_list_contains(block),
        "data_showlist" => compile_data_show_list(block),
        "data_hidelist" => compile_data_hide_list(block),

        // Pen
        "pen_clear" => Ok("runtime.penClear();".to_string()),
        "pen_stamp" => Ok("runtime.penStamp();".to_string()),
        "pen_penDown" => Ok("runtime.penDown();".to_string()),
        "pen_penUp" => Ok("runtime.penUp();".to_string()),
        "pen_setPenColorToColor" => compile_pen_set_color(block),
        "pen_changePenColorParamBy" => compile_pen_change_param(block),
        "pen_setPenColorParamTo" => compile_pen_set_param(block),
        "pen_changePenSizeBy" => compile_pen_change_size(block),
        "pen_setPenSizeTo" => compile_pen_set_size(block),

        _ => Err(CompilerError::UnknownOpcode(block.opcode.clone())),
    }
}

// --- Helper functions ---

/// Resolve an input value to a JS expression string.
fn resolve_input(block: &Block, input_name: &str) -> Result<String, CompilerError> {
    let input = block
        .inputs
        .get(input_name)
        .ok_or_else(|| CompilerError::MissingInput(input_name.to_string(), block.opcode.clone()))?;

    if let Some(ref value) = input.value {
        Ok(value_to_js(value))
    } else if let Some(ref block_id) = input.block_id {
        Ok(format!("/* block:{} */", block_id))
    } else {
        Ok("0".to_string())
    }
}

/// Convert a Value to a JS expression.
fn value_to_js(value: &Value) -> String {
    match value {
        Value::Number(n) => format!("{}", n),
        Value::String(s) => format!("\"{}\"", s.replace('"', "\\\"")),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        Value::List(items) => {
            let items_js: Vec<String> = items.iter().map(value_to_js).collect();
            format!("[{}]", items_js.join(", "))
        }
    }
}

/// Resolve a field value.
fn resolve_field(block: &Block, field_name: &str) -> Result<String, CompilerError> {
    block
        .fields
        .get(field_name)
        .map(|f| f.value.clone())
        .ok_or_else(|| CompilerError::MissingField(field_name.to_string(), block.opcode.clone()))
}

// --- Motion compilers ---

fn compile_motion_forward(block: &Block) -> Result<String, CompilerError> {
    let steps = resolve_input(block, "STEPS")?;
    Ok(format!("runtime.moveForward({});", steps))
}

fn compile_motion_turn_right(block: &Block) -> Result<String, CompilerError> {
    let degrees = resolve_input(block, "DEGREES")?;
    Ok(format!("runtime.turnRight({});", degrees))
}

fn compile_motion_turn_left(block: &Block) -> Result<String, CompilerError> {
    let degrees = resolve_input(block, "DEGREES")?;
    Ok(format!("runtime.turnLeft({});", degrees))
}

fn compile_motion_goto(block: &Block) -> Result<String, CompilerError> {
    let to = resolve_input(block, "TO")?;
    Ok(format!("runtime.goTo({});", to))
}

fn compile_motion_gotoxy(block: &Block) -> Result<String, CompilerError> {
    let x = resolve_input(block, "X")?;
    let y = resolve_input(block, "Y")?;
    Ok(format!("runtime.goTo({}, {});", x, y))
}

fn compile_motion_setx(block: &Block) -> Result<String, CompilerError> {
    let x = resolve_input(block, "X")?;
    Ok(format!("runtime.setX({});", x))
}

fn compile_motion_sety(block: &Block) -> Result<String, CompilerError> {
    let y = resolve_input(block, "Y")?;
    Ok(format!("runtime.setY({});", y))
}

fn compile_motion_changexby(block: &Block) -> Result<String, CompilerError> {
    let dx = resolve_input(block, "DX")?;
    Ok(format!("runtime.changeX({});", dx))
}

fn compile_motion_changeyby(block: &Block) -> Result<String, CompilerError> {
    let dy = resolve_input(block, "DY")?;
    Ok(format!("runtime.changeY({});", dy))
}

fn compile_motion_point_direction(block: &Block) -> Result<String, CompilerError> {
    let dir = resolve_input(block, "DIRECTION")?;
    Ok(format!("runtime.setDirection({});", dir))
}

// --- Looks compilers ---

fn compile_looks_say(block: &Block) -> Result<String, CompilerError> {
    let msg = resolve_input(block, "MESSAGE")?;
    Ok(format!("runtime.say({});", msg))
}

fn compile_looks_think(block: &Block) -> Result<String, CompilerError> {
    let msg = resolve_input(block, "MESSAGE")?;
    Ok(format!("runtime.think({});", msg))
}

fn compile_looks_switch_costume(block: &Block) -> Result<String, CompilerError> {
    let costume = resolve_input(block, "COSTUME")?;
    Ok(format!("runtime.switchCostume({});", costume))
}

fn compile_looks_change_size(block: &Block) -> Result<String, CompilerError> {
    let change = resolve_input(block, "CHANGE")?;
    Ok(format!("runtime.changeSize({});", change))
}

fn compile_looks_set_size(block: &Block) -> Result<String, CompilerError> {
    let size = resolve_input(block, "SIZE")?;
    Ok(format!("runtime.setSize({});", size))
}

fn compile_looks_set_effect(block: &Block) -> Result<String, CompilerError> {
    let effect = resolve_field(block, "EFFECT")?;
    let value = resolve_input(block, "VALUE")?;
    Ok(format!("runtime.setEffect(\"{}\", {});", effect, value))
}

fn compile_looks_change_effect(block: &Block) -> Result<String, CompilerError> {
    let effect = resolve_field(block, "EFFECT")?;
    let value = resolve_input(block, "VALUE")?;
    Ok(format!("runtime.changeEffect(\"{}\", {});", effect, value))
}

fn compile_looks_go_layers(block: &Block) -> Result<String, CompilerError> {
    let direction = resolve_field(block, "FORWARD_BACKWARD")?;
    let num = resolve_input(block, "NUM")?;
    Ok(format!("runtime.goLayers(\"{}\", {});", direction, num))
}

// --- Sound compilers ---

fn compile_sound_play(block: &Block) -> Result<String, CompilerError> {
    let sound = resolve_input(block, "SOUND_MENU")?;
    Ok(format!("runtime.playSound({});", sound))
}

fn compile_sound_set_effect(block: &Block) -> Result<String, CompilerError> {
    let effect = resolve_field(block, "EFFECT")?;
    let value = resolve_input(block, "VALUE")?;
    Ok(format!("runtime.setSoundEffect(\"{}\", {});", effect, value))
}

fn compile_sound_change_effect(block: &Block) -> Result<String, CompilerError> {
    let effect = resolve_field(block, "EFFECT")?;
    let value = resolve_input(block, "VALUE")?;
    Ok(format!("runtime.changeSoundEffect(\"{}\", {});", effect, value))
}

fn compile_sound_set_volume(block: &Block) -> Result<String, CompilerError> {
    let volume = resolve_input(block, "VOLUME")?;
    Ok(format!("runtime.setVolume({});", volume))
}

fn compile_sound_change_volume(block: &Block) -> Result<String, CompilerError> {
    let change = resolve_input(block, "VOLUME")?;
    Ok(format!("runtime.changeVolume({});", change))
}

// --- Event compilers ---

fn compile_event_flag_clicked(
    _block: &Block,
    _blocks: &HashMap<String, Block>,
) -> Result<String, CompilerError> {
    Ok(format!("// when green flag clicked\nruntime.onStart(async () => {{"))
}

fn compile_event_key_pressed(
    block: &Block,
    _blocks: &HashMap<String, Block>,
) -> Result<String, CompilerError> {
    let key = resolve_field(block, "KEY_OPTION")?;
    Ok(format!(
        "// when key pressed: {}\nruntime.onKey(\"{}\", async () => {{",
        key, key
    ))
}

fn compile_event_broadcast_received(
    block: &Block,
    _blocks: &HashMap<String, Block>,
) -> Result<String, CompilerError> {
    let msg = resolve_field(block, "BROADCAST_OPTION")?;
    Ok(format!(
        "// when I receive: {}\nruntime.onBroadcast(\"{}\", async () => {{",
        msg, msg
    ))
}

fn compile_event_broadcast(block: &Block) -> Result<String, CompilerError> {
    let msg = resolve_input(block, "BROADCAST_INPUT")?;
    Ok(format!("runtime.broadcast({});", msg))
}

fn compile_event_broadcast_and_wait(block: &Block) -> Result<String, CompilerError> {
    let msg = resolve_input(block, "BROADCAST_INPUT")?;
    Ok(format!("await runtime.broadcastAndWait({});", msg))
}

// --- Control compilers ---

fn compile_control_wait(block: &Block) -> Result<String, CompilerError> {
    let duration = resolve_input(block, "DURATION")?;
    Ok(format!("await runtime.wait({});", duration))
}

fn compile_control_repeat(
    block: &Block,
    blocks: &HashMap<String, Block>,
) -> Result<String, CompilerError> {
    let times = resolve_input(block, "TIMES")?;
    let substack = compile_substack(block, "SUBSTACK", blocks)?;
    Ok(format!(
        "for (let i = 0; i < {}; i++) {{\n{}\n}}",
        times, substack
    ))
}

fn compile_control_if(
    block: &Block,
    blocks: &HashMap<String, Block>,
) -> Result<String, CompilerError> {
    let condition = resolve_input(block, "CONDITION")?;
    let substack = compile_substack(block, "SUBSTACK", blocks)?;
    Ok(format!("if ({}) {{\n{}\n}}", condition, substack))
}

fn compile_control_if_else(
    block: &Block,
    blocks: &HashMap<String, Block>,
) -> Result<String, CompilerError> {
    let condition = resolve_input(block, "CONDITION")?;
    let if_substack = compile_substack(block, "SUBSTACK", blocks)?;
    let else_substack = compile_substack(block, "SUBSTACK2", blocks)?;
    Ok(format!(
        "if ({}) {{\n{}\n}} else {{\n{}\n}}",
        condition, if_substack, else_substack
    ))
}

fn compile_control_forever(
    block: &Block,
    blocks: &HashMap<String, Block>,
) -> Result<String, CompilerError> {
    let substack = compile_substack(block, "SUBSTACK", blocks)?;
    Ok(format!(
        "while (runtime.isRunning()) {{\n{}\nawait runtime.yield();\n}}",
        substack
    ))
}

fn compile_control_create_clone(block: &Block) -> Result<String, CompilerError> {
    let target = resolve_input(block, "CLONE_OPTION")?;
    Ok(format!("runtime.createClone({});", target))
}

/// Compile a substack (the blocks inside a C-shaped block).
fn compile_substack(
    block: &Block,
    input_name: &str,
    blocks: &HashMap<String, Block>,
) -> Result<String, CompilerError> {
    if let Some(input) = block.inputs.get(input_name) {
        if let Some(ref block_id) = input.block_id {
            if let Some(sub_block) = blocks.get(block_id) {
                return compile_block_tree(sub_block, blocks);
            }
        }
    }
    Ok(String::new())
}

// --- Sensing compilers ---

fn compile_sensing_ask(block: &Block) -> Result<String, CompilerError> {
    let question = resolve_input(block, "QUESTION")?;
    Ok(format!("await runtime.askAndWait({});", question))
}

fn compile_sensing_key_pressed(block: &Block) -> Result<String, CompilerError> {
    let key = resolve_input(block, "KEY_OPTION")?;
    Ok(format!("runtime.isKeyPressed({})", key))
}

fn compile_sensing_current(block: &Block) -> Result<String, CompilerError> {
    let menu = resolve_field(block, "CURRENTMENU")?;
    Ok(format!("runtime.current(\"{}\")", menu))
}

fn compile_sensing_touching(block: &Block) -> Result<String, CompilerError> {
    let object = resolve_input(block, "TOUCHINGOBJECTMENU")?;
    Ok(format!("runtime.isTouching({})", object))
}

// --- Operator compilers ---

fn compile_operator_binary(block: &Block, op: &str) -> Result<String, CompilerError> {
    let left = resolve_input(block, "NUM1")?;
    let right = resolve_input(block, "NUM2")?;
    Ok(format!("({} {} {})", left, op, right))
}

fn compile_operator_divide(block: &Block) -> Result<String, CompilerError> {
    let left = resolve_input(block, "NUM1")?;
    let right = resolve_input(block, "NUM2")?;
    Ok(format!("runtime.safeDivide({}, {})", left, right))
}

fn compile_operator_random(block: &Block) -> Result<String, CompilerError> {
    let from = resolve_input(block, "FROM")?;
    let to = resolve_input(block, "TO")?;
    Ok(format!("runtime.random({}, {})", from, to))
}

fn compile_operator_comparison(block: &Block, op: &str) -> Result<String, CompilerError> {
    let left = resolve_input(block, "OPERAND1")?;
    let right = resolve_input(block, "OPERAND2")?;
    Ok(format!("({} {} {})", left, op, right))
}

fn compile_operator_logical(block: &Block, op: &str) -> Result<String, CompilerError> {
    let left = resolve_input(block, "OPERAND1")?;
    let right = resolve_input(block, "OPERAND2")?;
    Ok(format!("({} {} {})", left, op, right))
}

fn compile_operator_not(block: &Block) -> Result<String, CompilerError> {
    let operand = resolve_input(block, "OPERAND")?;
    Ok(format!("(!{})", operand))
}

fn compile_operator_join(block: &Block) -> Result<String, CompilerError> {
    let left = resolve_input(block, "STRING1")?;
    let right = resolve_input(block, "STRING2")?;
    Ok(format!("String({}) + String({})", left, right))
}

fn compile_operator_letter_of(block: &Block) -> Result<String, CompilerError> {
    let letter = resolve_input(block, "LETTER")?;
    let string = resolve_input(block, "STRING")?;
    Ok(format!("String({}).charAt({} - 1)", string, letter))
}

fn compile_operator_length(block: &Block) -> Result<String, CompilerError> {
    let string = resolve_input(block, "STRING")?;
    Ok(format!("String({}).length", string))
}

fn compile_operator_contains(block: &Block) -> Result<String, CompilerError> {
    let string = resolve_input(block, "STRING1")?;
    let contains = resolve_input(block, "STRING2")?;
    Ok(format!(
        "String({}).includes(String({}))",
        string, contains
    ))
}

fn compile_operator_mod(block: &Block) -> Result<String, CompilerError> {
    let left = resolve_input(block, "NUM1")?;
    let right = resolve_input(block, "NUM2")?;
    Ok(format!("runtime.mod({}, {})", left, right))
}

fn compile_operator_round(block: &Block) -> Result<String, CompilerError> {
    let num = resolve_input(block, "NUM")?;
    Ok(format!("Math.round({})", num))
}

fn compile_operator_mathop(block: &Block) -> Result<String, CompilerError> {
    let operator = resolve_field(block, "OPERATOR")?;
    let num = resolve_input(block, "NUM")?;
    let fn_name = match operator.as_str() {
        "abs" => "Math.abs",
        "floor" => "Math.floor",
        "ceiling" => "Math.ceil",
        "sqrt" => "Math.sqrt",
        "sin" => "Math.sin",
        "cos" => "Math.cos",
        "tan" => "Math.tan",
        "asin" => "Math.asin",
        "acos" => "Math.acos",
        "atan" => "Math.atan",
        "ln" => "Math.log",
        "log" => "Math.log10",
        "e ^" => "Math.exp",
        "10 ^" => {
            return Ok(format!("Math.pow(10, {})", num));
        }
        _ => {
            return Ok(format!("Math.{}({})", operator, num));
        }
    };
    Ok(format!("{}({})", fn_name, num))
}

// --- Variable compilers ---

fn compile_data_set_variable(block: &Block) -> Result<String, CompilerError> {
    let var = resolve_field(block, "VARIABLE")?;
    let value = resolve_input(block, "VALUE")?;
    Ok(format!("runtime.setVariable(\"{}\", {});", var, value))
}

fn compile_data_change_variable(block: &Block) -> Result<String, CompilerError> {
    let var = resolve_field(block, "VARIABLE")?;
    let value = resolve_input(block, "VALUE")?;
    Ok(format!(
        "runtime.changeVariable(\"{}\", {});",
        var, value
    ))
}

fn compile_data_variable(block: &Block) -> Result<String, CompilerError> {
    let var = resolve_field(block, "VARIABLE")?;
    Ok(format!("runtime.getVariable(\"{}\")", var))
}

fn compile_data_show_variable(block: &Block) -> Result<String, CompilerError> {
    let var = resolve_field(block, "VARIABLE")?;
    Ok(format!("runtime.showVariable(\"{}\");", var))
}

fn compile_data_hide_variable(block: &Block) -> Result<String, CompilerError> {
    let var = resolve_field(block, "VARIABLE")?;
    Ok(format!("runtime.hideVariable(\"{}\");", var))
}

fn compile_data_add_to_list(block: &Block) -> Result<String, CompilerError> {
    let list = resolve_field(block, "LIST")?;
    let item = resolve_input(block, "ITEM")?;
    Ok(format!("runtime.addToList(\"{}\", {});", list, item))
}

fn compile_data_delete_of_list(block: &Block) -> Result<String, CompilerError> {
    let list = resolve_field(block, "LIST")?;
    let index = resolve_input(block, "INDEX")?;
    Ok(format!("runtime.deleteOfList(\"{}\", {});", list, index))
}

fn compile_data_insert_at_list(block: &Block) -> Result<String, CompilerError> {
    let list = resolve_field(block, "LIST")?;
    let index = resolve_input(block, "INDEX")?;
    let item = resolve_input(block, "ITEM")?;
    Ok(format!(
        "runtime.insertAtList(\"{}\", {}, {});",
        list, index, item
    ))
}

fn compile_data_replace_item_of_list(block: &Block) -> Result<String, CompilerError> {
    let list = resolve_field(block, "LIST")?;
    let index = resolve_input(block, "INDEX")?;
    let item = resolve_input(block, "ITEM")?;
    Ok(format!(
        "runtime.replaceItemOfList(\"{}\", {}, {});",
        list, index, item
    ))
}

fn compile_data_item_of_list(block: &Block) -> Result<String, CompilerError> {
    let list = resolve_field(block, "LIST")?;
    let index = resolve_input(block, "INDEX")?;
    Ok(format!("runtime.itemOfList(\"{}\", {})", list, index))
}

fn compile_data_length_of_list(block: &Block) -> Result<String, CompilerError> {
    let list = resolve_field(block, "LIST")?;
    Ok(format!("runtime.lengthOfList(\"{}\")", list))
}

fn compile_data_list_contains(block: &Block) -> Result<String, CompilerError> {
    let list = resolve_field(block, "LIST")?;
    let item = resolve_input(block, "ITEM")?;
    Ok(format!("runtime.listContains(\"{}\", {})", list, item))
}

fn compile_data_show_list(block: &Block) -> Result<String, CompilerError> {
    let list = resolve_field(block, "LIST")?;
    Ok(format!("runtime.showList(\"{}\");", list))
}

fn compile_data_hide_list(block: &Block) -> Result<String, CompilerError> {
    let list = resolve_field(block, "LIST")?;
    Ok(format!("runtime.hideList(\"{}\");", list))
}

// --- Pen compilers ---

fn compile_pen_set_color(block: &Block) -> Result<String, CompilerError> {
    let color = resolve_input(block, "COLOR")?;
    Ok(format!("runtime.setPenColor({});", color))
}

fn compile_pen_change_param(block: &Block) -> Result<String, CompilerError> {
    let param = resolve_field(block, "COLOR_PARAM")?;
    let value = resolve_input(block, "VALUE")?;
    Ok(format!(
        "runtime.changePenParam(\"{}\", {});",
        param, value
    ))
}

fn compile_pen_set_param(block: &Block) -> Result<String, CompilerError> {
    let param = resolve_field(block, "COLOR_PARAM")?;
    let value = resolve_input(block, "VALUE")?;
    Ok(format!("runtime.setPenParam(\"{}\", {});", param, value))
}

fn compile_pen_change_size(block: &Block) -> Result<String, CompilerError> {
    let size = resolve_input(block, "SIZE")?;
    Ok(format!("runtime.changePenSize({});", size))
}

fn compile_pen_set_size(block: &Block) -> Result<String, CompilerError> {
    let size = resolve_input(block, "SIZE")?;
    Ok(format!("runtime.setPenSize({});", size))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::BlockInput;

    fn _make_simple_block(id: &str, opcode: &str) -> Block {
        Block::new(id, opcode)
    }

    fn make_block_with_input(id: &str, opcode: &str, input_name: &str, value: Value) -> Block {
        Block::new(id, opcode).with_input(
            input_name,
            BlockInput {
                input_type: "shadow".to_string(),
                value: Some(value),
                block_id: None,
            },
        )
    }

    fn make_block_with_field(id: &str, opcode: &str, field_name: &str, field_value: &str) -> Block {
        Block::new(id, opcode).with_field(field_name, field_value)
    }

    #[test]
    fn test_compile_empty_project() {
        let project = Project::new("EmptyProject");
        let result = compile(&project).expect("should compile");
        assert!(result.contains("EmptyProject"));
        assert!(result.contains("async function"));
    }

    #[test]
    fn test_compile_project_with_target() {
        let mut project = Project::new("TestProject");
        let mut stage = Target::new_stage();
        let hat = Block::new_top_level("hat1", "event_whenflagclicked");
        let block1 = Block::new("block1", "motion_forward")
            .with_parent("hat1")
            .with_input(
                "STEPS",
                BlockInput {
                    input_type: "shadow".to_string(),
                    value: Some(Value::Number(10.0)),
                    block_id: None,
                },
            );
        let hat = hat.with_next("block1");

        stage.blocks.insert("hat1".to_string(), hat);
        stage.blocks.insert("block1".to_string(), block1);
        project.targets.push(stage);

        let result = compile(&project).expect("should compile");
        assert!(result.contains("moveForward"));
        assert!(result.contains("onStart"));
    }

    #[test]
    fn test_compile_motion_forward() {
        let block = make_block_with_input("b1", "motion_forward", "STEPS", Value::Number(15.0));
        let blocks = HashMap::new();
        let result = compile_block(&block, &blocks).expect("should compile");
        assert_eq!(result, "runtime.moveForward(15);");
    }

    #[test]
    fn test_compile_motion_turn_right() {
        let block = make_block_with_input("b1", "motion_turnright", "DEGREES", Value::Number(90.0));
        let blocks = HashMap::new();
        let result = compile_block(&block, &blocks).expect("should compile");
        assert_eq!(result, "runtime.turnRight(90);");
    }

    #[test]
    fn test_compile_motion_turn_left() {
        let block = make_block_with_input("b1", "motion_turnleft", "DEGREES", Value::Number(45.0));
        let blocks = HashMap::new();
        let result = compile_block(&block, &blocks).expect("should compile");
        assert_eq!(result, "runtime.turnLeft(45);");
    }

    #[test]
    fn test_compile_motion_goto() {
        let block = make_block_with_input("b1", "motion_goto", "TO", Value::String("_random_".to_string()));
        let blocks = HashMap::new();
        let result = compile_block(&block, &blocks).expect("should compile");
        assert!(result.contains("goTo"));
    }

    #[test]
    fn test_compile_looks_say() {
        let block = make_block_with_input("b1", "looks_say", "MESSAGE", Value::String("Hello!".to_string()));
        let blocks = HashMap::new();
        let result = compile_block(&block, &blocks).expect("should compile");
        assert!(result.contains("runtime.say"));
        assert!(result.contains("Hello!"));
    }

    #[test]
    fn test_compile_looks_think() {
        let block = make_block_with_input("b1", "looks_think", "MESSAGE", Value::String("Hmm...".to_string()));
        let blocks = HashMap::new();
        let result = compile_block(&block, &blocks).expect("should compile");
        assert!(result.contains("runtime.think"));
        assert!(result.contains("Hmm..."));
    }

    #[test]
    fn test_compile_control_wait() {
        let block = make_block_with_input("b1", "control_wait", "DURATION", Value::Number(1.0));
        let blocks = HashMap::new();
        let result = compile_block(&block, &blocks).expect("should compile");
        assert_eq!(result, "await runtime.wait(1);");
    }

    #[test]
    fn test_compile_control_repeat() {
        let block = make_block_with_input("b1", "control_repeat", "TIMES", Value::Number(10.0));
        let blocks = HashMap::new();
        let result = compile_block(&block, &blocks).expect("should compile");
        assert!(result.contains("for"));
        assert!(result.contains("10"));
    }

    #[test]
    fn test_compile_control_if() {
        let block = make_block_with_input("b1", "control_if", "CONDITION", Value::Bool(true));
        let blocks = HashMap::new();
        let result = compile_block(&block, &blocks).expect("should compile");
        assert!(result.contains("if"));
        assert!(result.contains("true"));
    }

    #[test]
    fn test_compile_control_if_else() {
        let block = make_block_with_input("b1", "control_if_else", "CONDITION", Value::Bool(true));
        let blocks = HashMap::new();
        let result = compile_block(&block, &blocks).expect("should compile");
        assert!(result.contains("if"));
        assert!(result.contains("else"));
    }

    #[test]
    fn test_compile_control_forever() {
        let block = Block::new("b1", "control_forever");
        let blocks = HashMap::new();
        let result = compile_block(&block, &blocks).expect("should compile");
        assert!(result.contains("while"));
        assert!(result.contains("isRunning"));
        assert!(result.contains("yield"));
    }

    #[test]
    fn test_compile_event_whenflagclicked() {
        let block = Block::new_top_level("b1", "event_whenflagclicked");
        let blocks = HashMap::new();
        let result = compile_block(&block, &blocks).expect("should compile");
        assert!(result.contains("onStart"));
    }

    #[test]
    fn test_compile_event_whenkeypressed() {
        let block = make_block_with_field("b1", "event_whenkeypressed", "KEY_OPTION", "space");
        let blocks = HashMap::new();
        let result = compile_block(&block, &blocks).expect("should compile");
        assert!(result.contains("onKey"));
        assert!(result.contains("space"));
    }

    #[test]
    fn test_compile_sensing_askandwait() {
        let block = make_block_with_input("b1", "sensing_askandwait", "QUESTION", Value::String("What?".to_string()));
        let blocks = HashMap::new();
        let result = compile_block(&block, &blocks).expect("should compile");
        assert!(result.contains("askAndWait"));
        assert!(result.contains("What?"));
    }

    #[test]
    fn test_compile_sensing_timer() {
        let block = Block::new("b1", "sensing_timer");
        let blocks = HashMap::new();
        let result = compile_block(&block, &blocks).expect("should compile");
        assert!(result.contains("getTimer"));
    }

    #[test]
    fn test_compile_operator_add() {
        let block = Block::new("b1", "operator_add")
            .with_input("NUM1", BlockInput { input_type: "shadow".to_string(), value: Some(Value::Number(3.0)), block_id: None })
            .with_input("NUM2", BlockInput { input_type: "shadow".to_string(), value: Some(Value::Number(4.0)), block_id: None });
        let blocks = HashMap::new();
        let result = compile_block(&block, &blocks).expect("should compile");
        assert_eq!(result, "(3 + 4)");
    }

    #[test]
    fn test_compile_operator_subtract() {
        let block = Block::new("b1", "operator_subtract")
            .with_input("NUM1", BlockInput { input_type: "shadow".to_string(), value: Some(Value::Number(10.0)), block_id: None })
            .with_input("NUM2", BlockInput { input_type: "shadow".to_string(), value: Some(Value::Number(3.0)), block_id: None });
        let blocks = HashMap::new();
        let result = compile_block(&block, &blocks).expect("should compile");
        assert_eq!(result, "(10 - 3)");
    }

    #[test]
    fn test_compile_operator_multiply() {
        let block = Block::new("b1", "operator_multiply")
            .with_input("NUM1", BlockInput { input_type: "shadow".to_string(), value: Some(Value::Number(6.0)), block_id: None })
            .with_input("NUM2", BlockInput { input_type: "shadow".to_string(), value: Some(Value::Number(7.0)), block_id: None });
        let blocks = HashMap::new();
        let result = compile_block(&block, &blocks).expect("should compile");
        assert_eq!(result, "(6 * 7)");
    }

    #[test]
    fn test_compile_operator_random() {
        let block = Block::new("b1", "operator_random")
            .with_input("FROM", BlockInput { input_type: "shadow".to_string(), value: Some(Value::Number(1.0)), block_id: None })
            .with_input("TO", BlockInput { input_type: "shadow".to_string(), value: Some(Value::Number(10.0)), block_id: None });
        let blocks = HashMap::new();
        let result = compile_block(&block, &blocks).expect("should compile");
        assert!(result.contains("random(1, 10)"));
    }

    #[test]
    fn test_compile_data_setvariableto() {
        let block = Block::new("b1", "data_setvariableto")
            .with_field("VARIABLE", "score")
            .with_input("VALUE", BlockInput { input_type: "shadow".to_string(), value: Some(Value::Number(100.0)), block_id: None });
        let blocks = HashMap::new();
        let result = compile_block(&block, &blocks).expect("should compile");
        assert!(result.contains("setVariable"));
        assert!(result.contains("score"));
        assert!(result.contains("100"));
    }

    #[test]
    fn test_compile_data_changevariableby() {
        let block = Block::new("b1", "data_changevariableby")
            .with_field("VARIABLE", "score")
            .with_input("VALUE", BlockInput { input_type: "shadow".to_string(), value: Some(Value::Number(1.0)), block_id: None });
        let blocks = HashMap::new();
        let result = compile_block(&block, &blocks).expect("should compile");
        assert!(result.contains("changeVariable"));
        assert!(result.contains("score"));
    }

    #[test]
    fn test_compile_pen_clear() {
        let block = Block::new("b1", "pen_clear");
        let blocks = HashMap::new();
        let result = compile_block(&block, &blocks).expect("should compile");
        assert_eq!(result, "runtime.penClear();");
    }

    #[test]
    fn test_compile_pen_stamp() {
        let block = Block::new("b1", "pen_stamp");
        let blocks = HashMap::new();
        let result = compile_block(&block, &blocks).expect("should compile");
        assert_eq!(result, "runtime.penStamp();");
    }

    #[test]
    fn test_compile_pen_pen_down() {
        let block = Block::new("b1", "pen_penDown");
        let blocks = HashMap::new();
        let result = compile_block(&block, &blocks).expect("should compile");
        assert_eq!(result, "runtime.penDown();");
    }

    #[test]
    fn test_compile_pen_pen_up() {
        let block = Block::new("b1", "pen_penUp");
        let blocks = HashMap::new();
        let result = compile_block(&block, &blocks).expect("should compile");
        assert_eq!(result, "runtime.penUp();");
    }

    #[test]
    fn test_compile_unknown_opcode() {
        let block = Block::new("b1", "custom_unknown_opcode");
        let blocks = HashMap::new();
        let result = compile_block(&block, &blocks);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("unknown opcode"));
    }

    #[test]
    fn test_compile_block_chain() {
        let mut target = Target::new_sprite("Test");
        let hat = Block::new_top_level("hat1", "event_whenflagclicked").with_next("b1");
        let b1 = Block::new("b1", "motion_forward")
            .with_parent("hat1")
            .with_next("b2")
            .with_input(
                "STEPS",
                BlockInput {
                    input_type: "shadow".to_string(),
                    value: Some(Value::Number(10.0)),
                    block_id: None,
                },
            );
        let b2 = Block::new("b2", "motion_turnright")
            .with_parent("b1")
            .with_input(
                "DEGREES",
                BlockInput {
                    input_type: "shadow".to_string(),
                    value: Some(Value::Number(15.0)),
                    block_id: None,
                },
            );

        target.blocks.insert("hat1".to_string(), hat);
        target.blocks.insert("b1".to_string(), b1);
        target.blocks.insert("b2".to_string(), b2);

        let result = compile_target(&target).expect("should compile target");
        assert!(result.contains("moveForward(10)"));
        assert!(result.contains("turnRight(15)"));
    }

    #[test]
    fn test_value_to_js() {
        assert_eq!(value_to_js(&Value::Number(42.0)), "42");
        assert_eq!(value_to_js(&Value::String("hello".to_string())), "\"hello\"");
        assert_eq!(value_to_js(&Value::Bool(true)), "true");
        assert_eq!(value_to_js(&Value::Null), "null");
        assert_eq!(
            value_to_js(&Value::List(vec![Value::Number(1.0), Value::Number(2.0)])),
            "[1, 2]"
        );
    }

    #[test]
    fn test_compile_operator_join() {
        let block = Block::new("b1", "operator_join")
            .with_input("STRING1", BlockInput { input_type: "shadow".to_string(), value: Some(Value::String("hello".to_string())), block_id: None })
            .with_input("STRING2", BlockInput { input_type: "shadow".to_string(), value: Some(Value::String(" world".to_string())), block_id: None });
        let blocks = HashMap::new();
        let result = compile_block(&block, &blocks).expect("should compile");
        assert!(result.contains("String(\"hello\") + String(\" world\")"));
    }
}
