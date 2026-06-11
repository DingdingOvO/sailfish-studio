//! Sound opcode execution.

use crate::ops::{Opcode, OpcodeError};
use crate::project::Value;
use crate::runtime::RuntimeState;

/// Execute a sound opcode.
pub fn execute(
    opcode: &Opcode,
    runtime: &mut RuntimeState,
    args: &Value,
) -> Result<Value, OpcodeError> {
    match opcode {
        Opcode::SoundPlay | Opcode::SoundPlayUntilDone => {
            // Sound playback would be handled by the audio subsystem
            let _sound_name = args.as_string().unwrap_or_default();
            Ok(Value::Null)
        }
        Opcode::SoundStopAllSounds => {
            // Would signal the audio subsystem
            Ok(Value::Null)
        }
        Opcode::SoundSetVolumeTo => {
            let volume = args.as_number().unwrap_or(100.0);
            // Store volume as a target variable
            if let Some(target) = runtime.current_target_state_mut() {
                target.variables.insert("__volume".to_string(), Value::Number(volume));
            }
            Ok(Value::Null)
        }
        Opcode::SoundChangeVolumeBy => {
            let change = args.as_number().unwrap_or(0.0);
            if let Some(target) = runtime.current_target_state_mut() {
                let current = target.variables
                    .get("__volume")
                    .and_then(|v| v.as_number())
                    .unwrap_or(100.0);
                target.variables.insert(
                    "__volume".to_string(),
                    Value::Number((current + change).max(0.0)),
                );
            }
            Ok(Value::Null)
        }
        Opcode::SoundVolume => {
            if let Some(target) = runtime.current_target_state() {
                let vol = target.variables
                    .get("__volume")
                    .and_then(|v| v.as_number())
                    .unwrap_or(100.0);
                Ok(Value::Number(vol))
            } else {
                Ok(Value::Number(100.0))
            }
        }
        Opcode::SoundChangeEffectBy | Opcode::SoundSetEffectTo => {
            // Sound effects handled by audio subsystem
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
    fn test_sound_play() {
        let mut runtime = make_runtime();
        let result = execute(
            &Opcode::SoundPlay,
            &mut runtime,
            &Value::String("meow".to_string()),
        )
        .expect("should execute");
        assert!(result.is_null());
    }

    #[test]
    fn test_sound_set_volume() {
        let mut runtime = make_runtime();
        execute(&Opcode::SoundSetVolumeTo, &mut runtime, &Value::Number(50.0))
            .expect("should execute");
        let vol = runtime.current_target_state().unwrap().variables.get("__volume");
        assert_eq!(vol, Some(&Value::Number(50.0)));
    }

    #[test]
    fn test_sound_change_volume() {
        let mut runtime = make_runtime();
        execute(&Opcode::SoundSetVolumeTo, &mut runtime, &Value::Number(80.0))
            .expect("should execute");
        execute(&Opcode::SoundChangeVolumeBy, &mut runtime, &Value::Number(-30.0))
            .expect("should execute");
        let vol = runtime.current_target_state().unwrap().variables.get("__volume");
        assert_eq!(vol, Some(&Value::Number(50.0)));
    }

    #[test]
    fn test_sound_volume_default() {
        let mut runtime = make_runtime();
        let result = execute(&Opcode::SoundVolume, &mut runtime, &Value::Null)
            .expect("should execute");
        assert_eq!(result, Value::Number(100.0));
    }

    #[test]
    fn test_sound_stop_all() {
        let mut runtime = make_runtime();
        let result = execute(&Opcode::SoundStopAllSounds, &mut runtime, &Value::Null)
            .expect("should execute");
        assert!(result.is_null());
    }
}
