//! Events opcode execution.

use crate::ops::{Opcode, OpcodeError};
use crate::project::Value;
use crate::runtime::{RuntimeEvent, RuntimeState};

/// Execute an events opcode.
pub fn execute(
    opcode: &Opcode,
    runtime: &mut RuntimeState,
    args: &Value,
) -> Result<Value, OpcodeError> {
    match opcode {
        Opcode::EventWhenFlagClicked => {
            runtime.push_event(RuntimeEvent::Start);
            Ok(Value::Null)
        }
        Opcode::EventWhenKeyPressed => {
            let key = args.as_string().unwrap_or_default();
            runtime.push_event(RuntimeEvent::KeyPress { key });
            Ok(Value::Null)
        }
        Opcode::EventBroadcast => {
            let name = args.as_string().unwrap_or_default();
            runtime.push_event(RuntimeEvent::Broadcast { name });
            Ok(Value::Null)
        }
        Opcode::EventBroadcastAndWait => {
            let name = args.as_string().unwrap_or_default();
            runtime.push_event(RuntimeEvent::Broadcast { name });
            // In a full implementation, would wait for all receivers to finish
            Ok(Value::Null)
        }
        Opcode::EventWhenBroadcastReceived => {
            // This is a hat block; execution is triggered by broadcast events
            Ok(Value::Null)
        }
        Opcode::EventWhenBackdropSwitchesTo
        | Opcode::EventWhenGreaterThan
        | Opcode::EventWhenTimerGreaterThan
        | Opcode::EventWhenLoudnessGreaterThan
        | Opcode::EventWhenVideoMotionGreaterThan
        | Opcode::EventWhenCloneCreated
        | Opcode::EventWhenStageClicked
        | Opcode::EventWhenThisSpriteClicked
        | Opcode::EventWhenTouchingObject => {
            // Hat blocks that trigger on specific conditions
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
    fn test_event_when_flag_clicked() {
        let mut runtime = make_runtime();
        execute(&Opcode::EventWhenFlagClicked, &mut runtime, &Value::Null)
            .expect("should execute");
        assert_eq!(runtime.event_queue.len(), 1);
        assert_eq!(runtime.event_queue[0], RuntimeEvent::Start);
    }

    #[test]
    fn test_event_when_key_pressed() {
        let mut runtime = make_runtime();
        execute(
            &Opcode::EventWhenKeyPressed,
            &mut runtime,
            &Value::String("space".to_string()),
        )
        .expect("should execute");
        assert_eq!(
            runtime.pop_event(),
            Some(RuntimeEvent::KeyPress {
                key: "space".to_string()
            })
        );
    }

    #[test]
    fn test_event_broadcast() {
        let mut runtime = make_runtime();
        execute(
            &Opcode::EventBroadcast,
            &mut runtime,
            &Value::String("message1".to_string()),
        )
        .expect("should execute");
        assert_eq!(
            runtime.pop_event(),
            Some(RuntimeEvent::Broadcast {
                name: "message1".to_string()
            })
        );
    }

    #[test]
    fn test_event_broadcast_and_wait() {
        let mut runtime = make_runtime();
        execute(
            &Opcode::EventBroadcastAndWait,
            &mut runtime,
            &Value::String("msg".to_string()),
        )
        .expect("should execute");
        assert_eq!(
            runtime.pop_event(),
            Some(RuntimeEvent::Broadcast {
                name: "msg".to_string()
            })
        );
    }

    #[test]
    fn test_event_broadcast_with_null() {
        let mut runtime = make_runtime();
        execute(&Opcode::EventBroadcast, &mut runtime, &Value::Null).expect("should execute");
        assert_eq!(
            runtime.pop_event(),
            Some(RuntimeEvent::Broadcast {
                name: "".to_string()
            })
        );
    }
}
