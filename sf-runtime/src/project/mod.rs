use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

use crate::error::{Result, SfError};

/// Supported project file formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectFormat {
    /// Sailfish text language format (.sfl)
    Sfl,
    /// Sailfish package format (.sfp) - ZIP archive
    Sfp,
}

impl std::fmt::Display for ProjectFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectFormat::Sfl => write!(f, "sfl"),
            ProjectFormat::Sfp => write!(f, "sfp"),
        }
    }
}

/// Project metadata stored in manifest.json or project.sfl header.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectMeta {
    /// Project name.
    pub name: String,
    /// Project description.
    #[serde(default)]
    pub description: String,
    /// Author of the project.
    #[serde(default)]
    pub author: String,
    /// Sailfish version this project targets.
    #[serde(default = "default_version")]
    pub version: String,
    /// Creation timestamp (ISO 8601).
    #[serde(default)]
    pub created_at: String,
    /// Last modified timestamp (ISO 8601).
    #[serde(default)]
    pub modified_at: String,
    /// Project UUID.
    #[serde(default = "default_uuid")]
    pub id: String,
    /// Custom metadata fields.
    #[serde(default)]
    pub extra: HashMap<String, String>,
}

fn default_version() -> String {
    "0.1.0".to_string()
}

fn default_uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

impl Default for ProjectMeta {
    fn default() -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            name: String::new(),
            description: String::new(),
            author: String::new(),
            version: default_version(),
            created_at: now.clone(),
            modified_at: now,
            id: uuid::Uuid::new_v4().to_string(),
            extra: HashMap::new(),
        }
    }
}

impl ProjectMeta {
    /// Create new metadata with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Self::default()
        }
    }

    /// Set description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set author.
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = author.into();
        self
    }

    /// Update modified_at to now.
    pub fn touch(&mut self) {
        self.modified_at = chrono::Utc::now().to_rfc3339();
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Deserialize from JSON string.
    pub fn from_json(s: &str) -> Result<Self> {
        Ok(serde_json::from_str(s)?)
    }
}

/// A loaded Sailfish project.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Project {
    /// Project metadata.
    pub meta: ProjectMeta,
    /// Main source code (for .sfl projects).
    #[serde(default)]
    pub source: String,
    /// List of asset paths relative to project root.
    #[serde(default)]
    pub assets: Vec<String>,
    /// Runtime config overrides embedded in the project.
    #[serde(default)]
    pub config: Option<crate::config::RuntimeConfig>,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            meta: ProjectMeta::default(),
            source: String::new(),
            assets: Vec::new(),
            config: None,
        }
    }
}

