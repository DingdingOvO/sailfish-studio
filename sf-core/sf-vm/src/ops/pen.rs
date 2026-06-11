//! Pen opcode execution.

use crate::ops::{Opcode, OpcodeError};
use crate::project::Value;
use crate::runtime::RuntimeState;

/// Execute a pen opcode.
pub fn execute(
    opcode: &Opcode,
    runtime: &mut RuntimeState,
    args: &Value,
) -> Result<Value, OpcodeError> {
    match opcode {
        Opcode::PenClear => {
            // Clear all pen trails
            if let Some(_target) = runtime.current_target_state_mut() {
                // Signal to renderer to clear
            }
            Ok(Value::Null)
        }
        Opcode::PenStamp => {
            // Stamp current costume at current position
            if let Some(_target) = runtime.current_target_state() {
                // Signal to renderer to stamp
            }
            Ok(Value::Null)
        }
        Opcode::PenPenDown => {
            if let Some(target) = runtime.current_target_state_mut() {
                target.pen_down = true;
            }
            Ok(Value::Null)
        }
        Opcode::PenPenUp => {
            if let Some(target) = runtime.current_target_state_mut() {
                target.pen_down = false;
            }
            Ok(Value::Null)
        }
        Opcode::PenSetPenColorToColor => {
            if let Some(target) = runtime.current_target_state_mut() {
                match args {
                    Value::Number(n) => {
                        target.pen_color = number_to_hex_color(*n);
                    }
                    Value::String(color) => {
                        target.pen_color = color.clone();
                    }
                    _ => {}
                }
            }
            Ok(Value::Null)
        }
        Opcode::PenChangePenColorParamBy => {
            // Change pen color parameter (hue, saturation, brightness, etc.)
            // Simplified - just acknowledge
            Ok(Value::Null)
        }
        Opcode::PenSetPenColorParamTo => {
            // Set pen color parameter
            Ok(Value::Null)
        }
        Opcode::PenChangePenSizeBy => {
            let change = args.as_number().unwrap_or(0.0);
            // Store pen size as a target variable
            if let Some(target) = runtime.current_target_state_mut() {
                let current = target
                    .variables
                    .get("__pen_size")
                    .and_then(|v| v.as_number())
                    .unwrap_or(1.0);
                target.variables.insert(
                    "__pen_size".to_string(),
                    Value::Number((current + change).max(1.0)),
                );
            }
            Ok(Value::Null)
        }
        Opcode::PenSetPenSizeTo => {
            let size = args.as_number().unwrap_or(1.0).max(1.0);
            if let Some(target) = runtime.current_target_state_mut() {
                target
                    .variables
                    .insert("__pen_size".to_string(), Value::Number(size));
            }
            Ok(Value::Null)
        }
        _ => Err(OpcodeError::UnknownOpcode(format!("{:?}", opcode))),
    }
}

/// Convert a number (0-0xFFFFFF) to hex color string.
fn number_to_hex_color(n: f64) -> String {
    let n = n as u32;
    format!("#{:06X}", n & 0xFFFFFF)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::TargetState;

    fn make_runtime() -> RuntimeState {
        let mut runtime = RuntimeState::new();
        runtime.current_target = "Cat".to_string();
        runtime.add_target(TargetState::new_sprite("Cat"));
        runtime
    }

    #[test]
    fn test_pen_down() {
        let mut runtime = make_runtime();
        execute(&Opcode::PenPenDown, &mut runtime, &Value::Null).expect("should execute");
        assert!(runtime.current_target_state().unwrap().pen_down);
    }

    #[test]
    fn test_pen_up() {
        let mut runtime = make_runtime();
        // First put pen down
        execute(&Opcode::PenPenDown, &mut runtime, &Value::Null).expect("should execute");
        assert!(runtime.current_target_state().unwrap().pen_down);
        // Then put pen up
        execute(&Opcode::PenPenUp, &mut runtime, &Value::Null).expect("should execute");
        assert!(!runtime.current_target_state().unwrap().pen_down);
    }

    #[test]
    fn test_pen_clear() {
        let mut runtime = make_runtime();
        let result = execute(&Opcode::PenClear, &mut runtime, &Value::Null)
            .expect("should execute");
        assert!(result.is_null());
    }

    #[test]
    fn test_pen_stamp() {
        let mut runtime = make_runtime();
        let result = execute(&Opcode::PenStamp, &mut runtime, &Value::Null)
            .expect("should execute");
        assert!(result.is_null());
    }

    #[test]
    fn test_pen_set_color() {
        let mut runtime = make_runtime();
        execute(
            &Opcode::PenSetPenColorToColor,
            &mut runtime,
            &Value::String("#FF0000".to_string()),
        )
        .expect("should execute");
        assert_eq!(runtime.current_target_state().unwrap().pen_color, "#FF0000");
    }

    #[test]
    fn test_pen_set_color_from_number() {
        let mut runtime = make_runtime();
        execute(
            &Opcode::PenSetPenColorToColor,
            &mut runtime,
            &Value::Number(255.0), // #0000FF
        )
        .expect("should execute");
        assert_eq!(runtime.current_target_state().unwrap().pen_color, "#0000FF");
    }

    #[test]
    fn test_pen_change_size() {
        let mut runtime = make_runtime();
        // Set initial size
        execute(&Opcode::PenSetPenSizeTo, &mut runtime, &Value::Number(5.0))
            .expect("should execute");
        // Change size
        execute(&Opcode::PenChangePenSizeBy, &mut runtime, &Value::Number(3.0))
            .expect("should execute");
        let size = runtime.current_target_state().unwrap().variables.get("__pen_size");
        assert_eq!(size, Some(&Value::Number(8.0)));
    }

    #[test]
    fn test_pen_set_size() {
        let mut runtime = make_runtime();
        execute(&Opcode::PenSetPenSizeTo, &mut runtime, &Value::Number(10.0))
            .expect("should execute");
        let size = runtime.current_target_state().unwrap().variables.get("__pen_size");
        assert_eq!(size, Some(&Value::Number(10.0)));
    }

    #[test]
    fn test_pen_set_size_minimum() {
        let mut runtime = make_runtime();
        execute(&Opcode::PenSetPenSizeTo, &mut runtime, &Value::Number(-5.0))
            .expect("should execute");
        let size = runtime.current_target_state().unwrap().variables.get("__pen_size");
        assert_eq!(size, Some(&Value::Number(1.0))); // Clamped to minimum
    }

    #[test]
    fn test_number_to_hex_color() {
        assert_eq!(number_to_hex_color(0.0), "#000000");
        assert_eq!(number_to_hex_color(255.0), "#0000FF");
        assert_eq!(number_to_hex_color(16711680.0), "#FF0000");
        assert_eq!(number_to_hex_color(16776960.0), "#FFFF00");
    }
}
