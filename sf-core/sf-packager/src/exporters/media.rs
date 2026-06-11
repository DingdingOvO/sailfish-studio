//! Media exporter (MP4/GIF stubs).
//!
//! These exporters require actual rendering of the project, which is not
//! yet available. They define configuration structures and stub implementations.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{PackagerConfig, PackResult, ProgressCallback, Result};
use crate::exporters::Exporter;

/// Configuration for video recording exports.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VideoRecordingConfig {
    /// Frames per second for the recording.
    pub fps: u32,
    /// Duration in seconds (0 = auto-detect from project).
    pub duration_secs: u32,
    /// Resolution (width, height).
    pub resolution: (u32, u32),
    /// Whether to include audio.
    pub include_audio: bool,
    /// Video quality (1-100).
    pub quality: u32,
}

impl Default for VideoRecordingConfig {
    fn default() -> Self {
        Self {
            fps: 30,
            duration_secs: 0,
            resolution: (480, 360),
            include_audio: true,
            quality: 75,
        }
    }
}

impl VideoRecordingConfig {
    /// Create a new video recording config.
    pub fn new(fps: u32, width: u32, height: u32) -> Self {
        Self {
            fps,
            duration_secs: 0,
            resolution: (width, height),
            include_audio: true,
            quality: 75,
        }
    }

    /// Validate the configuration.
    pub fn validate(&self) -> Result<()> {
        if self.fps == 0 {
            return Err(crate::PackError::Validation("FPS must be > 0".to_string()));
        }
        if self.resolution.0 == 0 || self.resolution.1 == 0 {
            return Err(crate::PackError::Validation("resolution must be > 0".to_string()));
        }
        if self.quality > 100 {
            return Err(crate::PackError::Validation("quality must be 1-100".to_string()));
        }
        Ok(())
    }

    /// Calculate total number of frames.
    pub fn total_frames(&self) -> Option<u32> {
        if self.duration_secs > 0 {
            Some(self.fps * self.duration_secs)
        } else {
            None // auto-detect
        }
    }

    /// Estimate output file size in bytes (rough approximation).
    pub fn estimated_size_bytes(&self) -> u64 {
        let (w, h) = self.resolution;
        let pixels = w as u64 * h as u64;
        let frames = self.total_frames().unwrap_or(self.fps * 60) as u64; // default 60s
        // Rough: ~0.5 bytes per pixel per frame for compressed video
        (pixels * frames * 500) / 1000
    }
}

/// MP4 video exporter (stub).
pub struct Mp4Exporter {
    config: VideoRecordingConfig,
}

impl Mp4Exporter {
    /// Create a new MP4 exporter.
    pub fn new(packager_config: &PackagerConfig) -> Self {
        Self {
            config: VideoRecordingConfig {
                fps: packager_config.video_fps,
                duration_secs: packager_config.video_duration_secs,
                resolution: packager_config.video_resolution,
                ..Default::default()
            },
        }
    }

    /// Create with custom recording config.
    pub fn with_config(config: VideoRecordingConfig) -> Self {
        Self { config }
    }

    /// Get the recording configuration.
    pub fn recording_config(&self) -> &VideoRecordingConfig {
        &self.config
    }
}

impl Exporter for Mp4Exporter {
    fn export(
        &self,
        _bundle: &crate::ProjectBundle,
        _output_path: &Path,
        _progress: Option<ProgressCallback>,
    ) -> Result<PackResult> {
        Err(crate::PackError::NotYetImplemented(
            "MP4 export requires a rendering pipeline to capture project execution".to_string(),
        ))
    }

    fn name(&self) -> &str {
        "mp4"
    }

    fn extension(&self) -> &str {
        "mp4"
    }
}

/// Animated GIF exporter (stub).
pub struct GifExporter {
    config: VideoRecordingConfig,
}

impl GifExporter {
    /// Create a new GIF exporter.
    pub fn new(packager_config: &PackagerConfig) -> Self {
        Self {
            config: VideoRecordingConfig {
                fps: packager_config.video_fps.min(25), // GIF typically <= 25 fps
                duration_secs: packager_config.video_duration_secs,
                resolution: packager_config.video_resolution,
                include_audio: false, // GIF doesn't support audio
                quality: 75,
            },
        }
    }

    /// Create with custom recording config.
    pub fn with_config(config: VideoRecordingConfig) -> Self {
        Self { config }
    }

    /// Get the recording configuration.
    pub fn recording_config(&self) -> &VideoRecordingConfig {
        &self.config
    }
}

impl Exporter for GifExporter {
    fn export(
        &self,
        _bundle: &crate::ProjectBundle,
        _output_path: &Path,
        _progress: Option<ProgressCallback>,
    ) -> Result<PackResult> {
        Err(crate::PackError::NotYetImplemented(
            "GIF export requires a rendering pipeline to capture project execution".to_string(),
        ))
    }

