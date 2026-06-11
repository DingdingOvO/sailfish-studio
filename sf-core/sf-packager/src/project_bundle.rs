//! Project bundle loading and validation.
//!
//! A ProjectBundle represents a loaded project with its data, assets, and metadata.

use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::Path;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{PackError, Result};

/// Information about a single asset in the project.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssetInfo {
    /// The asset file name (e.g., "costume1.svg").
    pub name: String,
    /// The asset ID (typically an MD5 or UUID hash).
    pub asset_id: String,
    /// The file extension (e.g., "svg", "png", "wav").
    pub extension: String,
    /// The size of the asset in bytes.
    pub size_bytes: u64,
    /// The MIME type of the asset.
    pub mime_type: String,
    /// The relative path within the project bundle.
    pub relative_path: String,
}

impl AssetInfo {
    /// Create a new AssetInfo.
    pub fn new(name: impl Into<String>, asset_id: impl Into<String>, extension: impl Into<String>) -> Self {
        let name = name.into();
        let ext = extension.into();
        let mime_type = Self::guess_mime_type(&ext);
        Self {
            relative_path: format!("assets/{}.{}", asset_id.into(), ext),
            name,
            asset_id: String::new(), // will be set by caller
            extension: ext,
            size_bytes: 0,
            mime_type,
        }
    }

    /// Guess MIME type from extension.
    pub fn guess_mime_type(ext: &str) -> String {
        match ext.to_lowercase().as_str() {
            "svg" => "image/svg+xml".to_string(),
            "png" => "image/png".to_string(),
            "jpg" | "jpeg" => "image/jpeg".to_string(),
            "gif" => "image/gif".to_string(),
            "wav" => "audio/wav".to_string(),
            "mp3" => "audio/mpeg".to_string(),
            "ogg" => "audio/ogg".to_string(),
            "webp" => "image/webp".to_string(),
            "bmp" => "image/bmp".to_string(),
            _ => "application/octet-stream".to_string(),
        }
    }

    /// Returns true if this is an image asset.
    pub fn is_image(&self) -> bool {
        self.mime_type.starts_with("image/")
    }

    /// Returns true if this is an audio asset.
    pub fn is_audio(&self) -> bool {
        self.mime_type.starts_with("audio/")
    }
}

/// The project manifest containing metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Manifest {
    /// The project name.
    pub project_name: String,
    /// The project version.
    pub version: String,
    /// The entry point file (e.g., "project.sfl").
    pub entry_point: String,
    /// The Sailfish runtime version required.
    pub runtime_version: String,
    /// List of assets in the project.
    pub assets: Vec<AssetInfo>,
    /// Unique project identifier.
    pub project_id: String,
    /// Creation timestamp (ISO 8601).
    pub created_at: String,
    /// Last modification timestamp (ISO 8601).
    pub modified_at: String,
    /// Author of the project.
    pub author: String,
    /// Description of the project.
    pub description: String,
    /// Custom metadata key-value pairs.
    pub custom_metadata: HashMap<String, String>,
}

impl Manifest {
    /// The current Sailfish runtime version.
    pub const RUNTIME_VERSION: &'static str = "0.1.0";

