//! Runtime state management for the Sailfish VM.
//!
//! Manages the execution state of the virtual machine including
//! target states, variables, timers, event queue, and thread states.

use crate::project::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;
use thiserror::Error;

/// Errors that can occur during runtime operations.
#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("target not found: {0}")]
    TargetNotFound(String),
    #[error("variable not found: {0}")]
    VariableNotFound(String),
    #[error("invalid operation: {0}")]
    InvalidOperation(String),
    #[error("runtime not started")]
    NotStarted,
}

/// The overall runtime state of the VM.
#[derive(Debug, Clone)]
pub struct RuntimeState {
    /// Name of the currently active target.
    pub current_target: String,
    /// Map of target names to their states.
    pub targets: HashMap<String, TargetState>,
    /// Global variables (shared across targets).
    pub variables: HashMap<String, Value>,
    /// Event queue for inter-target communication.
    pub event_queue: Vec<RuntimeEvent>,
    /// Whether the runtime is currently running.
    pub running: bool,
    /// Map of thread IDs to their states.
    pub thread_states: HashMap<String, ThreadState>,
    /// When the runtime was started (for timer calculations).
    pub start_time: Option<Instant>,
    /// Clone data for sprite clones.
    pub clones: Vec<CloneData>,
}

/// State of a single target (sprite or stage).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetState {
    /// Target name.
    pub name: String,
    /// X position.
    pub x: f64,
    /// Y position.
    pub y: f64,
    /// Direction in degrees (0=up, 90=right).
    pub direction: f64,
    /// Size as percentage (100 = normal).
    pub size: f64,
    /// Whether the sprite is visible.
    pub visible: bool,
    /// Whether the pen is down.
    pub pen_down: bool,
    /// Pen color as RGB hex string.
    pub pen_color: String,
    /// Current costume index.
    pub current_costume: usize,
    /// Local variables for this target.
    pub variables: HashMap<String, Value>,
    /// Whether this is the stage.
    pub is_stage: bool,
}

/// Events in the runtime event system.
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeEvent {
    /// Green flag was clicked; start the program.
    Start,
    /// Stop all execution.
    Stop,
    /// A broadcast message was sent.
    Broadcast { name: String },
    /// A key was pressed.
    KeyPress { key: String },
    /// The timer was reset.
    TimerReset,
    /// A sprite clone was created.
    CloneCreated { origin_name: String },
    /// A sprite clone was deleted.
    CloneDeleted { origin_name: String },
}

/// State of a single execution thread.
#[derive(Debug, Clone, PartialEq)]
pub enum ThreadState {
    /// Thread is actively running.
    Running,
    /// Thread is yielding (waiting for next frame or timer).
    Yielding,
    /// Thread has stopped.
    Stopped,
}

/// Data for a sprite clone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneData {
    /// Name of the original sprite.
    pub origin_name: String,
    /// X position.
    pub x: f64,
    /// Y position.
    pub y: f64,
    /// Direction.
    pub direction: f64,
    /// Local variables copied from original.
    pub variables: HashMap<String, Value>,
}

impl RuntimeState {
    /// Create a new empty runtime state.
    pub fn new() -> Self {
        Self {
            current_target: String::new(),
            targets: HashMap::new(),
            variables: HashMap::new(),
            event_queue: Vec::new(),
            running: false,
            thread_states: HashMap::new(),
            start_time: None,
            clones: Vec::new(),
        }
    }

    /// Start the runtime.
    pub fn start(&mut self) {
        self.running = true;
        self.start_time = Some(Instant::now());
        self.push_event(RuntimeEvent::Start);
    }

    /// Stop the runtime.
    pub fn stop(&mut self) {
        self.running = false;
        self.start_time = None;
        self.push_event(RuntimeEvent::Stop);
        // Mark all threads as stopped
        for state in self.thread_states.values_mut() {
            *state = ThreadState::Stopped;
        }
    }

    /// Broadcast a message to all targets.
    pub fn broadcast(&mut self, name: &str) {
        self.push_event(RuntimeEvent::Broadcast {
            name: name.to_string(),
        });
    }

    /// Push an event onto the event queue.
    pub fn push_event(&mut self, event: RuntimeEvent) {
        self.event_queue.push(event);
    }

