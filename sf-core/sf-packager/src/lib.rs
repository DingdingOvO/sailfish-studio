//! sf-packager: Export Sailfish Studio projects to various formats.
//!
//! Supported export formats:
//! - `.sfp` — Native Sailfish Package (zip with project.sfl + assets + manifest)
//! - `.html` — Self-contained HTML with embedded WASM runtime
//! - `.exe` — Native executable (Windows/macOS/Linux) via AOT
//! - SWF — Adobe Flash format (legacy compatibility)
//! - MP4 — Video recording of project execution
//! - GIF — Animated GIF export
//! - APK — Android package (mobile deployment)

pub mod exporters;
pub mod project_bundle;

use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::{Deserialize, Serialize};

pub use project_bundle::{AssetInfo, Manifest, ProjectBundle};

/// Export format variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    /// Native Sailfish Package (.sfp)
    Sfp,
    /// Self-contained HTML with embedded WASM runtime
    Html,
    /// Native executable (Windows/macOS/Linux)
    Exe,
    /// Adobe Flash SWF format
    Swf,
    /// MP4 video recording
    Mp4,
    /// Animated GIF
    Gif,
    /// Android APK package
    Apk,
}

impl ExportFormat {
    /// Returns the default file extension for this format.
    pub fn extension(&self) -> &str {
        match self {
            ExportFormat::Sfp => "sfp",
            ExportFormat::Html => "html",
            ExportFormat::Exe => "exe",
            ExportFormat::Swf => "swf",
            ExportFormat::Mp4 => "mp4",
            ExportFormat::Gif => "gif",
            ExportFormat::Apk => "apk",
        }
    }

    /// Returns a human-readable name for this format.
    pub fn display_name(&self) -> &str {
        match self {
            ExportFormat::Sfp => "Sailfish Package (.sfp)",
            ExportFormat::Html => "HTML with WASM Runtime",
            ExportFormat::Exe => "Native Executable",
            ExportFormat::Swf => "Adobe Flash (SWF)",
            ExportFormat::Mp4 => "MP4 Video",
            ExportFormat::Gif => "Animated GIF",
            ExportFormat::Apk => "Android APK",
        }
    }

    /// Returns true if this format is fully implemented (not a stub).
    pub fn is_implemented(&self) -> bool {
        matches!(self, ExportFormat::Sfp | ExportFormat::Html)
    }

    /// Parse from a string (case-insensitive).
    pub fn from_str_ignore_case(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "sfp" => Some(ExportFormat::Sfp),
            "html" | "htm" => Some(ExportFormat::Html),
            "exe" | "native" => Some(ExportFormat::Exe),
            "swf" | "flash" => Some(ExportFormat::Swf),
            "mp4" | "video" => Some(ExportFormat::Mp4),
            "gif" | "animated" => Some(ExportFormat::Gif),
            "apk" | "android" => Some(ExportFormat::Apk),
            _ => None,
        }
    }

    /// Detect format from a file extension.
    pub fn from_extension(ext: &str) -> Option<Self> {
        Self::from_str_ignore_case(ext)
    }

    /// Returns all supported export formats.
    pub fn all() -> Vec<ExportFormat> {
        vec![
            ExportFormat::Sfp,
            ExportFormat::Html,
            ExportFormat::Exe,
            ExportFormat::Swf,
            ExportFormat::Mp4,
            ExportFormat::Gif,
            ExportFormat::Apk,
        ]
    }
}

impl std::fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Progress reporting callback type.
/// Receives (stage_name, current_step, total_steps).
pub type ProgressCallback = Box<dyn Fn(&str, usize, usize) + Send + Sync>;

/// Result of a pack operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackResult {
    /// Path to the output file.
    pub output_path: PathBuf,
    /// Size of the output file in bytes.
    pub size_bytes: u64,
    /// Duration of the pack operation in milliseconds.
    pub duration_ms: u64,
    /// SHA256 checksum of the output file (hex-encoded).
    pub checksum: Option<String>,
    /// The export format used.
    pub format: ExportFormat,
    /// Number of assets included.
    pub asset_count: usize,
}