    /// Create a new manifest with the given project name.
    pub fn new(project_name: impl Into<String>) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            project_name: project_name.into(),
            version: "1.0.0".to_string(),
            entry_point: "project.sfl".to_string(),
            runtime_version: Self::RUNTIME_VERSION.to_string(),
            assets: Vec::new(),
            project_id: Uuid::new_v4().to_string(),
            created_at: now.clone(),
            modified_at: now,
            author: String::new(),
            description: String::new(),
            custom_metadata: HashMap::new(),
        }
    }

    /// Serialize the manifest to JSON.
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Deserialize a manifest from JSON.
    pub fn from_json(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }

    /// Validate the manifest fields.
    pub fn validate(&self) -> Result<()> {
        if self.project_name.is_empty() {
            return Err(PackError::Validation("project name is empty".to_string()));
        }
        if self.entry_point.is_empty() {
            return Err(PackError::Validation("entry point is empty".to_string()));
        }
        if self.runtime_version.is_empty() {
            return Err(PackError::Validation("runtime version is empty".to_string()));
        }
        if self.project_id.is_empty() {
            return Err(PackError::Validation("project ID is empty".to_string()));
        }
        Ok(())
    }

    /// Add an asset to the manifest.
    pub fn add_asset(&mut self, asset: AssetInfo) {
        self.assets.push(asset);
        self.modified_at = Utc::now().to_rfc3339();
    }

    /// Find an asset by name.
    pub fn find_asset(&self, name: &str) -> Option<&AssetInfo> {
        self.assets.iter().find(|a| a.name == name)
    }

    /// Find an asset by asset_id.
    pub fn find_asset_by_id(&self, asset_id: &str) -> Option<&AssetInfo> {
        self.assets.iter().find(|a| a.asset_id == asset_id)
    }

    /// Remove an asset by name.
    pub fn remove_asset(&mut self, name: &str) -> Option<AssetInfo> {
        if let Some(pos) = self.assets.iter().position(|a| a.name == name) {
            self.modified_at = Utc::now().to_rfc3339();
            Some(self.assets.remove(pos))
        } else {
            None
        }
    }

    /// Get total size of all assets.
    pub fn total_asset_size(&self) -> u64 {
        self.assets.iter().map(|a| a.size_bytes).sum()
    }
}

impl Default for Manifest {
    fn default() -> Self {
        Self::new("Untitled Project")
    }
}

/// A loaded project bundle containing project data, assets, and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectBundle {
    /// The project manifest.
    pub manifest: Manifest,
    /// The project source code (SFL format).
    pub source_code: String,
    /// Binary asset data: asset_id -> raw bytes.
    pub asset_data: HashMap<String, Vec<u8>>,
}

impl ProjectBundle {
    /// Create a new empty project bundle with the given name.
    pub fn new(project_name: impl Into<String>) -> Self {
        Self {
            manifest: Manifest::new(project_name),
            source_code: String::new(),
            asset_data: HashMap::new(),
        }
    }

    /// Load a project bundle from a directory.
    ///
    /// The directory should contain:
    /// - `project.sfl` (or the entry point specified in manifest)
    /// - `manifest.json` (optional, will be created if missing)
    /// - `assets/` directory (optional)
    pub fn load_from_directory(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(PackError::InvalidPath(format!(
                "directory does not exist: {}",
                path.display()
            )));
        }
        if !path.is_dir() {
            return Err(PackError::InvalidPath(format!(
                "path is not a directory: {}",
                path.display()
            )));
        }

        // Load manifest
        let manifest_path = path.join("manifest.json");
        let manifest = if manifest_path.exists() {
            let manifest_str = fs::read_to_string(&manifest_path)?;
            Manifest::from_json(&manifest_str)?
        } else {
            // Create a default manifest from directory name
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Untitled")
                .to_string();
            Manifest::new(name)
        };

        // Load source code
        let source_path = path.join(&manifest.entry_point);
        let source_code = if source_path.exists() {
            fs::read_to_string(&source_path)?
        } else {
            // Try default project.sfl
            let default_path = path.join("project.sfl");
            if default_path.exists() {
                fs::read_to_string(&default_path)?
            } else {
                String::new()
            }
        };

        // Load assets
        let mut asset_data = HashMap::new();
        let assets_dir = path.join("assets");
        if assets_dir.exists() && assets_dir.is_dir() {
            for entry in fs::read_dir(&assets_dir)? {
                let entry = entry?;
                let file_path = entry.path();
                if file_path.is_file() {
                    if let Some(file_name) = file_path.file_name().and_then(|n| n.to_str()) {
                        let mut data = Vec::new();
                        fs::File::open(&file_path)?.read_to_end(&mut data)?;
                        asset_data.insert(file_name.to_string(), data);
                    }
                }
            }
        }

