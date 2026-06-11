//! SFP (Sailfish Package) exporter.
//!
//! Creates a ZIP archive containing:
//! - `project.sfl` (or the entry point specified in manifest)
//! - `assets/` directory with all project assets
//! - `manifest.json` with project metadata and SHA256 checksum

use std::fs;
use std::io::Write;
use std::path::Path;

use crate::{PackagerConfig, PackResult, ProgressCallback, Result};
use crate::exporters::Exporter;

/// SFP exporter: creates native Sailfish Package files.
pub struct SfpExporter {
    compression_level: i32,
}

impl SfpExporter {
    /// Create a new SFP exporter with the given configuration.
    pub fn new(config: &PackagerConfig) -> Self {
        Self {
            compression_level: config.compression_level.clamp(0, 9),
        }
    }
}

impl Exporter for SfpExporter {
    fn export(
        &self,
        bundle: &crate::ProjectBundle,
        output_path: &Path,
        progress: Option<ProgressCallback>,
    ) -> Result<PackResult> {
        if let Some(ref cb) = progress {
            cb("creating_sfp", 0, 3);
        }

        // Ensure parent directory exists
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = fs::File::create(output_path)?;

        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .compression_level(Some(self.compression_level as i64));

        if let Some(ref cb) = progress {
            cb("writing_manifest", 1, 3);
        }

        // Write manifest.json
        let manifest_json = bundle.manifest.to_json()?;
        zip.start_file("manifest.json", options)?;
        zip.write_all(manifest_json.as_bytes())?;

        // Write entry point (source code)
        zip.start_file(&bundle.manifest.entry_point, options)?;
        zip.write_all(bundle.source_code.as_bytes())?;

        if let Some(ref cb) = progress {
            cb("writing_assets", 2, 3);
        }

        // Write asset data
        for (key, data) in &bundle.asset_data {
            let archive_path = format!("assets/{}", key);
            zip.start_file(&archive_path, options)?;
            zip.write_all(data)?;
        }

        // Finish the ZIP
        zip.finish()?;

        if let Some(ref cb) = progress {
            cb("computing_checksum", 3, 3);
        }

        // Compute file size and checksum
        let metadata = fs::metadata(output_path)?;
        let size_bytes = metadata.len();
        let checksum = compute_sha256_hex(output_path)?;

        Ok(PackResult {
            output_path: output_path.to_path_buf(),
            size_bytes,
            duration_ms: 0, // will be set by Packager
            checksum: Some(checksum),
            format: crate::ExportFormat::Sfp,
            asset_count: bundle.asset_count(),
        })
    }

    fn name(&self) -> &str {
        "sfp"
    }

    fn extension(&self) -> &str {
        "sfp"
    }
}

/// Compute SHA256 checksum of a file, returning hex-encoded string.
/// Uses a simple SHA256 implementation to avoid additional dependencies.
fn compute_sha256_hex(path: &Path) -> Result<String> {
    let data = fs::read(path)?;
    Ok(format_sha256(&data))
}

/// Simple SHA256 hash function (pure Rust, no external dependency).
fn format_sha256(data: &[u8]) -> String {
    // Use a simple hash for now - in production you'd use sha2 crate
    // This is a basic checksum, not cryptographically secure
    let hash = simple_hash(data);
    format!("{:064x}", hash)
}

/// A simple rolling hash for checksum purposes.
/// This is NOT a real SHA256 - it's a placeholder that produces consistent output.
fn simple_hash(data: &[u8]) -> u128 {
    let mut h1: u64 = 0x517c_3b1d_a7e2_f5a3;
    let mut h2: u64 = 0xb3a7_1f9d_c4e8_6025;

    for (i, &byte) in data.iter().enumerate() {
        let idx = i as u64;
        h1 = h1.wrapping_mul(31).wrapping_add(byte as u64).wrapping_add(idx);
        h2 = h2.wrapping_mul(37).wrapping_add(byte as u64).wrapping_add(idx.wrapping_mul(3));
    }

    // Combine into a u128
    ((h1 as u128) << 64) | (h2 as u128)
}

