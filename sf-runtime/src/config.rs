use serde::{Deserialize, Serialize};

/// Runtime configuration for project execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuntimeConfig {
    /// Target frames per second.
    pub fps: u32,
    /// Stage width in pixels.
    pub stage_width: u32,
    /// Stage height in pixels.
    pub stage_height: u32,
    /// Whether to run in headless mode (no display).
    pub headless: bool,
    /// Turbo mode (run as fast as possible, ignoring fps).
    pub turbo_mode: bool,
    /// Enable interpolation.
    pub interpolation: bool,
    /// Maximum execution time in seconds (0 = unlimited).
    pub max_execution_time: u64,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            fps: 30,
            stage_width: 480,
            stage_height: 360,
            headless: true,
            turbo_mode: false,
            interpolation: true,
            max_execution_time: 0,
        }
    }
}

impl RuntimeConfig {
    /// Create a new runtime config with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a headed runtime config (with display).
    pub fn headed() -> Self {
        Self {
            headless: false,
            ..Self::default()
        }
    }

    /// Create a headless runtime config (no display, for CI/automation).
    pub fn headless() -> Self {
        Self {
            headless: true,
            ..Self::default()
        }
    }

    /// Set fps.
    pub fn with_fps(mut self, fps: u32) -> Self {
        self.fps = fps;
        self
    }

    /// Set stage size.
    pub fn with_stage_size(mut self, width: u32, height: u32) -> Self {
        self.stage_width = width;
        self.stage_height = height;
        self
    }

    /// Set turbo mode.
    pub fn with_turbo(mut self, turbo: bool) -> Self {
        self.turbo_mode = turbo;
        self
    }

    /// Set max execution time.
    pub fn with_max_time(mut self, seconds: u64) -> Self {
        self.max_execution_time = seconds;
        self
    }

    /// Calculate frame duration in milliseconds.
    pub fn frame_duration_ms(&self) -> u64 {
        if self.fps == 0 {
            return 0;
        }
        1000 / self.fps as u64
    }

    /// Load config from a TOML string.
    pub fn from_toml_str(s: &str) -> crate::error::Result<Self> {
        let config: RuntimeConfig = toml::from_str(s)?;
        Ok(config)
    }

    /// Serialize config to TOML string.
    pub fn to_toml_string(&self) -> crate::error::Result<String> {
        Ok(toml::to_string_pretty(self).unwrap_or_default())
    }

    /// Load config from a JSON string.
    pub fn from_json_str(s: &str) -> crate::error::Result<Self> {
        let config: RuntimeConfig = serde_json::from_str(s)?;
        Ok(config)
    }

    /// Serialize config to JSON string.
    pub fn to_json_string(&self) -> crate::error::Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Merge with another config, where `other` takes priority for non-default values.
    pub fn merge(&self, other: &RuntimeConfig) -> RuntimeConfig {
        let defaults = RuntimeConfig::default();
        RuntimeConfig {
            fps: if other.fps != defaults.fps { other.fps } else { self.fps },
            stage_width: if other.stage_width != defaults.stage_width { other.stage_width } else { self.stage_width },
            stage_height: if other.stage_height != defaults.stage_height { other.stage_height } else { self.stage_height },
            headless: other.headless, // headless is always explicit
            turbo_mode: if other.turbo_mode != defaults.turbo_mode { other.turbo_mode } else { self.turbo_mode },
            interpolation: if other.interpolation != defaults.interpolation { other.interpolation } else { self.interpolation },
            max_execution_time: if other.max_execution_time != defaults.max_execution_time { other.max_execution_time } else { self.max_execution_time },
        }
    }

    /// Validate the config values.
    pub fn validate(&self) -> crate::error::Result<()> {
        if self.fps == 0 {
            return Err(crate::error::SfError::Custom("FPS must be greater than 0".to_string()));
        }
        if self.stage_width == 0 {
            return Err(crate::error::SfError::Custom("Stage width must be greater than 0".to_string()));
        }
        if self.stage_height == 0 {
            return Err(crate::error::SfError::Custom("Stage height must be greater than 0".to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RuntimeConfig::default();
        assert_eq!(config.fps, 30);
        assert_eq!(config.stage_width, 480);
        assert_eq!(config.stage_height, 360);
        assert!(config.headless);
        assert!(!config.turbo_mode);
        assert!(config.interpolation);
        assert_eq!(config.max_execution_time, 0);
    }

    #[test]
    fn test_headed_config() {
        let config = RuntimeConfig::headed();
        assert!(!config.headless);
    }

    #[test]
    fn test_headless_config() {
        let config = RuntimeConfig::headless();
        assert!(config.headless);
    }

    #[test]
    fn test_builder_pattern() {
        let config = RuntimeConfig::new()
            .with_fps(60)
            .with_stage_size(960, 720)
            .with_turbo(true)
            .with_max_time(300);
        assert_eq!(config.fps, 60);
        assert_eq!(config.stage_width, 960);
        assert_eq!(config.stage_height, 720);
        assert!(config.turbo_mode);
        assert_eq!(config.max_execution_time, 300);
    }

    #[test]
    fn test_frame_duration() {
        let config = RuntimeConfig::new().with_fps(30);
        assert_eq!(config.frame_duration_ms(), 33);
    }

    #[test]
    fn test_frame_duration_60fps() {
        let config = RuntimeConfig::new().with_fps(60);
        assert_eq!(config.frame_duration_ms(), 16);
    }

    #[test]
    fn test_toml_roundtrip() {
        let config = RuntimeConfig::new().with_fps(60);
        let toml_str = config.to_toml_string().unwrap();
        let parsed = RuntimeConfig::from_toml_str(&toml_str).unwrap();
        assert_eq!(config, parsed);
    }

    #[test]
    fn test_json_roundtrip() {
        let config = RuntimeConfig::new().with_fps(60);
        let json_str = config.to_json_string().unwrap();
        let parsed = RuntimeConfig::from_json_str(&json_str).unwrap();
        assert_eq!(config, parsed);
    }

    #[test]
    fn test_merge() {
        let base = RuntimeConfig::new().with_fps(30);
        let override_config = RuntimeConfig::new().with_fps(60);
        let merged = base.merge(&override_config);
        assert_eq!(merged.fps, 60);
    }

    #[test]
    fn test_merge_keeps_base_when_not_overridden() {
        let base = RuntimeConfig::new().with_fps(60);
        let override_config = RuntimeConfig::default(); // fps = 30 (default)
        let merged = base.merge(&override_config);
        assert_eq!(merged.fps, 60); // Keeps base value since override is default
    }

    #[test]
    fn test_validate_ok() {
        let config = RuntimeConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_zero_fps() {
        let config = RuntimeConfig { fps: 0, ..RuntimeConfig::default() };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_zero_width() {
        let config = RuntimeConfig { stage_width: 0, ..RuntimeConfig::default() };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_zero_height() {
        let config = RuntimeConfig { stage_height: 0, ..RuntimeConfig::default() };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_equality() {
        let c1 = RuntimeConfig::default();
        let c2 = RuntimeConfig::default();
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_inequality() {
        let c1 = RuntimeConfig::default();
        let c2 = RuntimeConfig::headed();
        assert_ne!(c1, c2);
    }
}
