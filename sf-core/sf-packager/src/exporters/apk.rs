//! APK (Android Package) exporter (stub).
//!
//! Full APK generation requires the Android SDK and build tools.
//! This module defines the Android manifest template and stub implementation.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{PackagerConfig, PackResult, ProgressCallback, Result};
use crate::exporters::Exporter;

/// Android manifest template data.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AndroidManifestTemplate {
    /// Package name (e.g., "com.example.myproject").
    pub package_name: String,
    /// Application version code.
    pub version_code: u32,
    /// Application version name.
    pub version_name: String,
    /// Minimum SDK version.
    pub min_sdk_version: u32,
    /// Target SDK version.
    pub target_sdk_version: u32,
    /// Application label (display name).
    pub label: String,
    /// Icon resource name.
    pub icon: String,
    /// Main activity class name.
    pub activity_class: String,
    /// Screen orientation.
    pub orientation: String,
}

impl Default for AndroidManifestTemplate {
    fn default() -> Self {
        Self {
            package_name: "com.sailfish.project".to_string(),
            version_code: 1,
            version_name: "1.0.0".to_string(),
            min_sdk_version: 24, // Android 7.0
            target_sdk_version: 34, // Android 14
            label: "Sailfish Project".to_string(),
            icon: "@mipmap/ic_launcher".to_string(),
            activity_class: "com.sailfish.project.MainActivity".to_string(),
            orientation: "landscape".to_string(),
        }
    }
}

impl AndroidManifestTemplate {
    /// Create a new Android manifest template for a project.
    pub fn new(project_name: &str, package_name: &str) -> Self {
        Self {
            package_name: package_name.to_string(),
            label: project_name.to_string(),
            activity_class: format!("{}.MainActivity", package_name),
            ..Default::default()
        }
    }

    /// Validate the manifest template.
    pub fn validate(&self) -> Result<()> {
        if self.package_name.is_empty() {
            return Err(crate::PackError::Validation("package name is empty".to_string()));
        }
        if !self.package_name.contains('.') {
            return Err(crate::PackError::Validation(
                "package name must contain at least one dot".to_string(),
            ));
        }
        if self.label.is_empty() {
            return Err(crate::PackError::Validation("label is empty".to_string()));
        }
        if self.min_sdk_version > self.target_sdk_version {
            return Err(crate::PackError::Validation(
                "min SDK version cannot exceed target SDK version".to_string(),
            ));
        }
        Ok(())
    }

    /// Generate AndroidManifest.xml content.
    pub fn to_xml(&self) -> String {
        format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android"
    package="{package_name}"
    android:versionCode="{version_code}"
    android:versionName="{version_name}">

    <uses-sdk
        android:minSdkVersion="{min_sdk}"
        android:targetSdkVersion="{target_sdk}" />

    <uses-feature android:glEsVersion="0x00020000" android:required="true" />

    <application
        android:label="{label}"
        android:icon="{icon}"
        android:hardwareAccelerated="true">

        <activity
            android:name="{activity_class}"
            android:screenOrientation="{orientation}"
            android:configChanges="orientation|keyboardHidden|screenSize"
            android:exported="true">
            <intent-filter>
                <action android:name="android.intent.action.MAIN" />
                <category android:name="android.intent.category.LAUNCHER" />
            </intent-filter>
        </activity>
    </application>
</manifest>"#,
            package_name = self.package_name,
            version_code = self.version_code,
            version_name = self.version_name,
            min_sdk = self.min_sdk_version,
            target_sdk = self.target_sdk_version,
            label = self.label,
            icon = self.icon,
            activity_class = self.activity_class,
            orientation = self.orientation,
        )
    }
}

/// APK exporter (stub).
pub struct ApkExporter {
    package_name: String,
}

impl ApkExporter {
    /// Create a new APK exporter.
    pub fn new(config: &PackagerConfig) -> Self {
        Self {
            package_name: config.apk_package_name.clone()
                .unwrap_or_else(|| "com.sailfish.project".to_string()),
        }
    }

    /// Get the configured package name.
    pub fn package_name(&self) -> &str {
        &self.package_name
    }
}

impl Exporter for ApkExporter {
    fn export(
        &self,
        _bundle: &crate::ProjectBundle,
        _output_path: &Path,
        _progress: Option<ProgressCallback>,
    ) -> Result<PackResult> {
        Err(crate::PackError::NotYetImplemented(
            "APK export requires the Android SDK and build tools".to_string(),
        ))
    }

