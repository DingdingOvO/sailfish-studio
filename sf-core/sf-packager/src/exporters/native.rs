//! Native executable exporter (stub).
//!
//! Full implementation requires the AOT compiler from Phase 5-C.
//! This module defines the interface and returns NotYetImplemented errors.

use std::path::Path;

use crate::{PackagerConfig, PackResult, ProgressCallback, Result};
use crate::exporters::Exporter;

/// Target platform for native executable export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativeTarget {
    /// Windows x86_64
    WindowsX64,
    /// macOS (Apple Silicon)
    MacosArm64,
    /// macOS (Intel)
    MacosX64,
    /// Linux x86_64
    LinuxX64,
}

impl NativeTarget {
    /// Get the default target for the current platform.
    pub fn current() -> Self {
        #[cfg(target_os = "windows")]
        { NativeTarget::WindowsX64 }
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        { NativeTarget::MacosArm64 }
        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        { NativeTarget::MacosX64 }
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        { NativeTarget::LinuxX64 }
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        { NativeTarget::LinuxX64 } // fallback
    }

    /// Get the file extension for this target.
    pub fn extension(&self) -> &str {
        match self {
            NativeTarget::WindowsX64 => "exe",
            NativeTarget::MacosArm64 => "app",
            NativeTarget::MacosX64 => "app",
            NativeTarget::LinuxX64 => "", // no extension on Linux
        }
    }

    /// Get a display name for this target.
    pub fn display_name(&self) -> &str {
        match self {
            NativeTarget::WindowsX64 => "Windows (x86_64)",
            NativeTarget::MacosArm64 => "macOS (Apple Silicon)",
            NativeTarget::MacosX64 => "macOS (Intel)",
            NativeTarget::LinuxX64 => "Linux (x86_64)",
        }
    }

    /// Get all supported targets.
    pub fn all() -> Vec<NativeTarget> {
        vec![
            NativeTarget::WindowsX64,
            NativeTarget::MacosArm64,
            NativeTarget::MacosX64,
            NativeTarget::LinuxX64,
        ]
    }
}

/// Native executable exporter (stub).
pub struct NativeExporter {
    _config: PackagerConfig,
}

impl NativeExporter {
    /// Create a new native exporter.
    pub fn new(config: &PackagerConfig) -> Self {
        Self { _config: config.clone() }
    }
}

impl Exporter for NativeExporter {
    fn export(
        &self,
        _bundle: &crate::ProjectBundle,
        _output_path: &Path,
        _progress: Option<ProgressCallback>,
    ) -> Result<PackResult> {
        Err(crate::PackError::NotYetImplemented(
            "Native executable export requires the AOT compiler (Phase 5-C)".to_string()
        ))
    }

    fn name(&self) -> &str {
        "native"
    }

    fn extension(&self) -> &str {
        "exe"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_bundle::ProjectBundle;

    #[test]
    fn test_native_exporter_returns_not_yet_implemented() {
        let config = PackagerConfig::default();
        let exporter = NativeExporter::new(&config);
        let bundle = ProjectBundle::create_test_bundle("Test");

        let result = exporter.export(&bundle, Path::new("/tmp/test.exe"), None);
        assert!(result.is_err());
        match result {
            Err(crate::PackError::NotYetImplemented(msg)) => {
                assert!(msg.contains("AOT compiler"));
            }
            _ => panic!("expected NotYetImplemented error"),
        }
    }

    #[test]
    fn test_native_exporter_name() {
        let config = PackagerConfig::default();
        let exporter = NativeExporter::new(&config);
        assert_eq!(exporter.name(), "native");
    }

    #[test]
    fn test_native_exporter_extension() {
        let config = PackagerConfig::default();
        let exporter = NativeExporter::new(&config);
        assert_eq!(exporter.extension(), "exe");
    }

    #[test]
    fn test_native_target_extension() {
        assert_eq!(NativeTarget::WindowsX64.extension(), "exe");
        assert_eq!(NativeTarget::MacosArm64.extension(), "app");
        assert_eq!(NativeTarget::MacosX64.extension(), "app");
        assert_eq!(NativeTarget::LinuxX64.extension(), "");
    }

    #[test]
    fn test_native_target_display_name() {
        assert_eq!(NativeTarget::WindowsX64.display_name(), "Windows (x86_64)");
        assert_eq!(NativeTarget::MacosArm64.display_name(), "macOS (Apple Silicon)");
        assert_eq!(NativeTarget::MacosX64.display_name(), "macOS (Intel)");
        assert_eq!(NativeTarget::LinuxX64.display_name(), "Linux (x86_64)");
    }

    #[test]
    fn test_native_target_all() {
        let all = NativeTarget::all();
        assert_eq!(all.len(), 4);
    }

    #[test]
    fn test_native_target_current() {
        let current = NativeTarget::current();
        assert!(NativeTarget::all().contains(&current));
    }
}
