//! 4-layer priority settings engine for the Sailfish VM.
//!
//! Settings are resolved in priority order:
//! 1. Session (highest) - temporary, per-session overrides
//! 2. Project - saved with the project
//! 3. User - user preferences
//! 4. Defaults (lowest) - built-in defaults

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during settings operations.
#[derive(Error, Debug)]
pub enum SettingsError {
    #[error("invalid layer: {0}")]
    InvalidLayer(String),
    #[error("serialization error: {0}")]
    SerializationError(String),
}

/// The four priority layers for settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SettingsLayer {
    /// Highest priority: temporary per-session overrides.
    Session = 0,
    /// Saved with the project.
    Project = 1,
    /// User preferences.
    User = 2,
    /// Lowest priority: built-in defaults.
    Defaults = 3,
}

/// A settings value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SettingsValue {
    Bool(bool),
    Number(f64),
    String(String),
}

impl SettingsValue {
    /// Try to get a bool value.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            SettingsValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Try to get a number value.
    pub fn as_number(&self) -> Option<f64> {
        match self {
            SettingsValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Try to get a string value.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            SettingsValue::String(s) => Some(s),
            _ => None,
        }
    }
}

impl std::fmt::Display for SettingsValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SettingsValue::Bool(b) => write!(f, "{}", b),
            SettingsValue::Number(n) => write!(f, "{}", n),
            SettingsValue::String(s) => write!(f, "{}", s),
        }
    }
}

/// A callback type for settings change notifications.
type ChangeCallback = Box<dyn Fn(&str, &SettingsValue) + Send + Sync>;

/// The settings engine with 4-layer priority resolution.
pub struct SettingsEngine {
    layers: HashMap<SettingsLayer, HashMap<String, SettingsValue>>,
    subscribers: Vec<ChangeCallback>,
}

impl std::fmt::Debug for SettingsEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SettingsEngine")
            .field("layers", &self.layers)
            .field("subscribers", &format!("{} subscribers", self.subscribers.len()))
            .finish()
    }
}

impl SettingsEngine {
    /// Create a new settings engine with Sailfish defaults.
    pub fn new() -> Self {
        let mut engine = Self {
            layers: HashMap::new(),
            subscribers: Vec::new(),
        };

        // Initialize all layers
        engine
            .layers
            .insert(SettingsLayer::Session, HashMap::new());
        engine
            .layers
            .insert(SettingsLayer::Project, HashMap::new());
        engine.layers.insert(SettingsLayer::User, HashMap::new());
        engine
            .layers
            .insert(SettingsLayer::Defaults, HashMap::new());

        // Set Sailfish defaults
        engine.set_default("turbo_mode", SettingsValue::Bool(false));
        engine.set_default("fps", SettingsValue::Number(30.0));
        engine.set_default("interpolation", SettingsValue::Bool(true));
        engine.set_default("stage_size", SettingsValue::String("480,360".to_string()));

        engine
    }

    /// Set a default value.
    fn set_default(&mut self, key: &str, value: SettingsValue) {
        if let Some(defaults) = self.layers.get_mut(&SettingsLayer::Defaults) {
            defaults.insert(key.to_string(), value);
        }
    }

    /// Get the effective value for a key (resolves through layers).
    pub fn get(&self, key: &str) -> Option<SettingsValue> {
        // Check layers in priority order (Session first, Defaults last)
        for layer in &[
            SettingsLayer::Session,
            SettingsLayer::Project,
            SettingsLayer::User,
            SettingsLayer::Defaults,
        ] {
            if let Some(layer_map) = self.layers.get(layer) {
                if let Some(value) = layer_map.get(key) {
                    return Some(value.clone());
                }
            }
        }
        None
    }

    /// Set a value in a specific layer.
    pub fn set(&mut self, key: &str, value: SettingsValue, layer: SettingsLayer) {
        if let Some(layer_map) = self.layers.get_mut(&layer) {
            layer_map.insert(key.to_string(), value.clone());
        }
        self.notify_subscribers(key, &value);
    }

