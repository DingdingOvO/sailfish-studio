//! Exporter trait and configuration.
//!
//! Each export format implements the `Exporter` trait.

use std::path::Path;

use crate::{PackagerConfig, PackResult, ProgressCallback, Result};

pub mod apk;
pub mod html;
pub mod media;
pub mod native;
pub mod sfp;
pub mod swf;

pub use apk::ApkExporter;
pub use html::HtmlExporter;
pub use media::{GifExporter, Mp4Exporter, VideoRecordingConfig};
pub use native::NativeExporter;
pub use sfp::SfpExporter;
pub use swf::SwfExporter;

/// Common export configuration shared by all exporters.
#[derive(Debug, Clone)]
pub struct ExportConfig {
    /// Whether to compress output.
    pub compress: bool,
    /// Compression level (0-9 for ZIP).
    pub compression_level: i32,
    /// Whether to embed the runtime.
    pub embed_runtime: bool,
    /// Maximum asset size to inline (in bytes).
    pub max_inline_asset_size: u64,
    /// Whether to minify output where applicable.
    pub minify: bool,
}

impl ExportConfig {
    /// Create from PackagerConfig.
    pub fn from_packager_config(config: &PackagerConfig) -> Self {
        Self {
            compress: true,
            compression_level: config.compression_level,
            embed_runtime: config.embed_runtime,
            max_inline_asset_size: config.max_inline_asset_size,
            minify: config.minify_html,
        }
    }
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self::from_packager_config(&PackagerConfig::default())
    }
}

/// The exporter trait that all export format implementations must satisfy.
pub trait Exporter {
    /// Export a project bundle to the given output path.
    ///
    /// # Arguments
    /// * `bundle` - The project bundle to export
    /// * `output_path` - Path for the output file
    /// * `progress` - Optional progress callback
    fn export(
        &self,
        bundle: &crate::ProjectBundle,
        output_path: &Path,
        progress: Option<ProgressCallback>,
    ) -> Result<PackResult>;

    /// Returns the name of this exporter.
    fn name(&self) -> &str;

    /// Returns the file extension this exporter produces.
    fn extension(&self) -> &str;
}

/// Get the appropriate exporter for a given format.
pub fn get_exporter(format: crate::ExportFormat, config: &PackagerConfig) -> Box<dyn Exporter> {
    match format {
        crate::ExportFormat::Sfp => Box::new(SfpExporter::new(config)),
        crate::ExportFormat::Html => Box::new(HtmlExporter::new(config)),
        crate::ExportFormat::Exe => Box::new(NativeExporter::new(config)),
        crate::ExportFormat::Swf => Box::new(SwfExporter::new(config)),
        crate::ExportFormat::Mp4 => Box::new(Mp4Exporter::new(config)),
        crate::ExportFormat::Gif => Box::new(GifExporter::new(config)),
        crate::ExportFormat::Apk => Box::new(ApkExporter::new(config)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_config_default() {
        let config = ExportConfig::default();
        assert!(config.compress);
        assert_eq!(config.compression_level, 6);
        assert!(config.embed_runtime);
        assert_eq!(config.max_inline_asset_size, 1024 * 1024);
        assert!(!config.minify);
    }

    #[test]
    fn test_export_config_from_packager_config() {
        let mut packager_config = PackagerConfig::default();
        packager_config.compression_level = 9;
        packager_config.minify_html = true;
        let config = ExportConfig::from_packager_config(&packager_config);
        assert_eq!(config.compression_level, 9);
        assert!(config.minify);
    }

    #[test]
    fn test_get_exporter_sfp() {
        let config = PackagerConfig::default();
        let exporter = get_exporter(crate::ExportFormat::Sfp, &config);
        assert_eq!(exporter.name(), "sfp");
        assert_eq!(exporter.extension(), "sfp");
    }

    #[test]
    fn test_get_exporter_html() {
        let config = PackagerConfig::default();
        let exporter = get_exporter(crate::ExportFormat::Html, &config);
        assert_eq!(exporter.name(), "html");
        assert_eq!(exporter.extension(), "html");
    }

    #[test]
    fn test_get_exporter_exe() {
        let config = PackagerConfig::default();
        let exporter = get_exporter(crate::ExportFormat::Exe, &config);
        assert_eq!(exporter.name(), "native");
        assert_eq!(exporter.extension(), "exe");
    }

    #[test]
    fn test_get_exporter_swf() {
        let config = PackagerConfig::default();
        let exporter = get_exporter(crate::ExportFormat::Swf, &config);
        assert_eq!(exporter.name(), "swf");
        assert_eq!(exporter.extension(), "swf");
    }

    #[test]
    fn test_get_exporter_mp4() {
        let config = PackagerConfig::default();
        let exporter = get_exporter(crate::ExportFormat::Mp4, &config);
        assert_eq!(exporter.name(), "mp4");
        assert_eq!(exporter.extension(), "mp4");
    }

    #[test]
    fn test_get_exporter_gif() {
        let config = PackagerConfig::default();
        let exporter = get_exporter(crate::ExportFormat::Gif, &config);
        assert_eq!(exporter.name(), "gif");
        assert_eq!(exporter.extension(), "gif");
    }

    #[test]
    fn test_get_exporter_apk() {
        let config = PackagerConfig::default();
        let exporter = get_exporter(crate::ExportFormat::Apk, &config);
        assert_eq!(exporter.name(), "apk");
        assert_eq!(exporter.extension(), "apk");
    }
}