    fn name(&self) -> &str {
        "apk"
    }

    fn extension(&self) -> &str {
        "apk"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_bundle::ProjectBundle;

    // === AndroidManifestTemplate tests ===

    #[test]
    fn test_android_manifest_default() {
        let manifest = AndroidManifestTemplate::default();
        assert_eq!(manifest.package_name, "com.sailfish.project");
        assert_eq!(manifest.version_code, 1);
        assert_eq!(manifest.version_name, "1.0.0");
        assert_eq!(manifest.min_sdk_version, 24);
        assert_eq!(manifest.target_sdk_version, 34);
        assert_eq!(manifest.label, "Sailfish Project");
    }

    #[test]
    fn test_android_manifest_new() {
        let manifest = AndroidManifestTemplate::new("My Game", "com.example.mygame");
        assert_eq!(manifest.label, "My Game");
        assert_eq!(manifest.package_name, "com.example.mygame");
        assert_eq!(manifest.activity_class, "com.example.mygame.MainActivity");
    }

    #[test]
    fn test_android_manifest_validate_valid() {
        let manifest = AndroidManifestTemplate::default();
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_android_manifest_validate_empty_package() {
        let manifest = AndroidManifestTemplate {
            package_name: String::new(),
            ..Default::default()
        };
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_android_manifest_validate_no_dot() {
        let manifest = AndroidManifestTemplate {
            package_name: "nocomponents".to_string(),
            ..Default::default()
        };
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_android_manifest_validate_empty_label() {
        let manifest = AndroidManifestTemplate {
            label: String::new(),
            ..Default::default()
        };
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_android_manifest_validate_min_exceeds_target() {
        let manifest = AndroidManifestTemplate {
            min_sdk_version: 35,
            target_sdk_version: 30,
            ..Default::default()
        };
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_android_manifest_to_xml() {
        let manifest = AndroidManifestTemplate::default();
        let xml = manifest.to_xml();
        assert!(xml.contains("<?xml"));
        assert!(xml.contains("com.sailfish.project"));
        assert!(xml.contains("android.intent.action.MAIN"));
        assert!(xml.contains("LAUNCHER"));
    }

    #[test]
    fn test_android_manifest_to_xml_custom() {
        let manifest = AndroidManifestTemplate::new("My Game", "com.example.game");
        let xml = manifest.to_xml();
        assert!(xml.contains("com.example.game"));
        assert!(xml.contains("My Game"));
    }

    #[test]
    fn test_android_manifest_serde() {
        let manifest = AndroidManifestTemplate::default();
        let json = serde_json::to_string(&manifest).unwrap();
        let parsed: AndroidManifestTemplate = serde_json::from_str(&json).unwrap();
        assert_eq!(manifest, parsed);
    }

    // === APK exporter tests ===

    #[test]
    fn test_apk_exporter_returns_not_yet_implemented() {
        let config = PackagerConfig::default();
        let exporter = ApkExporter::new(&config);
        let bundle = ProjectBundle::create_test_bundle("Test");

        let result = exporter.export(&bundle, Path::new("/tmp/test.apk"), None);
        assert!(result.is_err());
        match result {
            Err(crate::PackError::NotYetImplemented(msg)) => {
                assert!(msg.contains("Android SDK"));
            }
            _ => panic!("expected NotYetImplemented error"),
        }
    }

    #[test]
    fn test_apk_exporter_name_and_extension() {
        let config = PackagerConfig::default();
        let exporter = ApkExporter::new(&config);
        assert_eq!(exporter.name(), "apk");
        assert_eq!(exporter.extension(), "apk");
    }

    #[test]
    fn test_apk_exporter_default_package_name() {
        let config = PackagerConfig::default();
        let exporter = ApkExporter::new(&config);
        assert_eq!(exporter.package_name(), "com.sailfish.project");
    }

    #[test]
    fn test_apk_exporter_custom_package_name() {
        let mut config = PackagerConfig::default();
        config.apk_package_name = Some("com.example.myapp".to_string());
        let exporter = ApkExporter::new(&config);
        assert_eq!(exporter.package_name(), "com.example.myapp");
    }
}