    /// Pop an event from the event queue.
    pub fn pop_event(&mut self) -> Option<RuntimeEvent> {
        if self.event_queue.is_empty() {
            None
        } else {
            Some(self.event_queue.remove(0))
        }
    }

    /// Get a global variable value.
    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    /// Set a global variable value.
    pub fn set_variable(&mut self, name: &str, value: Value) {
        self.variables.insert(name.to_string(), value);
    }

    /// Get a variable from the current target's scope, falling back to global.
    pub fn get_variable_scoped(&self, name: &str) -> Option<&Value> {
        if let Some(target) = self.targets.get(&self.current_target) {
            if let Some(v) = target.variables.get(name) {
                return Some(v);
            }
        }
        self.variables.get(name)
    }

    /// Set a variable in the current target's scope.
    pub fn set_variable_scoped(&mut self, name: &str, value: Value) {
        if let Some(target) = self.targets.get_mut(&self.current_target) {
            target.variables.insert(name.to_string(), value);
        } else {
            self.variables.insert(name.to_string(), value);
        }
    }

    /// Get the elapsed time in seconds since the runtime was started.
    pub fn timer_elapsed(&self) -> f64 {
        match self.start_time {
            Some(start) => start.elapsed().as_secs_f64(),
            None => 0.0,
        }
    }

    /// Reset the timer.
    pub fn timer_reset(&mut self) {
        self.start_time = Some(Instant::now());
        self.push_event(RuntimeEvent::TimerReset);
    }

    /// Add a target state to the runtime.
    pub fn add_target(&mut self, target: TargetState) {
        if self.current_target.is_empty() && !target.is_stage {
            self.current_target = target.name.clone();
        }
        if self.current_target.is_empty() {
            self.current_target = target.name.clone();
        }
        self.targets.insert(target.name.clone(), target);
    }

    /// Get the current target state.
    pub fn current_target_state(&self) -> Option<&TargetState> {
        self.targets.get(&self.current_target)
    }

    /// Get a mutable reference to the current target state.
    pub fn current_target_state_mut(&mut self) -> Option<&mut TargetState> {
        self.targets.get_mut(&self.current_target)
    }

    /// Create a clone of the current target.
    pub fn create_clone(&mut self, origin_name: &str) -> Result<CloneData, RuntimeError> {
        let target = self
            .targets
            .get(origin_name)
            .ok_or_else(|| RuntimeError::TargetNotFound(origin_name.to_string()))?;

        let clone = CloneData {
            origin_name: origin_name.to_string(),
            x: target.x,
            y: target.y,
            direction: target.direction,
            variables: target.variables.clone(),
        };

        self.push_event(RuntimeEvent::CloneCreated {
            origin_name: origin_name.to_string(),
        });
        self.clones.push(clone.clone());
        Ok(clone)
    }

    /// Delete a clone by index.
    pub fn delete_clone(&mut self, index: usize) -> Option<CloneData> {
        if index < self.clones.len() {
            let clone = self.clones.remove(index);
            self.push_event(RuntimeEvent::CloneDeleted {
                origin_name: clone.origin_name.clone(),
            });
            Some(clone)
        } else {
            None
        }
    }

    /// Set a thread's state.
    pub fn set_thread_state(&mut self, thread_id: &str, state: ThreadState) {
        self.thread_states.insert(thread_id.to_string(), state);
    }

    /// Get a thread's state.
    pub fn get_thread_state(&self, thread_id: &str) -> Option<&ThreadState> {
        self.thread_states.get(thread_id)
    }
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self::new()
    }
}

impl TargetState {
    /// Create a new target state for a sprite.
    pub fn new_sprite(name: &str) -> Self {
        Self {
            name: name.to_string(),
            x: 0.0,
            y: 0.0,
            direction: 90.0,
            size: 100.0,
            visible: true,
            pen_down: false,
            pen_color: "#0000ff".to_string(),
            current_costume: 0,
            variables: HashMap::new(),
            is_stage: false,
        }
    }

    /// Create a new target state for the stage.
    pub fn new_stage() -> Self {
        Self {
            name: "Stage".to_string(),
            x: 0.0,
            y: 0.0,
            direction: 90.0,
            size: 100.0,
            visible: true,
            pen_down: false,
            pen_color: "#0000ff".to_string(),
            current_costume: 0,
            variables: HashMap::new(),
            is_stage: true,
        }
    }