impl Project {
    /// Create a new empty project.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            meta: ProjectMeta::new(name),
            ..Self::default()
        }
    }

    /// Detect the project format from a file extension.
    pub fn detect_format(path: &Path) -> Option<ProjectFormat> {
        match path.extension().and_then(|e| e.to_str()) {
            Some("sfl") => Some(ProjectFormat::Sfl),
            Some("sfp") => Some(ProjectFormat::Sfp),
            _ => None,
        }
    }

    /// Load a project from a file path.
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(SfError::ProjectNotFound(path.display().to_string()));
        }

        let format = Self::detect_format(path).ok_or_else(|| SfError::InvalidFormat {
            expected: ".sfl or .sfp".to_string(),
            actual: path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("(none)")
                .to_string(),
        })?;

        match format {
            ProjectFormat::Sfl => Self::load_sfl(path),
            ProjectFormat::Sfp => Self::load_sfp(path),
        }
    }

    /// Load a .sfl text file.
    pub fn load_sfl(path: &Path) -> Result<Self> {
        let source = fs::read_to_string(path)?;
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unnamed")
            .to_string();

        // Try to parse meta from header comments in the .sfl file
        // Format: // @name value
        let meta = Self::parse_sfl_meta(&source, &name);

        Ok(Self {
            meta,
            source,
            assets: Vec::new(),
            config: None,
        })
    }

    /// Parse metadata from .sfl header comments.
    fn parse_sfl_meta(source: &str, fallback_name: &str) -> ProjectMeta {
        let mut meta = ProjectMeta::new(fallback_name);
        for line in source.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("// @") {
                if let Some((key, value)) = rest.split_once(' ') {
                    match key.trim() {
                        "name" => meta.name = value.trim().to_string(),
                        "description" => meta.description = value.trim().to_string(),
                        "author" => meta.author = value.trim().to_string(),
                        "version" => meta.version = value.trim().to_string(),
                        _ => {
                            meta.extra
                                .insert(key.trim().to_string(), value.trim().to_string());
                        }
                    }
                }
            } else if !trimmed.is_empty() && !trimmed.starts_with("//") {
                break; // Stop at first non-comment line
            }
        }
        meta
    }

    /// Load a .sfp package (ZIP archive).
    pub fn load_sfp(path: &Path) -> Result<Self> {
        let file = fs::File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        // Read manifest.json
        let mut manifest_str = String::new();
        let meta = if let Ok(mut manifest_file) = archive.by_name("manifest.json") {
            manifest_file.read_to_string(&mut manifest_str)?;
            ProjectMeta::from_json(&manifest_str)?
        } else {
            ProjectMeta::new(
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unnamed"),
            )
        };

        // Read project.sfl
        let source = if let Ok(mut sfl_file) = archive.by_name("project.sfl") {
            let mut s = String::new();
            sfl_file.read_to_string(&mut s)?;
            s
        } else {
            String::new()
        };

        // Collect asset paths
        let mut assets = Vec::new();
        for i in 0..archive.len() {
            let file = archive.by_index(i).unwrap();
            let name = file.name().to_string();
            if name.starts_with("assets/") && !name.ends_with('/') {
                assets.push(name);
            }
        }

        Ok(Self {
            meta,
            source,
            assets,
            config: None,
        })
    }

    /// Save project as .sfl text file.
    pub fn save_sfl(&self, path: &Path) -> Result<()> {
        let mut content = String::new();
        content.push_str(&format!("// @name {}\n", self.meta.name));
        if !self.meta.description.is_empty() {
            content.push_str(&format!("// @description {}\n", self.meta.description));
        }
        if !self.meta.author.is_empty() {
            content.push_str(&format!("// @author {}\n", self.meta.author));
        }
        content.push_str(&format!("// @version {}\n", self.meta.version));
        content.push('\n');
        content.push_str(&self.source);
        fs::write(path, content)?;
        Ok(())
    }

    /// Save project as .sfp package (ZIP archive).
    pub fn save_sfp(&self, path: &Path) -> Result<()> {
        let file = fs::File::create(path)?;
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        // Write manifest.json
        let manifest = self.meta.to_json()?;
        zip.start_file("manifest.json", options)?;
        zip.write_all(manifest.as_bytes())?;

        // Write project.sfl
        zip.start_file("project.sfl", options)?;
        zip.write_all(self.source.as_bytes())?;

        // Write placeholder for assets directory
        if !self.assets.is_empty() {
            zip.add_directory("assets/", options)?;
            for asset in &self.assets {
                zip.start_file(asset, options)?;
                zip.write_all(b"")?; // Placeholder - actual asset loading would go here
            }
        }

        zip.finish()?;
        Ok(())
    }

    /// Save the project, detecting format from the file extension.
    pub fn save(&self, path: &Path) -> Result<()> {
        let format = Self::detect_format(path).ok_or_else(|| SfError::InvalidFormat {
            expected: ".sfl or .sfp".to_string(),
            actual: path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("(none)")
                .to_string(),
        })?;

        match format {
            ProjectFormat::Sfl => self.save_sfl(path),
            ProjectFormat::Sfp => self.save_sfp(path),
        }
    }

    /// Add an asset path to the project.
    pub fn add_asset(&mut self, asset_path: impl Into<String>) {
        let path = asset_path.into();
        if !self.assets.contains(&path) {
            self.assets.push(path);
        }
    }

    /// Remove an asset path from the project.
    pub fn remove_asset(&mut self, asset_path: &str) -> bool {
        let initial_len = self.assets.len();
        self.assets.retain(|a| a != asset_path);
        self.assets.len() != initial_len
    }

    /// Get the project name.
    pub fn name(&self) -> &str {
        &self.meta.name
    }

    /// Check if the project source is empty (no actual code, only comments/whitespace).
    pub fn is_empty(&self) -> bool {
        self.source.lines().all(|line| {
            let trimmed = line.trim();
            trimmed.is_empty() || trimmed.starts_with("//")
        })
    }

    /// Validate the project structure.
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.meta.name.is_empty() {
            errors.push("Project name is empty".to_string());
        }
        if self.meta.name.contains('/') || self.meta.name.contains('\\') {
            errors.push("Project name contains path separators".to_string());
        }
        errors
    }
}

/// Manifest for .sfp packages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PackageManifest {
    /// Package format version.
    #[serde(default = "default_manifest_version")]
    pub manifest_version: u32,
    /// Project metadata.
    #[serde(flatten)]
    pub meta: ProjectMeta,
    /// List of files in the package (relative paths).
    #[serde(default)]
    pub files: Vec<String>,
    /// Whether the runtime is embedded.
    #[serde(default)]
    pub embed_runtime: bool,
    /// Runtime version used.
    #[serde(default)]
    pub runtime_version: String,
}

fn default_manifest_version() -> u32 {
    1
}