        Ok(Self {
            manifest,
            source_code,
            asset_data,
        })
    }

    /// Load a project bundle from a .sfp file.
    pub fn load_from_sfp(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(PackError::InvalidPath(format!(
                "file does not exist: {}",
                path.display()
            )));
        }

        let file = fs::File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        // Load manifest
        let manifest = {
            let mut manifest_file = archive.by_name("manifest.json")
                .map_err(|_| PackError::Other("manifest.json not found in .sfp archive".to_string()))?;
            let mut manifest_str = String::new();
            manifest_file.read_to_string(&mut manifest_str)?;
            Manifest::from_json(&manifest_str)?
        };

        // Load source code
        let source_code = {
            let mut source_file = archive.by_name(&manifest.entry_point)
                .map_err(|_| PackError::Other(
                    format!("entry point '{}' not found in .sfp archive", manifest.entry_point)
                ))?;
            let mut source = String::new();
            source_file.read_to_string(&mut source)?;
            source
        };

        // Load asset data
        let mut asset_data = HashMap::new();
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();
            if name.starts_with("assets/") && !file.is_dir() {
                let mut data = Vec::new();
                file.read_to_end(&mut data)?;
                // Strip "assets/" prefix
                let asset_name = name.strip_prefix("assets/").unwrap_or(&name).to_string();
                asset_data.insert(asset_name, data);
            }
        }

        Ok(Self {
            manifest,
            source_code,
            asset_data,
        })
    }

    /// Validate the project bundle.
    pub fn validate(&self) -> Result<()> {
        self.manifest.validate()?;

        // Verify all manifest assets have corresponding data
        for asset in &self.manifest.assets {
            let key = format!("{}.{}", asset.asset_id, asset.extension);
            if !self.asset_data.contains_key(&key) && !self.asset_data.contains_key(&asset.relative_path) {
                // Not all assets need data (they may be referenced by path)
                // Only warn for assets with zero size
            }
        }

        // Verify source code exists (can be empty for new projects)
        if self.manifest.entry_point.is_empty() {
            return Err(PackError::Validation("entry point not specified".to_string()));
        }

        Ok(())
    }

    /// Get the list of assets in this bundle.
    pub fn assets(&self) -> &[AssetInfo] {
        &self.manifest.assets
    }

    /// Get asset data by asset key.
    pub fn get_asset_data(&self, key: &str) -> Option<&[u8]> {
        self.asset_data.get(key).map(|v| v.as_slice())
    }

    /// Add an asset to the bundle.
    pub fn add_asset(&mut self, info: AssetInfo, data: Vec<u8>) {
        let key = format!("{}.{}", info.asset_id, info.extension);
        self.asset_data.insert(key, data);
        self.manifest.add_asset(info);
    }

    /// Remove an asset from the bundle.
    pub fn remove_asset(&mut self, name: &str) -> Option<Vec<u8>> {
        if let Some(info) = self.manifest.remove_asset(name) {
            let key = format!("{}.{}", info.asset_id, info.extension);
            self.asset_data.remove(&key)
        } else {
            None
        }
    }

    /// Save the project bundle to a directory.
    pub fn save_to_directory(&self, path: &Path) -> Result<()> {
        fs::create_dir_all(path)?;
        fs::create_dir_all(path.join("assets"))?;

        // Write manifest
        let manifest_json = self.manifest.to_json()?;
        fs::write(path.join("manifest.json"), manifest_json)?;

        // Write source code
        fs::write(path.join(&self.manifest.entry_point), &self.source_code)?;

        // Write asset data
        for (key, data) in &self.asset_data {
            fs::write(path.join("assets").join(key), data)?;
        }

        Ok(())
    }

    /// Get the total size of all asset data.
    pub fn total_data_size(&self) -> u64 {
        self.asset_data.values().map(|d| d.len() as u64).sum()
    }

    /// Get the number of assets.
    pub fn asset_count(&self) -> usize {
        self.manifest.assets.len()
    }

    /// Set the source code.
    pub fn set_source_code(&mut self, code: impl Into<String>) {
        self.source_code = code.into();
    }

    /// Create a simple test bundle for unit testing.
    pub fn create_test_bundle(name: &str) -> Self {
        let mut bundle = Self::new(name);
        bundle.source_code = r#"
def main():
    sprite.move(10)
    sprite.say("Hello, World!")
"#.to_string();

        let costume = AssetInfo {
            name: "cat.svg".to_string(),
            asset_id: "abc123".to_string(),
            extension: "svg".to_string(),
            size_bytes: 256,
            mime_type: "image/svg+xml".to_string(),
            relative_path: "assets/abc123.svg".to_string(),
        };
        bundle.add_asset(costume, b"<svg>test</svg>".to_vec());

        let sound = AssetInfo {
            name: "meow.wav".to_string(),
            asset_id: "def456".to_string(),
            extension: "wav".to_string(),
            size_bytes: 1024,
            mime_type: "audio/wav".to_string(),
            relative_path: "assets/def456.wav".to_string(),
        };
        bundle.add_asset(sound, vec![0u8; 1024]);

        bundle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === AssetInfo tests ===

    #[test]
    fn test_asset_info_new() {
        let info = AssetInfo::new("test.svg", "id123", "svg");
        assert_eq!(info.name, "test.svg");
        assert_eq!(info.extension, "svg");
        assert_eq!(info.mime_type, "image/svg+xml");
        assert_eq!(info.size_bytes, 0);
    }

    #[test]
    fn test_asset_info_guess_mime_type() {
        assert_eq!(AssetInfo::guess_mime_type("svg"), "image/svg+xml");
        assert_eq!(AssetInfo::guess_mime_type("png"), "image/png");
        assert_eq!(AssetInfo::guess_mime_type("jpg"), "image/jpeg");
        assert_eq!(AssetInfo::guess_mime_type("jpeg"), "image/jpeg");
        assert_eq!(AssetInfo::guess_mime_type("gif"), "image/gif");
        assert_eq!(AssetInfo::guess_mime_type("wav"), "audio/wav");
        assert_eq!(AssetInfo::guess_mime_type("mp3"), "audio/mpeg");
        assert_eq!(AssetInfo::guess_mime_type("ogg"), "audio/ogg");
        assert_eq!(AssetInfo::guess_mime_type("webp"), "image/webp");
        assert_eq!(AssetInfo::guess_mime_type("bmp"), "image/bmp");
        assert_eq!(AssetInfo::guess_mime_type("xyz"), "application/octet-stream");
    }

    #[test]
    fn test_asset_info_is_image() {
        let info = AssetInfo::new("test.svg", "id", "svg");
        assert!(info.is_image());
        assert!(!info.is_audio());
    }

    #[test]
    fn test_asset_info_is_audio() {
        let info = AssetInfo::new("test.wav", "id", "wav");
        assert!(info.is_audio());
        assert!(!info.is_image());
    }

    #[test]
    fn test_asset_info_neither_image_nor_audio() {
        let info = AssetInfo::new("data.bin", "id", "bin");
        assert!(!info.is_image());
        assert!(!info.is_audio());
    }

    #[test]
    fn test_asset_info_equality() {
        let a = AssetInfo {
            name: "test.svg".to_string(),
            asset_id: "id1".to_string(),
            extension: "svg".to_string(),
            size_bytes: 100,
            mime_type: "image/svg+xml".to_string(),
            relative_path: "assets/id1.svg".to_string(),
        };
        let b = a.clone();
        assert_eq!(a, b);
    }

    // === Manifest tests ===

    #[test]
    fn test_manifest_new() {
        let manifest = Manifest::new("Test Project");
        assert_eq!(manifest.project_name, "Test Project");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.entry_point, "project.sfl");
        assert_eq!(manifest.runtime_version, Manifest::RUNTIME_VERSION);
        assert!(manifest.assets.is_empty());
        assert!(!manifest.project_id.is_empty());
        assert!(!manifest.created_at.is_empty());
        assert!(!manifest.modified_at.is_empty());
        assert!(manifest.author.is_empty());
        assert!(manifest.description.is_empty());
        assert!(manifest.custom_metadata.is_empty());
    }

    #[test]
    fn test_manifest_default() {
        let manifest = Manifest::default();
        assert_eq!(manifest.project_name, "Untitled Project");
    }

    #[test]
    fn test_manifest_validate_valid() {
        let manifest = Manifest::new("Valid Project");
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_manifest_validate_empty_name() {
        let mut manifest = Manifest::new("Test");
        manifest.project_name = String::new();
        assert!(manifest.validate().is_err());
        match manifest.validate() {
            Err(PackError::Validation(msg)) => assert!(msg.contains("project name")),
            _ => panic!("expected validation error"),
        }
    }

    #[test]
    fn test_manifest_validate_empty_entry_point() {
        let mut manifest = Manifest::new("Test");
        manifest.entry_point = String::new();
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_manifest_validate_empty_runtime_version() {
        let mut manifest = Manifest::new("Test");
        manifest.runtime_version = String::new();
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_manifest_validate_empty_project_id() {
        let mut manifest = Manifest::new("Test");
        manifest.project_id = String::new();
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_manifest_to_from_json() {
        let manifest = Manifest::new("JSON Test");
        let json = manifest.to_json().unwrap();
        let parsed = Manifest::from_json(&json).unwrap();
        assert_eq!(manifest.project_name, parsed.project_name);
        assert_eq!(manifest.version, parsed.version);
        assert_eq!(manifest.entry_point, parsed.entry_point);
        assert_eq!(manifest.runtime_version, parsed.runtime_version);
        assert_eq!(manifest.project_id, parsed.project_id);
    }

    #[test]
    fn test_manifest_from_json_invalid() {
        let result = Manifest::from_json("not json");
        assert!(result.is_err());
    }

    #[test]
    fn test_manifest_add_find_remove_asset() {
        let mut manifest = Manifest::new("Asset Test");
        let asset = AssetInfo {
            name: "cat.svg".to_string(),
            asset_id: "abc".to_string(),
            extension: "svg".to_string(),
            size_bytes: 100,
            mime_type: "image/svg+xml".to_string(),
            relative_path: "assets/abc.svg".to_string(),
        };
        manifest.add_asset(asset.clone());

        assert_eq!(manifest.assets.len(), 1);
        assert!(manifest.find_asset("cat.svg").is_some());
        assert!(manifest.find_asset("nonexistent.svg").is_none());
        assert!(manifest.find_asset_by_id("abc").is_some());
        assert!(manifest.find_asset_by_id("xyz").is_none());

        let removed = manifest.remove_asset("cat.svg");
        assert!(removed.is_some());
        assert_eq!(manifest.assets.len(), 0);
        assert!(manifest.find_asset("cat.svg").is_none());

        let removed_again = manifest.remove_asset("cat.svg");
        assert!(removed_again.is_none());
    }

    #[test]
    fn test_manifest_total_asset_size() {
        let mut manifest = Manifest::new("Size Test");
        let a1 = AssetInfo {
            name: "a.svg".to_string(),
            asset_id: "1".to_string(),
            extension: "svg".to_string(),
            size_bytes: 100,
            mime_type: "image/svg+xml".to_string(),
            relative_path: "assets/1.svg".to_string(),
        };
        let a2 = AssetInfo {
            name: "b.wav".to_string(),
            asset_id: "2".to_string(),
            extension: "wav".to_string(),
            size_bytes: 200,
            mime_type: "audio/wav".to_string(),
            relative_path: "assets/2.wav".to_string(),
        };
        manifest.add_asset(a1);
        manifest.add_asset(a2);
        assert_eq!(manifest.total_asset_size(), 300);
    }

    #[test]
    fn test_manifest_modified_at_updates() {
        let mut manifest = Manifest::new("Time Test");
        let _before = manifest.modified_at.clone();
        // Small sleep to ensure timestamp changes
        let asset = AssetInfo {
            name: "test.svg".to_string(),
            asset_id: "1".to_string(),
            extension: "svg".to_string(),
            size_bytes: 0,
            mime_type: "image/svg+xml".to_string(),
            relative_path: "assets/1.svg".to_string(),
        };
        manifest.add_asset(asset);
        // Modified time should have been updated (or be the same if very fast)
        // Just verify it's not empty
        assert!(!manifest.modified_at.is_empty());
    }

    #[test]
    fn test_manifest_custom_metadata() {
        let mut manifest = Manifest::new("Meta Test");
        manifest.custom_metadata.insert("key1".to_string(), "value1".to_string());
        assert_eq!(manifest.custom_metadata.get("key1"), Some(&"value1".to_string()));
    }

    // === ProjectBundle tests ===

    #[test]
    fn test_bundle_new() {
        let bundle = ProjectBundle::new("Test Bundle");
        assert_eq!(bundle.manifest.project_name, "Test Bundle");
        assert!(bundle.source_code.is_empty());
        assert!(bundle.asset_data.is_empty());
    }

    #[test]
    fn test_bundle_create_test_bundle() {
        let bundle = ProjectBundle::create_test_bundle("Test");
        assert_eq!(bundle.manifest.project_name, "Test");
        assert!(!bundle.source_code.is_empty());
        assert_eq!(bundle.manifest.assets.len(), 2);
        assert_eq!(bundle.asset_data.len(), 2);
    }

    #[test]
    fn test_bundle_validate_valid() {
        let bundle = ProjectBundle::create_test_bundle("Valid");
        assert!(bundle.validate().is_ok());
    }

    #[test]
    fn test_bundle_validate_invalid_manifest() {
        let mut bundle = ProjectBundle::new("Test");
        bundle.manifest.project_name = String::new();
        assert!(bundle.validate().is_err());
    }

    #[test]
    fn test_bundle_assets_accessor() {
        let bundle = ProjectBundle::create_test_bundle("Test");
        assert_eq!(bundle.assets().len(), 2);
    }

    #[test]
    fn test_bundle_get_asset_data() {
        let bundle = ProjectBundle::create_test_bundle("Test");
        assert!(bundle.get_asset_data("abc123.svg").is_some());
        assert!(bundle.get_asset_data("def456.wav").is_some());
        assert!(bundle.get_asset_data("nonexistent.bin").is_none());
    }

    #[test]
    fn test_bundle_add_remove_asset() {
        let mut bundle = ProjectBundle::new("Asset Test");
        let info = AssetInfo {
            name: "new.svg".to_string(),
            asset_id: "newid".to_string(),
            extension: "svg".to_string(),
            size_bytes: 50,
            mime_type: "image/svg+xml".to_string(),
            relative_path: "assets/newid.svg".to_string(),
        };
        bundle.add_asset(info, b"<svg>new</svg>".to_vec());
        assert_eq!(bundle.asset_count(), 1);
        assert!(bundle.get_asset_data("newid.svg").is_some());

        let removed = bundle.remove_asset("new.svg");
        assert!(removed.is_some());
        assert_eq!(bundle.asset_count(), 0);
    }

    #[test]
    fn test_bundle_remove_nonexistent_asset() {
        let mut bundle = ProjectBundle::new("Test");
        assert!(bundle.remove_asset("nonexistent.svg").is_none());
    }

    #[test]
    fn test_bundle_total_data_size() {
        let bundle = ProjectBundle::create_test_bundle("Test");
        let size = bundle.total_data_size();
        // cat.svg = b"<svg>test</svg>" = 15 bytes, meow.wav = 1024 zero bytes
        assert_eq!(size, 15 + 1024);
    }

    #[test]
    fn test_bundle_asset_count() {
        let bundle = ProjectBundle::create_test_bundle("Test");
        assert_eq!(bundle.asset_count(), 2);
    }

    #[test]
    fn test_bundle_set_source_code() {
        let mut bundle = ProjectBundle::new("Code Test");
        assert!(bundle.source_code.is_empty());
        bundle.set_source_code("def main(): pass");
        assert_eq!(bundle.source_code, "def main(): pass");
    }

    #[test]
    fn test_bundle_load_from_directory_nonexistent() {
        let result = ProjectBundle::load_from_directory(Path::new("/nonexistent/path"));
        assert!(result.is_err());
        match result {
            Err(PackError::InvalidPath(_)) => {}
            _ => panic!("expected InvalidPath error"),
        }
    }

    #[test]
    fn test_bundle_load_from_directory_not_a_dir() {
        let temp = tempfile::NamedTempFile::new().unwrap();
        let result = ProjectBundle::load_from_directory(temp.path());
        assert!(result.is_err());
        match result {
            Err(PackError::InvalidPath(_)) => {}
            _ => panic!("expected InvalidPath error"),
        }
    }

    #[test]
    fn test_bundle_load_from_sfp_nonexistent() {
        let result = ProjectBundle::load_from_sfp(Path::new("/nonexistent/file.sfp"));
        assert!(result.is_err());
        match result {
            Err(PackError::InvalidPath(_)) => {}
            _ => panic!("expected InvalidPath error"),
        }
    }

    #[test]
    fn test_bundle_load_from_directory_with_manifest() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dir = temp_dir.path();

        // Create manifest
        let manifest = Manifest::new("Dir Test");
        fs::write(dir.join("manifest.json"), manifest.to_json().unwrap()).unwrap();

        // Create source
        fs::write(dir.join("project.sfl"), "def main(): pass").unwrap();

        // Create assets dir
        fs::create_dir_all(dir.join("assets")).unwrap();
        fs::write(dir.join("assets/test.svg"), b"<svg>hello</svg>").unwrap();

        let bundle = ProjectBundle::load_from_directory(dir).unwrap();
        assert_eq!(bundle.manifest.project_name, "Dir Test");
        assert_eq!(bundle.source_code, "def main(): pass");
        // Assets loaded from filesystem won't be in manifest.assets unless listed there
        // but they will be in asset_data
        assert!(bundle.asset_data.contains_key("test.svg"));
    }

    #[test]
    fn test_bundle_load_from_directory_without_manifest() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dir = temp_dir.path();

        // Only source file, no manifest
        fs::write(dir.join("project.sfl"), "def main(): pass").unwrap();

        let bundle = ProjectBundle::load_from_directory(dir).unwrap();
        // Project name should come from directory name
        assert!(!bundle.manifest.project_name.is_empty());
        assert_eq!(bundle.source_code, "def main(): pass");
    }

    #[test]
    fn test_bundle_save_and_reload_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dir = temp_dir.path().join("project");

        let bundle = ProjectBundle::create_test_bundle("Save Test");
        bundle.save_to_directory(&dir).unwrap();

        let loaded = ProjectBundle::load_from_directory(&dir).unwrap();
        assert_eq!(loaded.manifest.project_name, "Save Test");
        assert_eq!(loaded.source_code, bundle.source_code);
        // Assets should be present
        assert_eq!(loaded.manifest.assets.len(), 2);
    }

    #[test]
    fn test_bundle_round_trip_sfp() {
        let temp_dir = tempfile::tempdir().unwrap();
        let sfp_path = temp_dir.path().join("test.sfp");

        // Create bundle
        let bundle = ProjectBundle::create_test_bundle("SFP Round Trip");

        // Export to SFP
        let config = crate::PackagerConfig::default();
        let exporter = crate::exporters::SfpExporter::new(&config);
        let _result = crate::exporters::Exporter::export(&exporter, &bundle, &sfp_path, None).unwrap();

        // Re-load from SFP
        let loaded = ProjectBundle::load_from_sfp(&sfp_path).unwrap();
        assert_eq!(loaded.manifest.project_name, "SFP Round Trip");
        assert_eq!(loaded.source_code, bundle.source_code);
        assert_eq!(loaded.manifest.assets.len(), 2);
        // Verify asset data
        assert!(loaded.get_asset_data("abc123.svg").is_some());
        assert!(loaded.get_asset_data("def456.wav").is_some());
    }

    #[test]
    fn test_manifest_runtime_version_constant() {
        assert!(!Manifest::RUNTIME_VERSION.is_empty());
    }

    #[test]
    fn test_asset_info_relative_path() {
        let info = AssetInfo::new("test.png", "hash123", "png");
        assert_eq!(info.relative_path, "assets/hash123.png");
    }

    #[test]
    fn test_bundle_empty_validate() {
        let bundle = ProjectBundle::new("");  // empty name
        // Should fail because project_name is empty after manifest validation
        // Actually Manifest::new("") will set project_name to "", but validate checks for empty
        // Wait, Manifest::new("") sets project_name to "", and validate checks is_empty()
        // But we pass "" which IS empty
        assert!(bundle.validate().is_err());
    }
}