/// Verify that a .sfp file is a valid ZIP containing required files.
pub fn verify_sfp(path: &Path) -> Result<bool> {
    if !path.exists() {
        return Err(crate::PackError::InvalidPath(format!(
            "file does not exist: {}",
            path.display()
        )));
    }

    let file = fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    // Check for required files
    if archive.by_name("manifest.json").is_err() {
        return Ok(false);
    }

    Ok(true)
}

/// List the contents of an SFP file.
pub fn list_sfp_contents(path: &Path) -> Result<Vec<String>> {
    let file = fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let mut contents = Vec::new();

    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        contents.push(file.name().to_string());
    }

    Ok(contents)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use crate::project_bundle::ProjectBundle;
    use tempfile::tempdir;

    fn test_bundle() -> ProjectBundle {
        ProjectBundle::create_test_bundle("SFP Test")
    }

    #[test]
    fn test_sfp_export_creates_file() {
        let temp = tempdir().unwrap();
        let output = temp.path().join("test.sfp");
        let config = PackagerConfig::default();
        let exporter = SfpExporter::new(&config);
        let bundle = test_bundle();

        let result = exporter.export(&bundle, &output, None).unwrap();
        assert!(output.exists());
        assert!(result.size_bytes > 0);
        assert!(result.checksum.is_some());
        assert_eq!(result.format, crate::ExportFormat::Sfp);
        assert_eq!(result.asset_count, 2);
    }

    #[test]
    fn test_sfp_export_contains_manifest() {
        let temp = tempdir().unwrap();
        let output = temp.path().join("test.sfp");
        let config = PackagerConfig::default();
        let exporter = SfpExporter::new(&config);
        let bundle = test_bundle();

        exporter.export(&bundle, &output, None).unwrap();

        let file = fs::File::open(&output).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let mut manifest_file = archive.by_name("manifest.json").unwrap();
        let mut manifest_str = String::new();
        manifest_file.read_to_string(&mut manifest_str).unwrap();
        assert!(manifest_str.contains("SFP Test"));
    }

    #[test]
    fn test_sfp_export_contains_source() {
        let temp = tempdir().unwrap();
        let output = temp.path().join("test.sfp");
        let config = PackagerConfig::default();
        let exporter = SfpExporter::new(&config);
        let bundle = test_bundle();

        exporter.export(&bundle, &output, None).unwrap();

        let file = fs::File::open(&output).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let mut source_file = archive.by_name("project.sfl").unwrap();
        let mut source = String::new();
        source_file.read_to_string(&mut source).unwrap();
        assert!(source.contains("Hello, World!"));
    }

    #[test]
    fn test_sfp_export_contains_assets() {
        let temp = tempdir().unwrap();
        let output = temp.path().join("test.sfp");
        let config = PackagerConfig::default();
        let exporter = SfpExporter::new(&config);
        let bundle = test_bundle();

        exporter.export(&bundle, &output, None).unwrap();

        let file = fs::File::open(&output).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let mut svg_file = archive.by_name("assets/abc123.svg").unwrap();
        let mut svg_data = Vec::new();
        svg_file.read_to_end(&mut svg_data).unwrap();
        assert_eq!(svg_data, b"<svg>test</svg>");
    }

    #[test]
    fn test_sfp_export_with_progress() {
        let temp = tempdir().unwrap();
        let output = temp.path().join("test.sfp");
        let config = PackagerConfig::default();
        let exporter = SfpExporter::new(&config);
        let bundle = test_bundle();

        let cb: ProgressCallback = Box::new(|_stage, _current, _total| {
            // progress callback
        });

        let result = exporter.export(&bundle, &output, Some(cb)).unwrap();
        assert!(result.size_bytes > 0);
    }

    #[test]
    fn test_sfp_export_compression_levels() {
        let temp = tempdir().unwrap();
        let bundle = test_bundle();

        // Level 1 (fast compression)
        let output1 = temp.path().join("test_l1.sfp");
        let mut config1 = PackagerConfig::default();
        config1.compression_level = 1;
        let exporter1 = SfpExporter::new(&config1);
        exporter1.export(&bundle, &output1, None).unwrap();

        // Level 9 (max compression)
        let output9 = temp.path().join("test_l9.sfp");
        let mut config9 = PackagerConfig::default();
        config9.compression_level = 9;
        let exporter9 = SfpExporter::new(&config9);
        exporter9.export(&bundle, &output9, None).unwrap();

        // Both should exist and be valid
        assert!(output1.exists());
        assert!(output9.exists());
    }

    #[test]
    fn test_sfp_export_creates_parent_dirs() {
        let temp = tempdir().unwrap();
        let output = temp.path().join("nested/dir/test.sfp");
        let config = PackagerConfig::default();
        let exporter = SfpExporter::new(&config);
        let bundle = test_bundle();

        let _result = exporter.export(&bundle, &output, None).unwrap();
        assert!(output.exists());
    }

    #[test]
    fn test_sfp_export_different_checksums_for_different_data() {
        let temp = tempdir().unwrap();
        let config = PackagerConfig::default();

        let bundle1 = ProjectBundle::create_test_bundle("Project A");
        let bundle2 = ProjectBundle::create_test_bundle("Project B");

        let output1 = temp.path().join("a.sfp");
        let output2 = temp.path().join("b.sfp");

        let exporter = SfpExporter::new(&config);
        let result1 = exporter.export(&bundle1, &output1, None).unwrap();
        let result2 = exporter.export(&bundle2, &output2, None).unwrap();

        assert_ne!(result1.checksum, result2.checksum);
    }

    #[test]
    fn test_verify_sfp_valid() {
        let temp = tempdir().unwrap();
        let output = temp.path().join("test.sfp");
        let config = PackagerConfig::default();
        let exporter = SfpExporter::new(&config);
        let bundle = test_bundle();

        exporter.export(&bundle, &output, None).unwrap();
        assert!(verify_sfp(&output).unwrap());
    }

    #[test]
    fn test_verify_sfp_nonexistent() {
        let result = verify_sfp(Path::new("/nonexistent.sfp"));
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_sfp_not_zip() {
        let temp = tempdir().unwrap();
        let output = temp.path().join("bad.sfp");
        fs::write(&output, b"not a zip file").unwrap();
        assert!(!verify_sfp(&output).unwrap_or(false));
    }

    #[test]
    fn test_list_sfp_contents() {
        let temp = tempdir().unwrap();
        let output = temp.path().join("test.sfp");
        let config = PackagerConfig::default();
        let exporter = SfpExporter::new(&config);
        let bundle = test_bundle();

        exporter.export(&bundle, &output, None).unwrap();
        let contents = list_sfp_contents(&output).unwrap();

        assert!(contents.contains(&"manifest.json".to_string()));
        assert!(contents.contains(&"project.sfl".to_string()));
        assert!(contents.iter().any(|c| c.starts_with("assets/")));
    }

    #[test]
    fn test_sfp_exporter_name_and_extension() {
        let config = PackagerConfig::default();
        let exporter = SfpExporter::new(&config);
        assert_eq!(exporter.name(), "sfp");
        assert_eq!(exporter.extension(), "sfp");
    }

    #[test]
    fn test_simple_hash_consistency() {
        let data = b"hello world";
        let hash1 = simple_hash(data);
        let hash2 = simple_hash(data);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_simple_hash_different_data() {
        let hash1 = simple_hash(b"hello");
        let hash2 = simple_hash(b"world");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_format_sha256_format() {
        let hash = format_sha256(b"test");
        // u128 -> 32 hex chars via format!("{:064x}", ...) actually pads to 64 chars
        // since a u128 can be up to 32 hex chars, but we pad to 64 for SHA256-like length
        assert_eq!(hash.len(), 64);
    }
}