    fn name(&self) -> &str {
        "gif"
    }

    fn extension(&self) -> &str {
        "gif"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_bundle::ProjectBundle;

    // === VideoRecordingConfig tests ===

    #[test]
    fn test_video_recording_config_default() {
        let config = VideoRecordingConfig::default();
        assert_eq!(config.fps, 30);
        assert_eq!(config.duration_secs, 0);
        assert_eq!(config.resolution, (480, 360));
        assert!(config.include_audio);
        assert_eq!(config.quality, 75);
    }

    #[test]
    fn test_video_recording_config_new() {
        let config = VideoRecordingConfig::new(60, 1920, 1080);
        assert_eq!(config.fps, 60);
        assert_eq!(config.resolution, (1920, 1080));
    }

    #[test]
    fn test_video_recording_config_validate_valid() {
        let config = VideoRecordingConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_video_recording_config_validate_zero_fps() {
        let config = VideoRecordingConfig { fps: 0, ..Default::default() };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_video_recording_config_validate_zero_resolution() {
        let config = VideoRecordingConfig { resolution: (0, 360), ..Default::default() };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_video_recording_config_validate_quality_too_high() {
        let config = VideoRecordingConfig { quality: 101, ..Default::default() };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_video_recording_config_total_frames_auto() {
        let config = VideoRecordingConfig::default(); // duration 0 = auto
        assert!(config.total_frames().is_none());
    }

    #[test]
    fn test_video_recording_config_total_frames_fixed() {
        let config = VideoRecordingConfig { fps: 30, duration_secs: 10, ..Default::default() };
        assert_eq!(config.total_frames(), Some(300));
    }

    #[test]
    fn test_video_recording_config_estimated_size() {
        let config = VideoRecordingConfig { fps: 30, duration_secs: 10, ..Default::default() };
        let size = config.estimated_size_bytes();
        assert!(size > 0);
    }

    #[test]
    fn test_video_recording_config_serde() {
        let config = VideoRecordingConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: VideoRecordingConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, parsed);
    }

    // === MP4 exporter tests ===

    #[test]
    fn test_mp4_exporter_returns_not_yet_implemented() {
        let config = PackagerConfig::default();
        let exporter = Mp4Exporter::new(&config);
        let bundle = ProjectBundle::create_test_bundle("Test");

        let result = exporter.export(&bundle, Path::new("/tmp/test.mp4"), None);
        assert!(result.is_err());
        match result {
            Err(crate::PackError::NotYetImplemented(msg)) => {
                assert!(msg.contains("rendering pipeline"));
            }
            _ => panic!("expected NotYetImplemented error"),
        }
    }

    #[test]
    fn test_mp4_exporter_name_and_extension() {
        let config = PackagerConfig::default();
        let exporter = Mp4Exporter::new(&config);
        assert_eq!(exporter.name(), "mp4");
        assert_eq!(exporter.extension(), "mp4");
    }

    #[test]
    fn test_mp4_exporter_with_config() {
        let rec_config = VideoRecordingConfig::new(60, 1920, 1080);
        let exporter = Mp4Exporter::with_config(rec_config);
        assert_eq!(exporter.recording_config().fps, 60);
    }

    // === GIF exporter tests ===

    #[test]
    fn test_gif_exporter_returns_not_yet_implemented() {
        let config = PackagerConfig::default();
        let exporter = GifExporter::new(&config);
        let bundle = ProjectBundle::create_test_bundle("Test");

        let result = exporter.export(&bundle, Path::new("/tmp/test.gif"), None);
        assert!(result.is_err());
        match result {
            Err(crate::PackError::NotYetImplemented(msg)) => {
                assert!(msg.contains("rendering pipeline"));
            }
            _ => panic!("expected NotYetImplemented error"),
        }
    }

    #[test]
    fn test_gif_exporter_name_and_extension() {
        let config = PackagerConfig::default();
        let exporter = GifExporter::new(&config);
        assert_eq!(exporter.name(), "gif");
        assert_eq!(exporter.extension(), "gif");
    }

    #[test]
    fn test_gif_exporter_caps_fps() {
        let mut packager_config = PackagerConfig::default();
        packager_config.video_fps = 60; // too high for GIF
        let exporter = GifExporter::new(&packager_config);
        assert!(exporter.recording_config().fps <= 25);
    }

    #[test]
    fn test_gif_exporter_no_audio() {
        let config = PackagerConfig::default();
        let exporter = GifExporter::new(&config);
        assert!(!exporter.recording_config().include_audio);
    }

    #[test]
    fn test_gif_exporter_with_config() {
        let rec_config = VideoRecordingConfig::new(15, 320, 240);
        let exporter = GifExporter::with_config(rec_config);
        assert_eq!(exporter.recording_config().fps, 15);
    }
}