/// Packager error types.
#[derive(Debug, thiserror::Error)]
pub enum PackError {
    /// The project bundle failed validation.
    #[error("validation error: {0}")]
    Validation(String),

    /// An I/O error occurred.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// A ZIP archive error occurred.
    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    /// A JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// The requested format is not yet implemented.
    #[error("not yet implemented: {0}")]
    NotYetImplemented(String),

    /// An asset was not found.
    #[error("asset not found: {0}")]
    AssetNotFound(String),

    /// Invalid path or file format.
    #[error("invalid path: {0}")]
    InvalidPath(String),

    /// The export format is unsupported.
    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),

    /// A generic packager error.
    #[error("{0}")]
    Other(String),
}

/// Convenience type alias.
pub type Result<T> = std::result::Result<T, PackError>;

/// Configuration for the packager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackagerConfig {
    /// Compression level for ZIP-based formats (0-9).
    pub compression_level: i32,
    /// Maximum asset size (in bytes) to inline as base64 in HTML exports.
    pub max_inline_asset_size: u64,
    /// Whether to embed the WASM runtime in HTML exports.
    pub embed_runtime: bool,
    /// Whether to minify HTML output.
    pub minify_html: bool,
    /// Custom title for HTML exports (defaults to project name).
    pub html_title: Option<String>,
    /// Custom icon URL for HTML exports.
    pub html_icon: Option<String>,
    /// Video FPS for media exports.
    pub video_fps: u32,
    /// Video duration in seconds for media exports (0 = auto).
    pub video_duration_secs: u32,
    /// Video resolution (width, height).
    pub video_resolution: (u32, u32),
    /// Package name for APK exports.
    pub apk_package_name: Option<String>,
}

impl Default for PackagerConfig {
    fn default() -> Self {
        Self {
            compression_level: 6,
            max_inline_asset_size: 1024 * 1024, // 1 MB
            embed_runtime: true,
            minify_html: false,
            html_title: None,
            html_icon: None,
            video_fps: 30,
            video_duration_secs: 0,
            video_resolution: (480, 360),
            apk_package_name: None,
        }
    }
}

/// The main packager struct.
pub struct Packager {
    config: PackagerConfig,
}

impl Packager {
    /// Create a new packager with the given configuration.
    pub fn new(config: PackagerConfig) -> Self {
        Self { config }
    }