    /// Remove a value from a specific layer.
    pub fn remove(&mut self, key: &str, layer: SettingsLayer) -> Option<SettingsValue> {
        if let Some(layer_map) = self.layers.get_mut(&layer) {
            layer_map.remove(key)
        } else {
            None
        }
    }

    /// Get the effective value for a key, returning which layer it comes from.
    pub fn effective(&self, key: &str) -> Option<(SettingsValue, SettingsLayer)> {
        for layer in &[
            SettingsLayer::Session,
            SettingsLayer::Project,
            SettingsLayer::User,
            SettingsLayer::Defaults,
        ] {
            if let Some(layer_map) = self.layers.get(layer) {
                if let Some(value) = layer_map.get(key) {
                    return Some((value.clone(), *layer));
                }
            }
        }
        None
    }

    /// Subscribe to settings changes.
    pub fn subscribe(&mut self, callback: impl Fn(&str, &SettingsValue) + Send + Sync + 'static) {
        self.subscribers.push(Box::new(callback));
    }

    /// Notify all subscribers of a settings change.
    fn notify_subscribers(&self, key: &str, value: &SettingsValue) {
        for callback in &self.subscribers {
            callback(key, value);
        }
    }

    /// Export all settings as a serializable map.
    pub fn export(&self) -> HashMap<String, HashMap<String, SettingsValue>> {
        let mut result = HashMap::new();
        for (layer, map) in &self.layers {
            let layer_name = match layer {
                SettingsLayer::Session => "session",
                SettingsLayer::Project => "project",
                SettingsLayer::User => "user",
                SettingsLayer::Defaults => "defaults",
            };
            result.insert(layer_name.to_string(), map.clone());
        }
        result
    }

    /// Import settings from a serializable map.
    pub fn import(&mut self, data: &HashMap<String, HashMap<String, SettingsValue>>) {
        for (layer_name, map) in data {
            let layer = match layer_name.as_str() {
                "session" => SettingsLayer::Session,
                "project" => SettingsLayer::Project,
                "user" => SettingsLayer::User,
                "defaults" => SettingsLayer::Defaults,
                _ => continue,
            };
            if let Some(layer_map) = self.layers.get_mut(&layer) {
                for (key, value) in map {
                    layer_map.insert(key.clone(), value.clone());
                }
            }
        }
    }

    /// Get all keys across all layers.
    pub fn all_keys(&self) -> Vec<String> {
        let mut keys = std::collections::HashSet::new();
        for map in self.layers.values() {
            for key in map.keys() {
                keys.insert(key.clone());
            }
        }
        let mut result: Vec<String> = keys.into_iter().collect();
        result.sort();
        result
    }

    /// Clear all settings in a specific layer.
    pub fn clear_layer(&mut self, layer: SettingsLayer) {
        if let Some(layer_map) = self.layers.get_mut(&layer) {
            layer_map.clear();
        }
    }

    /// Clear all settings in all layers (except defaults).
    pub fn clear_all(&mut self) {
        for layer in &[
            SettingsLayer::Session,
            SettingsLayer::Project,
            SettingsLayer::User,
        ] {
            self.clear_layer(*layer);
        }
    }
}

