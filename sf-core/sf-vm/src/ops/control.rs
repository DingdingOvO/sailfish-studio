//! Control opcode execution.

use crate::ops::{Opcode, OpcodeError};
use crate::project::Value;
use crate::runtime::{RuntimeState, ThreadState};

/// Execute a control opcode.
pub fn execute(
    opcode: &Opcode,
    runtime: &mut RuntimeState,
    args: &Value,
) -> Result<Value, OpcodeError> {
    match opcode {
        Opcode::ControlWait => {
            // In a real VM, this would yield for the specified duration
            let _duration = args.as_number().unwrap_or(1.0);
            Ok(Value::Null)
        }
        Opcode::ControlRepeat => {
            // Iteration count - the actual loop is handled by the compiler/runtime loop
            let count = args.as_number().unwrap_or(10.0);
            Ok(Value::Number(count.floor()))
        }
        Opcode::ControlForever => {
            // Infinite loop marker - handled by the compiler
            Ok(Value::Null)
        }
        Opcode::ControlIf => {
            // Condition evaluation is handled by the compiler
            let condition = args.as_bool();
            Ok(Value::Bool(condition))
        }
        Opcode::ControlIfElse => {
            let condition = args.as_bool();
            Ok(Value::Bool(condition))
        }
        Opcode::ControlWaitUntil => {
            // Async waiting - handled by runtime loop
            Ok(Value::Null)
        }
        Opcode::ControlRepeatUntil => {
            // Loop condition - handled by runtime loop
            Ok(Value::Null)
        }
        Opcode::ControlStop => {
            // Stop this thread
            runtime.set_thread_state("current", ThreadState::Stopped);
            Ok(Value::Null)
        }
        Opcode::ControlCreateCloneOf => {
            let target_name = args.as_string().unwrap_or_default();
            if !target_name.is_empty() {
                let _ = runtime.create_clone(&target_name);
            }
            Ok(Value::Null)
        }
        Opcode::ControlDeleteThisClone => {
            // Would remove this clone from the runtime
            Ok(Value::Null)
        }
        Opcode::ControlStartAsClone => {
            // Hat block for clone startup
            Ok(Value::Null)
        }
        Opcode::ControlRunWithoutScreenRefresh => {
            // Optimization hint
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
    fn test_control_wait() {
        let mut runtime = make_runtime();
        let result = execute(&Opcode::ControlWait, &mut runtime, &Value::Number(1.5))
            .expect("should execute");
        assert!(result.is_null());
    }

    #[test]
    fn test_control_repeat() {
        let mut runtime = make_runtime();
        let result = execute(&Opcode::ControlRepeat, &mut runtime, &Value::Number(5.0))
            .expect("should execute");
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_control_if_true() {
        let mut runtime = make_runtime();
        let result = execute(&Opcode::ControlIf, &mut runtime, &Value::Bool(true))
            .expect("should execute");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_control_if_false() {
        let mut runtime = make_runtime();
        let result = execute(&Opcode::ControlIf, &mut runtime, &Value::Bool(false))
            .expect("should execute");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_control_if_else_condition() {
        let mut runtime = make_runtime();
        let result = execute(&Opcode::ControlIfElse, &mut runtime, &Value::Bool(true))
            .expect("should execute");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_control_stop() {
        let mut runtime = make_runtime();
        runtime.set_thread_state("current", ThreadState::Running);
        execute(&Opcode::ControlStop, &mut runtime, &Value::Null).expect("should execute");
        assert_eq!(
            runtime.get_thread_state("current"),
            Some(&ThreadState::Stopped)
        );
    }

    #[test]
    fn test_control_create_clone() {
        let mut runtime = make_runtime();
        // Add a target that can be cloned
        runtime.add_target(TargetState::new_sprite("Sprite1"));
        execute(
            &Opcode::ControlCreateCloneOf,
            &mut runtime,
            &Value::String("Sprite1".to_string()),
        )
        .expect("should execute");
        assert_eq!(runtime.clones.len(), 1);
        assert_eq!(runtime.clones[0].origin_name, "Sprite1");
    }

    #[test]
    fn test_control_forever() {
        let mut runtime = make_runtime();
        let result = execute(&Opcode::ControlForever, &mut runtime, &Value::Null)
            .expect("should execute");
        assert!(result.is_null());
    }
}