    /// Move the target forward by the given number of steps in its current direction.
    pub fn move_forward(&mut self, steps: f64) {
        let rad = self.direction.to_radians();
        self.x += steps * rad.sin();
        self.y += steps * rad.cos();
    }

    /// Turn the target right by the given number of degrees.
    pub fn turn_right(&mut self, degrees: f64) {
        self.direction = (self.direction + degrees) % 360.0;
    }

    /// Turn the target left by the given number of degrees.
    pub fn turn_left(&mut self, degrees: f64) {
        self.direction = (self.direction - degrees + 360.0) % 360.0;
    }

    /// Go to a specific position.
    pub fn go_to(&mut self, x: f64, y: f64) {
        self.x = x;
        self.y = y;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::Value;

    #[test]
    fn test_runtime_state_new() {
        let state = RuntimeState::new();
        assert!(state.current_target.is_empty());
        assert!(state.targets.is_empty());
        assert!(state.variables.is_empty());
        assert!(state.event_queue.is_empty());
        assert!(!state.running);
        assert!(state.start_time.is_none());
    }

    #[test]
    fn test_runtime_start_stop() {
        let mut state = RuntimeState::new();
        state.start();
        assert!(state.running);
        assert!(state.start_time.is_some());
        // Should have a Start event
        assert_eq!(state.event_queue.len(), 1);
        assert_eq!(state.event_queue[0], RuntimeEvent::Start);

        state.stop();
        assert!(!state.running);
        assert!(state.start_time.is_none());
        // Should have a Stop event
        assert_eq!(state.pop_event(), Some(RuntimeEvent::Start));
        assert_eq!(state.pop_event(), Some(RuntimeEvent::Stop));
    }

    #[test]
    fn test_runtime_broadcast() {
        let mut state = RuntimeState::new();
        state.broadcast("message1");
        state.broadcast("message2");

        assert_eq!(state.event_queue.len(), 2);
        assert_eq!(
            state.pop_event(),
            Some(RuntimeEvent::Broadcast {
                name: "message1".to_string()
            })
        );
        assert_eq!(
            state.pop_event(),
            Some(RuntimeEvent::Broadcast {
                name: "message2".to_string()
            })
        );
    }

    #[test]
    fn test_runtime_event_queue() {
        let mut state = RuntimeState::new();
        assert_eq!(state.pop_event(), None);

        state.push_event(RuntimeEvent::KeyPress {
            key: "space".to_string(),
        });
        state.push_event(RuntimeEvent::TimerReset);

        assert_eq!(
            state.pop_event(),
            Some(RuntimeEvent::KeyPress {
                key: "space".to_string()
            })
        );
        assert_eq!(state.pop_event(), Some(RuntimeEvent::TimerReset));
        assert_eq!(state.pop_event(), None);
    }

    #[test]
    fn test_runtime_variables_global() {
        let mut state = RuntimeState::new();
        state.set_variable("score", Value::Number(100.0));
        assert_eq!(state.get_variable("score"), Some(&Value::Number(100.0)));
        assert_eq!(state.get_variable("missing"), None);

        state.set_variable("score", Value::Number(200.0));
        assert_eq!(state.get_variable("score"), Some(&Value::Number(200.0)));
    }

    #[test]
    fn test_runtime_variables_scoped() {
        let mut state = RuntimeState::new();
        state.current_target = "Sprite1".to_string();
        let mut target = TargetState::new_sprite("Sprite1");
        target.variables.insert("x".to_string(), Value::Number(5.0));
        state.add_target(target);

        // Scoped lookup finds target variable first
        assert_eq!(
            state.get_variable_scoped("x"),
            Some(&Value::Number(5.0))
        );

        // Falls back to global
        state.set_variable("y", Value::Number(10.0));
        assert_eq!(
            state.get_variable_scoped("y"),
            Some(&Value::Number(10.0))
        );
    }

    #[test]
    fn test_runtime_timer() {
        let mut state = RuntimeState::new();
        // Timer not started
        assert_eq!(state.timer_elapsed(), 0.0);

        state.start();
        let elapsed = state.timer_elapsed();
        assert!(elapsed >= 0.0);
        assert!(elapsed < 1.0); // Should be very small

        // Reset timer
        state.timer_reset();
        let elapsed2 = state.timer_elapsed();
        assert!(elapsed2 < elapsed + 0.1); // Should be near zero
    }

    #[test]
    fn test_runtime_thread_states() {
        let mut state = RuntimeState::new();
        state.set_thread_state("thread1", ThreadState::Running);
        state.set_thread_state("thread2", ThreadState::Yielding);

        assert_eq!(
            state.get_thread_state("thread1"),
            Some(&ThreadState::Running)
        );
        assert_eq!(
            state.get_thread_state("thread2"),
            Some(&ThreadState::Yielding)
        );
        assert_eq!(state.get_thread_state("thread3"), None);

        state.stop();
        assert_eq!(
            state.get_thread_state("thread1"),
            Some(&ThreadState::Stopped)
        );
    }

    #[test]
    fn test_target_state_new_sprite() {
        let target = TargetState::new_sprite("Cat");
        assert_eq!(target.name, "Cat");
        assert!(!target.is_stage);
        assert_eq!(target.x, 0.0);
        assert_eq!(target.y, 0.0);
        assert_eq!(target.direction, 90.0);
        assert_eq!(target.size, 100.0);
        assert!(target.visible);
        assert!(!target.pen_down);
    }

    #[test]
    fn test_target_state_new_stage() {
        let target = TargetState::new_stage();
        assert_eq!(target.name, "Stage");
        assert!(target.is_stage);
    }

    #[test]
    fn test_target_move_forward() {
        let mut target = TargetState::new_sprite("Cat");
        // Direction 90 means facing right
        target.move_forward(100.0);
        assert!((target.x - 100.0).abs() < 0.001);
        assert!(target.y.abs() < 0.001);
    }

    #[test]
    fn test_target_turn_right() {
        let mut target = TargetState::new_sprite("Cat");
        target.turn_right(45.0);
        assert_eq!(target.direction, 135.0);
    }

    #[test]
    fn test_target_turn_left() {
        let mut target = TargetState::new_sprite("Cat");
        target.turn_left(45.0);
        assert_eq!(target.direction, 45.0);
    }

    #[test]
    fn test_target_go_to() {
        let mut target = TargetState::new_sprite("Cat");
        target.go_to(100.0, -50.0);
        assert_eq!(target.x, 100.0);
        assert_eq!(target.y, -50.0);
    }

    #[test]
    fn test_runtime_add_target() {
        let mut state = RuntimeState::new();
        let stage = TargetState::new_stage();
        state.add_target(stage);
        assert!(state.targets.contains_key("Stage"));
        assert_eq!(state.current_target, "Stage");

        let sprite = TargetState::new_sprite("Cat");
        state.add_target(sprite);
        assert!(state.targets.contains_key("Cat"));
    }

    #[test]
    fn test_runtime_clones() {
        let mut state = RuntimeState::new();
        let mut sprite = TargetState::new_sprite("Cat");
        sprite.x = 50.0;
        sprite.y = 75.0;
        sprite.variables
            .insert("speed".to_string(), Value::Number(3.0));
        state.add_target(sprite);

        let clone = state.create_clone("Cat").expect("should create clone");
        assert_eq!(clone.origin_name, "Cat");
        assert_eq!(clone.x, 50.0);
        assert_eq!(clone.y, 75.0);
        // Check the clone's variables
        assert_eq!(clone.variables.get("speed"), Some(&Value::Number(3.0)));

        assert_eq!(state.clones.len(), 1);

        let deleted = state.delete_clone(0).expect("should delete clone");
        assert_eq!(deleted.origin_name, "Cat");
        assert!(state.clones.is_empty());
    }

    #[test]
    fn test_runtime_clone_not_found() {
        let mut state = RuntimeState::new();
        let result = state.create_clone("NonExistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_runtime_event_equality() {
        assert_eq!(RuntimeEvent::Start, RuntimeEvent::Start);
        assert_eq!(
            RuntimeEvent::Broadcast {
                name: "msg".to_string()
            },
            RuntimeEvent::Broadcast {
                name: "msg".to_string()
            }
        );
        assert_ne!(RuntimeEvent::Start, RuntimeEvent::Stop);
    }
}