impl Default for SettingsEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for SettingsEngine {
    fn clone(&self) -> Self {
        Self {
            layers: self.layers.clone(),
            subscribers: Vec::new(), // Can't clone subscribers
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_settings_engine_new() {
        let engine = SettingsEngine::new();
        // Should have Sailfish defaults
        assert_eq!(
            engine.get("turbo_mode"),
            Some(SettingsValue::Bool(false))
        );
        assert_eq!(engine.get("fps"), Some(SettingsValue::Number(30.0)));
        assert_eq!(
            engine.get("interpolation"),
            Some(SettingsValue::Bool(true))
        );
        assert_eq!(
            engine.get("stage_size"),
            Some(SettingsValue::String("480,360".to_string()))
        );
    }

    #[test]
    fn test_settings_layer_priority() {
        let mut engine = SettingsEngine::new();

        // Default value
        assert_eq!(engine.get("fps"), Some(SettingsValue::Number(30.0)));
        assert_eq!(
            engine.effective("fps"),
            Some((SettingsValue::Number(30.0), SettingsLayer::Defaults))
        );

        // User override
        engine.set("fps", SettingsValue::Number(60.0), SettingsLayer::User);
        assert_eq!(engine.get("fps"), Some(SettingsValue::Number(60.0)));
        assert_eq!(
            engine.effective("fps"),
            Some((SettingsValue::Number(60.0), SettingsLayer::User))
        );

        // Project override (higher priority than user)
        engine.set("fps", SettingsValue::Number(45.0), SettingsLayer::Project);
        assert_eq!(engine.get("fps"), Some(SettingsValue::Number(45.0)));
        assert_eq!(
            engine.effective("fps"),
            Some((SettingsValue::Number(45.0), SettingsLayer::Project))
        );

        // Session override (highest priority)
        engine.set("fps", SettingsValue::Number(120.0), SettingsLayer::Session);
        assert_eq!(engine.get("fps"), Some(SettingsValue::Number(120.0)));
        assert_eq!(
            engine.effective("fps"),
            Some((SettingsValue::Number(120.0), SettingsLayer::Session))
        );
    }

    #[test]
    fn test_settings_set_and_get() {
        let mut engine = SettingsEngine::new();

        engine.set("custom_key", SettingsValue::String("custom_value".to_string()), SettingsLayer::Project);
        assert_eq!(
            engine.get("custom_key"),
            Some(SettingsValue::String("custom_value".to_string()))
        );
    }

    #[test]
    fn test_settings_remove() {
        let mut engine = SettingsEngine::new();
        engine.set("temp", SettingsValue::Bool(true), SettingsLayer::Session);
        assert_eq!(engine.get("temp"), Some(SettingsValue::Bool(true)));

        let removed = engine.remove("temp", SettingsLayer::Session);
        assert_eq!(removed, Some(SettingsValue::Bool(true)));
        assert_eq!(engine.get("temp"), None);

        // Remove from wrong layer returns None
        let removed2 = engine.remove("temp", SettingsLayer::User);
        assert_eq!(removed2, None);
    }

    #[test]
    fn test_settings_remove_reveals_lower_layer() {
        let mut engine = SettingsEngine::new();
        engine.set("key", SettingsValue::Number(1.0), SettingsLayer::Defaults);
        engine.set("key", SettingsValue::Number(2.0), SettingsLayer::User);
        engine.set("key", SettingsValue::Number(3.0), SettingsLayer::Session);

        // Session overrides
        assert_eq!(engine.get("key"), Some(SettingsValue::Number(3.0)));

        // Remove session, user shows
        engine.remove("key", SettingsLayer::Session);
        assert_eq!(engine.get("key"), Some(SettingsValue::Number(2.0)));

        // Remove user, defaults show
        engine.remove("key", SettingsLayer::User);
        assert_eq!(engine.get("key"), Some(SettingsValue::Number(1.0)));
    }

    #[test]
    fn test_settings_export_import() {
        let mut engine = SettingsEngine::new();
        engine.set("fps", SettingsValue::Number(60.0), SettingsLayer::User);
        engine.set("custom", SettingsValue::Bool(true), SettingsLayer::Project);

        let exported = engine.export();
        assert!(exported.contains_key("defaults"));
        assert!(exported.contains_key("user"));
        assert!(exported.contains_key("project"));

        // Import into a new engine
        let mut engine2 = SettingsEngine::new();
        engine2.import(&exported);
        assert_eq!(engine2.get("fps"), Some(SettingsValue::Number(60.0)));
        assert_eq!(engine2.get("custom"), Some(SettingsValue::Bool(true)));
    }

    #[test]
    fn test_settings_subscriber() {
        let notifications = Arc::new(Mutex::new(Vec::new()));
        let notifications_clone = notifications.clone();

        let mut engine = SettingsEngine::new();
        engine.subscribe(move |key, value| {
            notifications_clone
                .lock()
                .unwrap()
                .push((key.to_string(), value.clone()));
        });

        engine.set("fps", SettingsValue::Number(60.0), SettingsLayer::User);

        let notifs = notifications.lock().unwrap();
        assert_eq!(notifs.len(), 1);
        assert_eq!(notifs[0].0, "fps");
        assert_eq!(notifs[0].1, SettingsValue::Number(60.0));
    }

    #[test]
    fn test_settings_all_keys() {
        let mut engine = SettingsEngine::new();
        engine.set("custom1", SettingsValue::Bool(true), SettingsLayer::User);
        engine.set("custom2", SettingsValue::Number(5.0), SettingsLayer::Project);

        let keys = engine.all_keys();
        assert!(keys.contains(&"turbo_mode".to_string()));
        assert!(keys.contains(&"fps".to_string()));
        assert!(keys.contains(&"interpolation".to_string()));
        assert!(keys.contains(&"stage_size".to_string()));
        assert!(keys.contains(&"custom1".to_string()));
        assert!(keys.contains(&"custom2".to_string()));
    }

    #[test]
    fn test_settings_clear_layer() {
        let mut engine = SettingsEngine::new();
        engine.set("temp", SettingsValue::Bool(true), SettingsLayer::Session);
        engine.set("temp2", SettingsValue::Bool(false), SettingsLayer::Session);

        engine.clear_layer(SettingsLayer::Session);
        assert_eq!(engine.get("temp"), None);
        assert_eq!(engine.get("temp2"), None);
        // Defaults should still be there
        assert_eq!(engine.get("fps"), Some(SettingsValue::Number(30.0)));
    }

    #[test]
    fn test_settings_clear_all() {
        let mut engine = SettingsEngine::new();
        engine.set("temp", SettingsValue::Bool(true), SettingsLayer::Session);
        engine.set("temp2", SettingsValue::Bool(false), SettingsLayer::Project);
        engine.set("temp3", SettingsValue::Bool(true), SettingsLayer::User);

        engine.clear_all();
        assert_eq!(engine.get("temp"), None);
        assert_eq!(engine.get("temp2"), None);
        assert_eq!(engine.get("temp3"), None);
        // Defaults should still be there
        assert_eq!(engine.get("fps"), Some(SettingsValue::Number(30.0)));
    }

    #[test]
    fn test_settings_value_as_methods() {
        assert_eq!(SettingsValue::Bool(true).as_bool(), Some(true));
        assert_eq!(SettingsValue::Bool(true).as_number(), None);
        assert_eq!(SettingsValue::Number(42.0).as_number(), Some(42.0));
        assert_eq!(SettingsValue::Number(42.0).as_bool(), None);
        assert_eq!(
            SettingsValue::String("hello".to_string()).as_str(),
            Some("hello")
        );
        assert_eq!(SettingsValue::String("hello".to_string()).as_number(), None);
    }

    #[test]
    fn test_settings_layer_ordering() {
        assert!(SettingsLayer::Session < SettingsLayer::Project);
        assert!(SettingsLayer::Project < SettingsLayer::User);
        assert!(SettingsLayer::User < SettingsLayer::Defaults);
    }

    #[test]
    fn test_settings_defaults_not_overwritten() {
        let mut engine = SettingsEngine::new();
        // Setting a higher priority value should not change the default
        engine.set("turbo_mode", SettingsValue::Bool(true), SettingsLayer::User);

        // Effective value is User's true
        assert_eq!(engine.get("turbo_mode"), Some(SettingsValue::Bool(true)));

        // But if we remove the User layer, Default's false shows
        engine.remove("turbo_mode", SettingsLayer::User);
        assert_eq!(engine.get("turbo_mode"), Some(SettingsValue::Bool(false)));
    }
}