    /// Create a packager with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(PackagerConfig::default())
    }

    /// Get a reference to the packager configuration.
    pub fn config(&self) -> &PackagerConfig {
        &self.config
    }

    /// Pack a project from a source path to the specified format.
    ///
    /// # Arguments
    /// * `source` - Path to the project directory or .sfp file
    /// * `format` - The export format to use
    /// * `output` - Path for the output file (or directory)
    pub fn pack(
        &self,
        source: &Path,
        format: ExportFormat,
        output: &Path,
    ) -> Result<PackResult> {
        self.pack_with_progress(source, format, output, None)
    }

    /// Pack a project with progress reporting.
    pub fn pack_with_progress(
        &self,
        source: &Path,
        format: ExportFormat,
        output: &Path,
        progress: Option<ProgressCallback>,
    ) -> Result<PackResult> {
        let start = Instant::now();

        // Load the project bundle
        if let Some(ref cb) = progress {
            cb("loading", 0, 4);
        }

        let bundle = if source.extension().and_then(|e| e.to_str()) == Some("sfp") {
            ProjectBundle::load_from_sfp(source)?
        } else {
            ProjectBundle::load_from_directory(source)?
        };

        // Validate the bundle
        if let Some(ref cb) = progress {
            cb("validating", 1, 4);
        }
        bundle.validate()?;

        // Get the appropriate exporter
        if let Some(ref cb) = progress {
            cb("exporting", 2, 4);
        }
        let exporter = exporters::get_exporter(format, &self.config);

        // Export
        let mut result = exporter.export(&bundle, output, progress)?;

        // Finalize
        result.duration_ms = start.elapsed().as_millis() as u64;
        result.format = format;
        result.asset_count = bundle.assets().len();

        Ok(result)
    }

    /// List all supported export formats.
    pub fn supported_formats() -> Vec<ExportFormat> {
        ExportFormat::all()
    }

    /// Detect the export format from an output file path.
    pub fn detect_format_from_path(path: &Path) -> Option<ExportFormat> {
        path.extension()
            .and_then(|e| e.to_str())
            .and_then(ExportFormat::from_extension)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_format_extension() {
        assert_eq!(ExportFormat::Sfp.extension(), "sfp");
        assert_eq!(ExportFormat::Html.extension(), "html");
        assert_eq!(ExportFormat::Exe.extension(), "exe");
        assert_eq!(ExportFormat::Swf.extension(), "swf");
        assert_eq!(ExportFormat::Mp4.extension(), "mp4");
        assert_eq!(ExportFormat::Gif.extension(), "gif");
        assert_eq!(ExportFormat::Apk.extension(), "apk");
    }

    #[test]
    fn test_export_format_display_name() {
        assert_eq!(ExportFormat::Sfp.display_name(), "Sailfish Package (.sfp)");
        assert_eq!(ExportFormat::Html.display_name(), "HTML with WASM Runtime");
    }

    #[test]
    fn test_export_format_from_str_ignore_case() {
        assert_eq!(ExportFormat::from_str_ignore_case("sfp"), Some(ExportFormat::Sfp));
        assert_eq!(ExportFormat::from_str_ignore_case("HTML"), Some(ExportFormat::Html));
        assert_eq!(ExportFormat::from_str_ignore_case("native"), Some(ExportFormat::Exe));
        assert_eq!(ExportFormat::from_str_ignore_case("flash"), Some(ExportFormat::Swf));
        assert_eq!(ExportFormat::from_str_ignore_case("video"), Some(ExportFormat::Mp4));
        assert_eq!(ExportFormat::from_str_ignore_case("animated"), Some(ExportFormat::Gif));
        assert_eq!(ExportFormat::from_str_ignore_case("android"), Some(ExportFormat::Apk));
        assert_eq!(ExportFormat::from_str_ignore_case("unknown"), None);
    }

    #[test]
    fn test_export_format_from_extension() {
        assert_eq!(ExportFormat::from_extension("sfp"), Some(ExportFormat::Sfp));
        assert_eq!(ExportFormat::from_extension("html"), Some(ExportFormat::Html));
        assert_eq!(ExportFormat::from_extension("exe"), Some(ExportFormat::Exe));
        assert_eq!(ExportFormat::from_extension("swf"), Some(ExportFormat::Swf));
        assert_eq!(ExportFormat::from_extension("mp4"), Some(ExportFormat::Mp4));
        assert_eq!(ExportFormat::from_extension("gif"), Some(ExportFormat::Gif));
        assert_eq!(ExportFormat::from_extension("apk"), Some(ExportFormat::Apk));
        assert_eq!(ExportFormat::from_extension("xyz"), None);
    }

    #[test]
    fn test_export_format_is_implemented() {
        assert!(ExportFormat::Sfp.is_implemented());
        assert!(ExportFormat::Html.is_implemented());
        assert!(!ExportFormat::Exe.is_implemented());
        assert!(!ExportFormat::Swf.is_implemented());
        assert!(!ExportFormat::Mp4.is_implemented());
        assert!(!ExportFormat::Gif.is_implemented());
        assert!(!ExportFormat::Apk.is_implemented());
    }

    #[test]
    fn test_export_format_all() {
        let all = ExportFormat::all();
        assert_eq!(all.len(), 7);
    }

    #[test]
    fn test_export_format_display() {
        assert_eq!(format!("{}", ExportFormat::Sfp), "Sailfish Package (.sfp)");
    }

    #[test]
    fn test_export_format_serde() {
        let format = ExportFormat::Sfp;
        let json = serde_json::to_string(&format).unwrap();
        assert_eq!(json, "\"sfp\"");
        let parsed: ExportFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, ExportFormat::Sfp);
    }

    #[test]
    fn test_packager_config_default() {
        let config = PackagerConfig::default();
        assert_eq!(config.compression_level, 6);
        assert_eq!(config.max_inline_asset_size, 1024 * 1024);
        assert!(config.embed_runtime);
        assert!(!config.minify_html);
        assert!(config.html_title.is_none());
        assert!(config.html_icon.is_none());
        assert_eq!(config.video_fps, 30);
        assert_eq!(config.video_duration_secs, 0);
        assert_eq!(config.video_resolution, (480, 360));
        assert!(config.apk_package_name.is_none());
    }

    #[test]
    fn test_packager_config_serde_roundtrip() {
        let config = PackagerConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: PackagerConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.compression_level, parsed.compression_level);
        assert_eq!(config.max_inline_asset_size, parsed.max_inline_asset_size);
        assert_eq!(config.embed_runtime, parsed.embed_runtime);
    }

    #[test]
    fn test_pack_result_fields() {
        let result = PackResult {
            output_path: PathBuf::from("/tmp/test.sfp"),
            size_bytes: 12345,
            duration_ms: 500,
            checksum: Some("abc123".to_string()),
            format: ExportFormat::Sfp,
            asset_count: 5,
        };
        assert_eq!(result.output_path, PathBuf::from("/tmp/test.sfp"));
        assert_eq!(result.size_bytes, 12345);
        assert_eq!(result.duration_ms, 500);
        assert_eq!(result.checksum, Some("abc123".to_string()));
        assert_eq!(result.format, ExportFormat::Sfp);
        assert_eq!(result.asset_count, 5);
    }

    #[test]
    fn test_pack_error_variants() {
        let err = PackError::Validation("test".to_string());
        assert_eq!(format!("{}", err), "validation error: test");

        let err = PackError::NotYetImplemented("EXE".to_string());
        assert_eq!(format!("{}", err), "not yet implemented: EXE");

        let err = PackError::AssetNotFound("sprite.svg".to_string());
        assert_eq!(format!("{}", err), "asset not found: sprite.svg");

        let err = PackError::InvalidPath("bad path".to_string());
        assert_eq!(format!("{}", err), "invalid path: bad path");

        let err = PackError::UnsupportedFormat("xyz".to_string());
        assert_eq!(format!("{}", err), "unsupported format: xyz");

        let err = PackError::Other("generic".to_string());
        assert_eq!(format!("{}", err), "generic");
    }

    #[test]
    fn test_packager_new() {
        let packager = Packager::new(PackagerConfig::default());
        assert_eq!(packager.config().compression_level, 6);
    }

    #[test]
    fn test_packager_with_defaults() {
        let packager = Packager::with_defaults();
        assert_eq!(packager.config().compression_level, 6);
    }

    #[test]
    fn test_supported_formats() {
        let formats = Packager::supported_formats();
        assert_eq!(formats.len(), 7);
        assert!(formats.contains(&ExportFormat::Sfp));
        assert!(formats.contains(&ExportFormat::Html));
    }

    #[test]
    fn test_detect_format_from_path() {
        assert_eq!(
            Packager::detect_format_from_path(Path::new("output.sfp")),
            Some(ExportFormat::Sfp)
        );
        assert_eq!(
            Packager::detect_format_from_path(Path::new("output.html")),
            Some(ExportFormat::Html)
        );
        assert_eq!(
            Packager::detect_format_from_path(Path::new("output.xyz")),
            None
        );
        assert_eq!(
            Packager::detect_format_from_path(Path::new("no_extension")),
            None
        );
    }

    #[test]
    fn test_progress_callback_type() {
        let cb: ProgressCallback = Box::new(|stage, current, total| {
            // Just verify the callback type compiles and can be called
            let _ = (stage, current, total);
        });
        cb("loading", 0, 4);
        cb("exporting", 2, 4);
    }
}