impl Default for PackageManifest {
    fn default() -> Self {
        Self {
            manifest_version: 1,
            meta: ProjectMeta::default(),
            files: Vec::new(),
            embed_runtime: false,
            runtime_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

impl PackageManifest {
    /// Create a new manifest.
    pub fn new(meta: ProjectMeta) -> Self {
        Self {
            meta,
            ..Self::default()
        }
    }

    /// Create a manifest with embedded runtime.
    pub fn with_embedded_runtime(meta: ProjectMeta) -> Self {
        Self {
            meta,
            embed_runtime: true,
            ..Self::default()
        }
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Deserialize from JSON.
    pub fn from_json(s: &str) -> Result<Self> {
        Ok(serde_json::from_str(s)?)
    }

    /// Add a file to the manifest.
    pub fn add_file(&mut self, path: impl Into<String>) {
        let p = path.into();
        if !self.files.contains(&p) {
            self.files.push(p);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_project_format_display() {
        assert_eq!(ProjectFormat::Sfl.to_string(), "sfl");
        assert_eq!(ProjectFormat::Sfp.to_string(), "sfp");
    }

    #[test]
    fn test_detect_format_sfl() {
        assert_eq!(
            Project::detect_format(Path::new("test.sfl")),
            Some(ProjectFormat::Sfl)
        );
    }

    #[test]
    fn test_detect_format_sfp() {
        assert_eq!(
            Project::detect_format(Path::new("test.sfp")),
            Some(ProjectFormat::Sfp)
        );
    }

    #[test]
    fn test_detect_format_unknown() {
        assert_eq!(Project::detect_format(Path::new("test.txt")), None);
    }

    #[test]
    fn test_detect_format_no_extension() {
        assert_eq!(Project::detect_format(Path::new("test")), None);
    }

    #[test]
    fn test_project_meta_new() {
        let meta = ProjectMeta::new("TestProject");
        assert_eq!(meta.name, "TestProject");
        assert_eq!(meta.version, "0.1.0");
        assert!(!meta.id.is_empty());
    }

    #[test]
    fn test_project_meta_with_description() {
        let meta = ProjectMeta::new("Test").with_description("A test project");
        assert_eq!(meta.description, "A test project");
    }

    #[test]
    fn test_project_meta_with_author() {
        let meta = ProjectMeta::new("Test").with_author("Alice");
        assert_eq!(meta.author, "Alice");
    }

    #[test]
    fn test_project_meta_json_roundtrip() {
        let meta = ProjectMeta::new("Test");
        let json = meta.to_json().unwrap();
        let parsed = ProjectMeta::from_json(&json).unwrap();
        assert_eq!(meta, parsed);
    }

    #[test]
    fn test_project_meta_touch_updates_modified() {
        let mut meta = ProjectMeta::new("Test");
        let old_modified = meta.modified_at.clone();
        meta.touch();
        assert_ne!(old_modified, meta.modified_at);
    }

    #[test]
    fn test_project_new() {
        let project = Project::new("MyProject");
        assert_eq!(project.name(), "MyProject");
        assert!(project.is_empty());
    }

    #[test]
    fn test_project_add_asset() {
        let mut project = Project::new("Test");
        project.add_asset("assets/image.png");
        assert_eq!(project.assets.len(), 1);
        assert_eq!(project.assets[0], "assets/image.png");
    }

    #[test]
    fn test_project_add_duplicate_asset() {
        let mut project = Project::new("Test");
        project.add_asset("assets/image.png");
        project.add_asset("assets/image.png");
        assert_eq!(project.assets.len(), 1);
    }

    #[test]
    fn test_project_remove_asset() {
        let mut project = Project::new("Test");
        project.add_asset("assets/image.png");
        assert!(project.remove_asset("assets/image.png"));
        assert!(project.assets.is_empty());
    }

    #[test]
    fn test_project_remove_nonexistent_asset() {
        let mut project = Project::new("Test");
        assert!(!project.remove_asset("assets/nonexistent.png"));
    }

    #[test]
    fn test_project_validate_empty_name() {
        let project = Project {
            meta: ProjectMeta {
                name: String::new(),
                ..ProjectMeta::default()
            },
            ..Project::default()
        };
        let errors = project.validate();
        assert!(errors.iter().any(|e| e.contains("empty")));
    }

    #[test]
    fn test_project_validate_name_with_slash() {
        let project = Project {
            meta: ProjectMeta::new("bad/name"),
            ..Project::default()
        };
        let errors = project.validate();
        assert!(errors.iter().any(|e| e.contains("path separators")));
    }

    #[test]
    fn test_project_validate_ok() {
        let project = Project::new("GoodName");
        assert!(project.validate().is_empty());
    }

    #[test]
    fn test_parse_sfl_meta() {
        let source = "// @name MyProject\n// @author Bob\n// @version 2.0\n\nfn main() {}";
        let meta = Project::parse_sfl_meta(source, "fallback");
        assert_eq!(meta.name, "MyProject");
        assert_eq!(meta.author, "Bob");
        assert_eq!(meta.version, "2.0");
    }

    #[test]
    fn test_parse_sfl_meta_fallback() {
        let source = "fn main() {}";
        let meta = Project::parse_sfl_meta(source, "fallback");
        assert_eq!(meta.name, "fallback");
    }

    #[test]
    fn test_save_load_sfl_roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.sfl");

        let mut project = Project::new("TestProject");
        project.source = "fn main() { print(\"hello\"); }".to_string();

        project.save_sfl(&path).unwrap();
        let loaded = Project::load_sfl(&path).unwrap();

        // Note: save_sfl prepends metadata comments, load_sfl reads the entire file as source
        assert!(loaded.source.contains("fn main() { print(\"hello\"); }"));
        assert_eq!(loaded.meta.name, "TestProject");
    }

    #[test]
    fn test_save_load_sfp_roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.sfp");

        let mut project = Project::new("PackedProject");
        project.source = "fn main() {}".to_string();
        project.add_asset("assets/sprite.png");

        project.save_sfp(&path).unwrap();
        let loaded = Project::load_sfp(&path).unwrap();

        assert_eq!(loaded.meta.name, "PackedProject");
        assert_eq!(loaded.source, "fn main() {}");
        assert!(loaded.assets.contains(&"assets/sprite.png".to_string()));
    }

    #[test]
    fn test_load_nonexistent_file() {
        let result = Project::load(Path::new("/nonexistent/file.sfl"));
        assert!(result.is_err());
    }

    #[test]
    fn test_load_invalid_format() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.txt");
        fs::write(&path, "hello").unwrap();
        let result = Project::load(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_project_is_empty() {
        let project = Project::new("Test");
        assert!(project.is_empty());
    }

    #[test]
    fn test_project_is_empty_with_only_comments() {
        let mut project = Project::new("Test");
        project.source = "// just a comment\n".to_string();
        assert!(project.is_empty());
    }

    #[test]
    fn test_project_is_not_empty() {
        let mut project = Project::new("Test");
        project.source = "fn main() {}".to_string();
        assert!(!project.is_empty());
    }

    #[test]
    fn test_project_is_not_empty_with_code_and_comments() {
        let mut project = Project::new("Test");
        project.source = "// comment\nfn main() {}".to_string();
        assert!(!project.is_empty());
    }

    #[test]
    fn test_package_manifest_new() {
        let meta = ProjectMeta::new("Test");
        let manifest = PackageManifest::new(meta);
        assert_eq!(manifest.manifest_version, 1);
        assert!(!manifest.embed_runtime);
    }

    #[test]
    fn test_package_manifest_with_embedded_runtime() {
        let meta = ProjectMeta::new("Test");
        let manifest = PackageManifest::with_embedded_runtime(meta);
        assert!(manifest.embed_runtime);
    }

    #[test]
    fn test_package_manifest_add_file() {
        let meta = ProjectMeta::new("Test");
        let mut manifest = PackageManifest::new(meta);
        manifest.add_file("project.sfl");
        assert!(manifest.files.contains(&"project.sfl".to_string()));
    }

    #[test]
    fn test_package_manifest_add_duplicate_file() {
        let meta = ProjectMeta::new("Test");
        let mut manifest = PackageManifest::new(meta);
        manifest.add_file("project.sfl");
        manifest.add_file("project.sfl");
        assert_eq!(manifest.files.len(), 1);
    }

    #[test]
    fn test_package_manifest_json_roundtrip() {
        let meta = ProjectMeta::new("Test");
        let manifest = PackageManifest::with_embedded_runtime(meta);
        let json = manifest.to_json().unwrap();
        let parsed = PackageManifest::from_json(&json).unwrap();
        assert_eq!(manifest, parsed);
    }

    #[test]
    fn test_save_detects_format() {
        let dir = TempDir::new().unwrap();
        let sfl_path = dir.path().join("test.sfl");
        let sfp_path = dir.path().join("test.sfp");

        let project = Project::new("Test");
        project.save(&sfl_path).unwrap();
        project.save(&sfp_path).unwrap();

        assert!(sfl_path.exists());
        assert!(sfp_path.exists());
    }

    #[test]
    fn test_save_invalid_format() {
        let dir = TempDir::new().unwrap();
        let txt_path = dir.path().join("test.txt");

        let project = Project::new("Test");
        let result = project.save(&txt_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_meta_extra_fields() {
        let source = "// @name Test\n// @custom_field hello\n\nfn main() {}";
        let meta = Project::parse_sfl_meta(source, "fallback");
        assert_eq!(meta.extra.get("custom_field"), Some(&"hello".to_string()));
    }
}
