//! Sensing opcode execution.

use crate::ops::{Opcode, OpcodeError};
use crate::project::Value;
use crate::runtime::RuntimeState;

/// Execute a sensing opcode.
pub fn execute(
    opcode: &Opcode,
    runtime: &mut RuntimeState,
    args: &Value,
) -> Result<Value, OpcodeError> {
    match opcode {
        Opcode::SensingAskAndWait => {
            // Would prompt user input - simplified
            let _question = args.as_string().unwrap_or_default();
            Ok(Value::Null)
        }
        Opcode::SensingAnswer => {
            // Would return the last user answer
            runtime
                .get_variable("__answer")
                .cloned()
                .or_else(|| Some(Value::String(String::new())))
                .ok_or_else(|| OpcodeError::RuntimeError("no answer".to_string()))
        }
        Opcode::SensingTimer => {
            Ok(Value::Number(runtime.timer_elapsed()))
        }
        Opcode::SensingResetTimer => {
            runtime.timer_reset();
            Ok(Value::Null)
        }
        Opcode::SensingKeyPressed => {
            // Would check actual keyboard state
            let _key = args.as_string().unwrap_or_default();
            Ok(Value::Bool(false))
        }
        Opcode::SensingMouseDown => {
            // Would check actual mouse state
            Ok(Value::Bool(false))
        }
        Opcode::SensingMouseX => {
            // Would return actual mouse position
            runtime
                .get_variable("__mouse_x")
                .and_then(|v| v.as_number())
                .map(Value::Number)
                .unwrap_or(Value::Number(0.0))
                .into_result()
        }
        Opcode::SensingMouseY => {
            runtime
                .get_variable("__mouse_y")
                .and_then(|v| v.as_number())
                .map(Value::Number)
                .unwrap_or(Value::Number(0.0))
                .into_result()
        }
        Opcode::SensingLoudness => {
            // Would check microphone
            Ok(Value::Number(-1.0))
        }
        Opcode::SensingCurrent => {
            let what = args.as_string().unwrap_or_default();
            let val = match what.as_str() {
                "YEAR" => chrono_year(),
                "MONTH" => chrono_month(),
                "DATE" => chrono_date(),
                "DAYOFWEEK" => chrono_day_of_week(),
                "HOUR" => chrono_hour(),
                "MINUTE" => chrono_minute(),
                "SECOND" => chrono_second(),
                _ => 0.0,
            };
            Ok(Value::Number(val))
        }
        Opcode::SensingDaysSince2000 => {
            // Simplified calculation
            Ok(Value::Number(days_since_2000()))
        }
        Opcode::SensingUsername => {
            Ok(Value::String(String::new()))
        }
        Opcode::SensingTouchingObject => {
            let _object = args.as_string().unwrap_or_default();
            Ok(Value::Bool(false))
        }
        Opcode::SensingTouchingColor => {
            Ok(Value::Bool(false))
        }
        Opcode::SensingColorIsTouchingColor => {
            Ok(Value::Bool(false))
        }
        Opcode::SensingDistanceTo => {
            Ok(Value::Number(10000.0))
        }
        Opcode::SensingSetDragMode => {
            Ok(Value::Null)
        }
        Opcode::SensingOf => {
            // Would query another sprite's property
            Ok(Value::Number(0.0))
        }
        _ => Err(OpcodeError::UnknownOpcode(format!("{:?}", opcode))),
    }
}

/// Helper trait to convert Value into a Result.
trait IntoResult {
    fn into_result(self) -> Result<Value, OpcodeError>;
}

impl IntoResult for Value {
    fn into_result(self) -> Result<Value, OpcodeError> {
        Ok(self)
    }
}

// Simplified time functions
fn chrono_year() -> f64 {
    2024.0 // Simplified
}

fn chrono_month() -> f64 {
    1.0
}

fn chrono_date() -> f64 {
    1.0
}

fn chrono_day_of_week() -> f64 {
    1.0
}

fn chrono_hour() -> f64 {
    0.0
}

fn chrono_minute() -> f64 {
    0.0
}

fn chrono_second() -> f64 {
    0.0
}

fn days_since_2000() -> f64 {
    // Simplified - would use actual date calculation
    8765.0
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
    fn test_sensing_timer() {
        let mut runtime = make_runtime();
        runtime.start();
        let result = execute(&Opcode::SensingTimer, &mut runtime, &Value::Null)
            .expect("should execute");
        if let Value::Number(t) = result {
            assert!(t >= 0.0);
        } else {
            panic!("expected number");
        }
    }

    #[test]
    fn test_sensing_reset_timer() {
        let mut runtime = make_runtime();
        runtime.start();
        let result = execute(&Opcode::SensingResetTimer, &mut runtime, &Value::Null)
            .expect("should execute");
        assert!(result.is_null());
        // Timer should now be near zero
        let elapsed = runtime.timer_elapsed();
        assert!(elapsed < 0.1);
    }

    #[test]
    fn test_sensing_ask_and_wait() {
        let mut runtime = make_runtime();
        let result = execute(
            &Opcode::SensingAskAndWait,
            &mut runtime,
            &Value::String("What's your name?".to_string()),
        )
        .expect("should execute");
        assert!(result.is_null());
    }

    #[test]
    fn test_sensing_key_pressed() {
        let mut runtime = make_runtime();
        let result = execute(
            &Opcode::SensingKeyPressed,
            &mut runtime,
            &Value::String("space".to_string()),
        )
        .expect("should execute");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_sensing_mouse_down() {
        let mut runtime = make_runtime();
        let result = execute(&Opcode::SensingMouseDown, &mut runtime, &Value::Null)
            .expect("should execute");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_sensing_mouse_x() {
        let mut runtime = make_runtime();
        let result = execute(&Opcode::SensingMouseX, &mut runtime, &Value::Null)
            .expect("should execute");
        assert_eq!(result, Value::Number(0.0));
    }

    #[test]
    fn test_sensing_mouse_y() {
        let mut runtime = make_runtime();
        let result = execute(&Opcode::SensingMouseY, &mut runtime, &Value::Null)
            .expect("should execute");
        assert_eq!(result, Value::Number(0.0));
    }

    #[test]
    fn test_sensing_current() {
        let mut runtime = make_runtime();
        let result = execute(
            &Opcode::SensingCurrent,
            &mut runtime,
            &Value::String("YEAR".to_string()),
        )
        .expect("should execute");
        if let Value::Number(year) = result {
            assert!(year > 2000.0);
        } else {
            panic!("expected number");
        }
    }

    #[test]
    fn test_sensing_days_since_2000() {
        let mut runtime = make_runtime();
        let result = execute(&Opcode::SensingDaysSince2000, &mut runtime, &Value::Null)
            .expect("should execute");
        if let Value::Number(days) = result {
            assert!(days > 0.0);
        } else {
            panic!("expected number");
        }
    }

    #[test]
    fn test_sensing_touching_object() {
        let mut runtime = make_runtime();
        let result = execute(
            &Opcode::SensingTouchingObject,
            &mut runtime,
            &Value::String("Sprite1".to_string()),
        )
        .expect("should execute");
        assert_eq!(result, Value::Bool(false));
    }
}
