//! Looks opcode execution.

use crate::ops::{Opcode, OpcodeError};
use crate::project::Value;
use crate::runtime::RuntimeState;

/// Execute a looks opcode.
pub fn execute(
    opcode: &Opcode,
    runtime: &mut RuntimeState,
    args: &Value,
) -> Result<Value, OpcodeError> {
    match opcode {
        Opcode::LooksSay => {
            // In a real VM, this would display a speech bubble
            // For now, we just acknowledge the operation
            let _msg = args.as_string().unwrap_or_default();
            Ok(Value::Null)
        }
        Opcode::LooksSayForSecs => {
            let _msg = args.as_string().unwrap_or_default();
            // Would need async for timed display
            Ok(Value::Null)
        }
        Opcode::LooksThink => {
            let _msg = args.as_string().unwrap_or_default();
            Ok(Value::Null)
        }
        Opcode::LooksThinkForSecs => {
            let _msg = args.as_string().unwrap_or_default();
            Ok(Value::Null)
        }
        Opcode::LooksSwitchCostumeTo => {
            if let Some(target) = runtime.current_target_state_mut() {
                if let Value::Number(n) = args {
                    target.current_costume = n.max(0.0) as usize;
                } else if let Value::String(name) = args {
                    // Would look up costume by name - simplified
                    let _ = name;
                }
            }
            Ok(Value::Null)
        }
        Opcode::LooksNextCostume => {
            if let Some(target) = runtime.current_target_state_mut() {
                target.current_costume += 1;
            }
            Ok(Value::Null)
        }
        Opcode::LooksShow => {
            if let Some(target) = runtime.current_target_state_mut() {
                target.visible = true;
            }
            Ok(Value::Null)
        }
        Opcode::LooksHide => {
            if let Some(target) = runtime.current_target_state_mut() {
                target.visible = false;
            }
            Ok(Value::Null)
        }
        Opcode::LooksChangeSizeBy => {
            let change = args.as_number().unwrap_or(0.0);
            if let Some(target) = runtime.current_target_state_mut() {
                target.size += change;
            }
            Ok(Value::Null)
        }
        Opcode::LooksSetSizeTo => {
            let size = args.as_number().unwrap_or(100.0);
            if let Some(target) = runtime.current_target_state_mut() {
                target.size = size;
            }
            Ok(Value::Null)
        }
        Opcode::LooksSize => {
            if let Some(target) = runtime.current_target_state() {
                Ok(Value::Number(target.size))
            } else {
                Ok(Value::Number(100.0))
            }
        }
        Opcode::LooksGoToFrontBack => {
            // Z-ordering handled by renderer
            Ok(Value::Null)
        }
        Opcode::LooksGoForwardBackwardLayers => {
            Ok(Value::Null)
        }
        Opcode::LooksCostumeNumberName => {
            if let Some(target) = runtime.current_target_state() {
                Ok(Value::Number(target.current_costume as f64))
            } else {
                Ok(Value::Number(0.0))
            }
        }
        Opcode::LooksSwitchBackdropTo
        | Opcode::LooksNextBackdrop
        | Opcode::LooksBackdropNumberName => {
            // Backdrop operations act on the stage
            Ok(Value::Null)
        }
        Opcode::LooksChangeEffectBy | Opcode::LooksSetEffectTo => {
            // Graphic effects would be handled by the renderer
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
    fn test_looks_show() {
        let mut runtime = make_runtime();
        // First hide
        execute(&Opcode::LooksHide, &mut runtime, &Value::Null).expect("should execute");
        assert!(!runtime.current_target_state().unwrap().visible);
        // Then show
        execute(&Opcode::LooksShow, &mut runtime, &Value::Null).expect("should execute");
        assert!(runtime.current_target_state().unwrap().visible);
    }

    #[test]
    fn test_looks_hide() {
        let mut runtime = make_runtime();
        execute(&Opcode::LooksHide, &mut runtime, &Value::Null).expect("should execute");
        assert!(!runtime.current_target_state().unwrap().visible);
    }

    #[test]
    fn test_looks_change_size() {
        let mut runtime = make_runtime();
        execute(&Opcode::LooksChangeSizeBy, &mut runtime, &Value::Number(50.0))
            .expect("should execute");
        assert_eq!(runtime.current_target_state().unwrap().size, 150.0);
    }

    #[test]
    fn test_looks_set_size() {
        let mut runtime = make_runtime();
        execute(&Opcode::LooksSetSizeTo, &mut runtime, &Value::Number(200.0))
            .expect("should execute");
        assert_eq!(runtime.current_target_state().unwrap().size, 200.0);
    }

    #[test]
    fn test_looks_size() {
        let mut runtime = make_runtime();
        let result = execute(&Opcode::LooksSize, &mut runtime, &Value::Null)
            .expect("should execute");
        assert_eq!(result, Value::Number(100.0));
    }

    #[test]
    fn test_looks_next_costume() {
        let mut runtime = make_runtime();
        execute(&Opcode::LooksNextCostume, &mut runtime, &Value::Null)
            .expect("should execute");
        assert_eq!(runtime.current_target_state().unwrap().current_costume, 1);
    }

    #[test]
    fn test_looks_switch_costume() {
        let mut runtime = make_runtime();
        execute(
            &Opcode::LooksSwitchCostumeTo,
            &mut runtime,
            &Value::Number(3.0),
        )
        .expect("should execute");
        assert_eq!(runtime.current_target_state().unwrap().current_costume, 3);
    }

    #[test]
    fn test_looks_say() {
        let mut runtime = make_runtime();
        let result = execute(
            &Opcode::LooksSay,
            &mut runtime,
            &Value::String("Hello!".to_string()),
        )
        .expect("should execute");
        assert!(result.is_null());
    }

    #[test]
    fn test_looks_think() {
        let mut runtime = make_runtime();
        let result = execute(
            &Opcode::LooksThink,
            &mut runtime,
            &Value::String("Hmm...".to_string()),
        )
        .expect("should execute");
        assert!(result.is_null());
    }

    #[test]
    fn test_looks_costume_number() {
        let mut runtime = make_runtime();
        let result = execute(&Opcode::LooksCostumeNumberName, &mut runtime, &Value::Null)
            .expect("should execute");
        assert_eq!(result, Value::Number(0.0));
    }
}
