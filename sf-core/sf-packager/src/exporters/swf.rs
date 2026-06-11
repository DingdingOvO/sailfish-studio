//! SWF (Adobe Flash) exporter (stub).
//!
//! Full SWF generation is complex and will be implemented later.
//! This module defines the SWF header structure and stub implementation.

use std::path::Path;

use crate::{PackagerConfig, PackResult, ProgressCallback, Result};
use crate::exporters::Exporter;

/// SWF file signature (magic bytes).
pub const SWF_SIGNATURE_COMPRESSED: &[u8; 3] = b"CWS";
/// SWF file signature (uncompressed).
pub const SWF_SIGNATURE_UNCOMPRESSED: &[u8; 3] = b"FWS";

/// SWF version number this exporter targets.
pub const SWF_VERSION: u8 = 11;

/// SWF file header structure.
#[derive(Debug, Clone, PartialEq)]
pub struct SwfHeader {
    /// Compression type: 'C' (CWS/zlib) or 'F' (FWS/uncompressed).
    pub signature: [u8; 3],
    /// SWF version.
    pub version: u8,
    /// Total file size in bytes.
    pub file_length: u32,
    /// Stage width in twips (1 twip = 1/20 pixel).
    pub frame_size_x_min: i32,
    pub frame_size_x_max: i32,
    pub frame_size_y_min: i32,
    pub frame_size_y_max: i32,
    /// Frame rate (8.8 fixed-point).
    pub frame_rate: u16,
    /// Number of frames.
    pub frame_count: u16,
}

impl SwfHeader {
    /// Create a new SWF header with default values.
    pub fn new(width: u32, height: u32, fps: u32) -> Self {
        Self {
            signature: *SWF_SIGNATURE_COMPRESSED,
            version: SWF_VERSION,
            file_length: 0, // will be computed
            frame_size_x_min: 0,
            frame_size_x_max: (width * 20) as i32, // pixels to twips
            frame_size_y_min: 0,
            frame_size_y_max: (height * 20) as i32,
            frame_rate: ((fps as u16) << 8), // 8.8 fixed-point
            frame_count: 1,
        }
    }

    /// Validate the SWF header.
    pub fn validate(&self) -> Result<()> {
        if self.signature != *SWF_SIGNATURE_COMPRESSED
            && self.signature != *SWF_SIGNATURE_UNCOMPRESSED
        {
            return Err(crate::PackError::Validation(
                "invalid SWF signature".to_string(),
            ));
        }
        if self.version > 50 {
            return Err(crate::PackError::Validation(
                "SWF version too high".to_string(),
            ));
        }
        if self.frame_size_x_max <= self.frame_size_x_min {
            return Err(crate::PackError::Validation(
                "invalid frame width".to_string(),
            ));
        }
        if self.frame_size_y_max <= self.frame_size_y_min {
            return Err(crate::PackError::Validation(
                "invalid frame height".to_string(),
            ));
        }
        Ok(())
    }

    /// Get the stage width in pixels.
    pub fn width(&self) -> u32 {
        ((self.frame_size_x_max - self.frame_size_x_min) / 20) as u32
    }

    /// Get the stage height in pixels.
    pub fn height(&self) -> u32 {
        ((self.frame_size_y_max - self.frame_size_y_min) / 20) as u32
    }

    /// Get the frame rate as an integer.
    pub fn fps(&self) -> u32 {
        (self.frame_rate >> 8) as u32
    }
}

impl Default for SwfHeader {
    fn default() -> Self {
        Self::new(480, 360, 30)
    }
}

/// SWF exporter (stub).
pub struct SwfExporter {
    _config: PackagerConfig,
}

impl SwfExporter {
    /// Create a new SWF exporter.
    pub fn new(config: &PackagerConfig) -> Self {
        Self { _config: config.clone() }
    }
}

impl Exporter for SwfExporter {
    fn export(
        &self,
        _bundle: &crate::ProjectBundle,
        _output_path: &Path,
        _progress: Option<ProgressCallback>,
    ) -> Result<PackResult> {
        Err(crate::PackError::NotYetImplemented(
            "SWF export is not yet implemented (full SWF generation is complex)".to_string(),
        ))
    }

    fn name(&self) -> &str {
        "swf"
    }

    fn extension(&self) -> &str {
        "swf"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_bundle::ProjectBundle;

    #[test]
    fn test_swf_exporter_returns_not_yet_implemented() {
        let config = PackagerConfig::default();
        let exporter = SwfExporter::new(&config);
        let bundle = ProjectBundle::create_test_bundle("Test");

        let result = exporter.export(&bundle, Path::new("/tmp/test.swf"), None);
        assert!(result.is_err());
        match result {
            Err(crate::PackError::NotYetImplemented(msg)) => {
                assert!(msg.contains("SWF"));
            }
            _ => panic!("expected NotYetImplemented error"),
        }
    }

    #[test]
    fn test_swf_exporter_name_and_extension() {
        let config = PackagerConfig::default();
        let exporter = SwfExporter::new(&config);
        assert_eq!(exporter.name(), "swf");
        assert_eq!(exporter.extension(), "swf");
    }

    #[test]
    fn test_swf_header_new() {
        let header = SwfHeader::new(640, 480, 60);
        assert_eq!(header.signature, *SWF_SIGNATURE_COMPRESSED);
        assert_eq!(header.version, SWF_VERSION);
        assert_eq!(header.width(), 640);
        assert_eq!(header.height(), 480);
        assert_eq!(header.fps(), 60);
    }

    #[test]
    fn test_swf_header_default() {
        let header = SwfHeader::default();
        assert_eq!(header.width(), 480);
        assert_eq!(header.height(), 360);
        assert_eq!(header.fps(), 30);
    }

    #[test]
    fn test_swf_header_validate_valid() {
        let header = SwfHeader::default();
        assert!(header.validate().is_ok());
    }

    #[test]
    fn test_swf_header_validate_invalid_signature() {
        let mut header = SwfHeader::default();
        header.signature = *b"XWS";
        assert!(header.validate().is_err());
    }

    #[test]
    fn test_swf_header_validate_invalid_version() {
        let mut header = SwfHeader::default();
        header.version = 100;
        assert!(header.validate().is_err());
    }

    #[test]
    fn test_swf_header_validate_invalid_width() {
        let mut header = SwfHeader::default();
        header.frame_size_x_max = 0;
        assert!(header.validate().is_err());
    }

    #[test]
    fn test_swf_header_validate_invalid_height() {
        let mut header = SwfHeader::default();
        header.frame_size_y_max = 0;
        assert!(header.validate().is_err());
    }

    #[test]
    fn test_swf_header_uncompressed_signature() {
        assert_eq!(SWF_SIGNATURE_UNCOMPRESSED, b"FWS");
    }

    #[test]
    fn test_swf_header_compressed_signature() {
        assert_eq!(SWF_SIGNATURE_COMPRESSED, b"CWS");
    }

    #[test]
    fn test_swf_header_twips_conversion() {
        let header = SwfHeader::new(480, 360, 30);
        assert_eq!(header.frame_size_x_max, 480 * 20);
        assert_eq!(header.frame_size_y_max, 360 * 20);
    }
}
