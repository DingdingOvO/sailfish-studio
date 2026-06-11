//! Motion opcode execution.

use crate::ops::{Opcode, OpcodeError};
use crate::project::Value;
use crate::runtime::RuntimeState;

/// Execute a motion opcode.
pub fn execute(
    opcode: &Opcode,
    runtime: &mut RuntimeState,
    args: &Value,
) -> Result<Value, OpcodeError> {
    match opcode {
        Opcode::MotionForward => {
            let steps = args.as_number().unwrap_or(10.0);
            if let Some(target) = runtime.current_target_state_mut() {
                target.move_forward(steps);
            }
            Ok(Value::Null)
        }
        Opcode::MotionTurnRight => {
            let degrees = args.as_number().unwrap_or(15.0);
            if let Some(target) = runtime.current_target_state_mut() {
                target.turn_right(degrees);
            }
            Ok(Value::Null)
        }
        Opcode::MotionTurnLeft => {
            let degrees = args.as_number().unwrap_or(15.0);
            if let Some(target) = runtime.current_target_state_mut() {
                target.turn_left(degrees);
            }
            Ok(Value::Null)
        }
        Opcode::MotionGoto => {
            // Args could be a list [x, y] or a named target
            if let Value::List(coords) = args {
                if coords.len() >= 2 {
                    let x = coords[0].as_number().unwrap_or(0.0);
                    let y = coords[1].as_number().unwrap_or(0.0);
                    if let Some(target) = runtime.current_target_state_mut() {
                        target.go_to(x, y);
                    }
                }
            } else if let Value::String(name) = args {
                // Go to another sprite's position
                if let Some(other) = runtime.targets.get(name.as_str()) {
                    let (x, y) = (other.x, other.y);
                    if let Some(target) = runtime.current_target_state_mut() {
                        target.go_to(x, y);
                    }
                }
            }
            Ok(Value::Null)
        }
        Opcode::MotionGotoxy => {
            if let Value::List(coords) = args {
                if coords.len() >= 2 {
                    let x = coords[0].as_number().unwrap_or(0.0);
                    let y = coords[1].as_number().unwrap_or(0.0);
                    if let Some(target) = runtime.current_target_state_mut() {
                        target.go_to(x, y);
                    }
                }
            }
            Ok(Value::Null)
        }
        Opcode::MotionSetX => {
            let x = args.as_number().unwrap_or(0.0);
            if let Some(target) = runtime.current_target_state_mut() {
                target.x = x;
            }
            Ok(Value::Null)
        }
        Opcode::MotionSetY => {
            let y = args.as_number().unwrap_or(0.0);
            if let Some(target) = runtime.current_target_state_mut() {
                target.y = y;
            }
            Ok(Value::Null)
        }
        Opcode::MotionChangeXBy => {
            let dx = args.as_number().unwrap_or(0.0);
            if let Some(target) = runtime.current_target_state_mut() {
                target.x += dx;
            }
            Ok(Value::Null)
        }
        Opcode::MotionChangeYBy => {
            let dy = args.as_number().unwrap_or(0.0);
            if let Some(target) = runtime.current_target_state_mut() {
                target.y += dy;
            }
            Ok(Value::Null)
        }
        Opcode::MotionPointInDirection => {
            let dir = args.as_number().unwrap_or(90.0);
            if let Some(target) = runtime.current_target_state_mut() {
                target.direction = dir;
            }
            Ok(Value::Null)
        }
        Opcode::MotionXPosition => {
            if let Some(target) = runtime.current_target_state() {
                Ok(Value::Number(target.x))
            } else {
                Ok(Value::Number(0.0))
            }
        }
        Opcode::MotionYPosition => {
            if let Some(target) = runtime.current_target_state() {
                Ok(Value::Number(target.y))
            } else {
                Ok(Value::Number(0.0))
            }
        }
        Opcode::MotionDirection => {
            if let Some(target) = runtime.current_target_state() {
                Ok(Value::Number(target.direction))
            } else {
                Ok(Value::Number(90.0))
            }
        }
        Opcode::MotionBounceOffEdge => {
            if let Some(target) = runtime.current_target_state_mut() {
                // Simplified bounce: reverse direction if at edge
                let half_w = 240.0;
                let half_h = 180.0;
                if target.x.abs() > half_w || target.y.abs() > half_h {
                    target.direction = (180.0 - target.direction + 360.0) % 360.0;
                    target.x = target.x.clamp(-half_w, half_w);
                    target.y = target.y.clamp(-half_h, half_h);
                }
            }
            Ok(Value::Null)
        }
        Opcode::MotionSetRotationStyle => {
            // Rotation style is tracked but simplified here
            Ok(Value::Null)
        }
        Opcode::MotionPointTowards
        | Opcode::MotionGlideTo
        | Opcode::MotionGlideSecsToxy => {
            // These would require async/time-based execution
            Ok(Value::Null)
        }
        _ => Err(OpcodeError::UnknownOpcode(format!("{:?}", opcode))),
    }
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
    fn test_motion_forward() {
        let mut runtime = make_runtime();
        execute(&Opcode::MotionForward, &mut runtime, &Value::Number(50.0))
            .expect("should execute");
        let target = runtime.current_target_state().unwrap();
        assert!((target.x - 50.0).abs() < 0.001);
        assert!(target.y.abs() < 0.001);
    }

    #[test]
    fn test_motion_turn_right() {
        let mut runtime = make_runtime();
        execute(&Opcode::MotionTurnRight, &mut runtime, &Value::Number(45.0))
            .expect("should execute");
        let target = runtime.current_target_state().unwrap();
        assert_eq!(target.direction, 135.0);
    }

    #[test]
    fn test_motion_turn_left() {
        let mut runtime = make_runtime();
        execute(&Opcode::MotionTurnLeft, &mut runtime, &Value::Number(30.0))
            .expect("should execute");
        let target = runtime.current_target_state().unwrap();
        assert_eq!(target.direction, 60.0);
    }

    #[test]
    fn test_motion_goto_xy() {
        let mut runtime = make_runtime();
        execute(
            &Opcode::MotionGotoxy,
            &mut runtime,
            &Value::List(vec![Value::Number(100.0), Value::Number(-50.0)]),
        )
        .expect("should execute");
        let target = runtime.current_target_state().unwrap();
        assert_eq!(target.x, 100.0);
        assert_eq!(target.y, -50.0);
    }

    #[test]
    fn test_motion_set_x() {
        let mut runtime = make_runtime();
        execute(&Opcode::MotionSetX, &mut runtime, &Value::Number(42.0))
            .expect("should execute");
        assert_eq!(runtime.current_target_state().unwrap().x, 42.0);
    }

    #[test]
    fn test_motion_set_y() {
        let mut runtime = make_runtime();
        execute(&Opcode::MotionSetY, &mut runtime, &Value::Number(-17.0))
            .expect("should execute");
        assert_eq!(runtime.current_target_state().unwrap().y, -17.0);
    }

    #[test]
    fn test_motion_change_x() {
        let mut runtime = make_runtime();
        execute(&Opcode::MotionChangeXBy, &mut runtime, &Value::Number(10.0))
            .expect("should execute");
        assert_eq!(runtime.current_target_state().unwrap().x, 10.0);
        execute(&Opcode::MotionChangeXBy, &mut runtime, &Value::Number(-5.0))
            .expect("should execute");
        assert_eq!(runtime.current_target_state().unwrap().x, 5.0);
    }

    #[test]
    fn test_motion_change_y() {
        let mut runtime = make_runtime();
        execute(&Opcode::MotionChangeYBy, &mut runtime, &Value::Number(20.0))
            .expect("should execute");
        assert_eq!(runtime.current_target_state().unwrap().y, 20.0);
    }

    #[test]
    fn test_motion_x_position() {
        let mut runtime = make_runtime();
        let result = execute(&Opcode::MotionXPosition, &mut runtime, &Value::Null)
            .expect("should execute");
        assert_eq!(result, Value::Number(0.0));
    }

    #[test]
    fn test_motion_y_position() {
        let mut runtime = make_runtime();
        let result = execute(&Opcode::MotionYPosition, &mut runtime, &Value::Null)
            .expect("should execute");
        assert_eq!(result, Value::Number(0.0));
    }

    #[test]
    fn test_motion_direction() {
        let mut runtime = make_runtime();
        let result = execute(&Opcode::MotionDirection, &mut runtime, &Value::Null)
            .expect("should execute");
        assert_eq!(result, Value::Number(90.0));
    }

    #[test]
    fn test_motion_bounce_off_edge() {
        let mut runtime = make_runtime();
        // Move sprite past the edge
        if let Some(target) = runtime.current_target_state_mut() {
            target.x = 300.0;
        }
        execute(&Opcode::MotionBounceOffEdge, &mut runtime, &Value::Null)
            .expect("should execute");
        let target = runtime.current_target_state().unwrap();
        assert!(target.x <= 240.0);
    }
}
