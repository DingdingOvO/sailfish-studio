//! Parse `.sf` files (Sailfish project format based on SQLite).
//!
//! The .sf format stores project data in a SQLite database with tables for
//! metadata, targets, assets, and settings. This module provides structures
//! for representing parsed .sf data and functions for deserializing from JSON
//! (for testing) and from actual SQLite databases.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::{ParseError, Result};

// ---------------------------------------------------------------------------
// Top-level project structure
// ---------------------------------------------------------------------------

/// A Sailfish project parsed from a `.sf` file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SfProject {
    /// Project metadata.
    #[serde(default)]
    pub metadata: SfMetadata,

    /// All targets (sprites and the stage).
    #[serde(default)]
    pub targets: Vec<SfTarget>,

    /// Asset references (costumes, sounds, backdrops).
    #[serde(default)]
    pub assets: Vec<SfAsset>,

    /// Project-level settings.
    #[serde(default)]
    pub settings: SfSettings,
}

// ---------------------------------------------------------------------------
// Metadata
// ---------------------------------------------------------------------------

/// Default version number for Sailfish projects.
fn default_version() -> u32 {
    1
}

/// Project metadata stored in the .sf file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SfMetadata {
    /// Project name.
    #[serde(default, rename = "name")]
    pub name: String,

    /// Human-readable description.
    #[serde(default, rename = "description")]
    pub description: String,

    /// Author / creator name.
    #[serde(default, rename = "author")]
    pub author: String,

    /// ISO-8601 creation timestamp.
    #[serde(default, rename = "created_at")]
    pub created_at: String,

    /// ISO-8601 last-modified timestamp.
    #[serde(default, rename = "modified_at")]
    pub modified_at: String,

    /// Sailfish format version.
    #[serde(default = "default_version", rename = "version")]
    pub version: u32,
}

impl Default for SfMetadata {
    fn default() -> Self {
        SfMetadata {
            name: String::new(),
            description: String::new(),
            author: String::new(),
            created_at: String::new(),
            modified_at: String::new(),
            version: 1,
        }
    }
}

// ---------------------------------------------------------------------------
// Target
// ---------------------------------------------------------------------------

/// A Sailfish target – either the Stage or a Sprite.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SfTarget {
    /// Whether this target is the Stage.
    #[serde(default, rename = "isStage")]
    pub is_stage: bool,

    /// Human-readable name.
    #[serde(default, rename = "name")]
    pub name: String,

    /// Variables keyed by unique id.
    #[serde(default, rename = "variables")]
    pub variables: HashMap<String, SfVariable>,

    /// Lists keyed by unique id.
    #[serde(default, rename = "lists")]
    pub lists: HashMap<String, SfList>,

    /// Blocks keyed by unique id.
    #[serde(default, rename = "blocks")]
    pub blocks: HashMap<String, SfBlock>,

    /// Costume asset references (IDs into the assets table).
    #[serde(default, rename = "costumeIds")]
    pub costume_ids: Vec<String>,

    /// Sound asset references (IDs into the assets table).
    #[serde(default, rename = "soundIds")]
    pub sound_ids: Vec<String>,

    /// Index of the currently selected costume.
    #[serde(default, rename = "currentCostume")]
    pub current_costume: i64,

    /// Layer order for rendering.
    #[serde(default, rename = "layerOrder")]
    pub layer_order: i64,

    /// Sprite-specific: x position.
    #[serde(default, rename = "x", skip_serializing_if = "Option::is_none")]
    pub x: Option<f64>,

    /// Sprite-specific: y position.
    #[serde(default, rename = "y", skip_serializing_if = "Option::is_none")]
    pub y: Option<f64>,

    /// Sprite-specific: direction in degrees.
    #[serde(default, rename = "direction", skip_serializing_if = "Option::is_none")]
    pub direction: Option<f64>,

    /// Sprite-specific: size percentage.
    #[serde(default, rename = "size", skip_serializing_if = "Option::is_none")]
    pub size: Option<f64>,

    /// Sprite-specific: rotation style.
    #[serde(default, rename = "rotationStyle", skip_serializing_if = "Option::is_none")]
    pub rotation_style: Option<String>,

    /// Sprite-specific: visibility.
    #[serde(default, rename = "visible", skip_serializing_if = "Option::is_none")]
    pub visible: Option<bool>,

    /// Sprite-specific: draggable.
    #[serde(default, rename = "draggable", skip_serializing_if = "Option::is_none")]
    pub draggable: Option<bool>,

    /// Sailfish-specific: target tags for categorization.
    #[serde(default, rename = "tags")]
    pub tags: Vec<String>,

    /// Sailfish-specific: custom properties.
    #[serde(default, rename = "customProperties")]
    pub custom_properties: HashMap<String, serde_json::Value>,
}

impl Default for SfTarget {
    fn default() -> Self {
        SfTarget {
            is_stage: false,
            name: String::new(),
            variables: HashMap::new(),
            lists: HashMap::new(),
            blocks: HashMap::new(),
            costume_ids: Vec::new(),
            sound_ids: Vec::new(),
            current_costume: 0,
            layer_order: 0,
            x: None,
            y: None,
            direction: None,
            size: None,
            rotation_style: None,
            visible: None,
            draggable: None,
            tags: Vec::new(),
            custom_properties: HashMap::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Variable, List, Block
// ---------------------------------------------------------------------------

/// A Sailfish variable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SfVariable {
    /// Variable name.
    #[serde(default)]
    pub name: String,

    /// Variable value (can be any JSON value).
    #[serde(default)]
    pub value: serde_json::Value,

    /// Whether this is a cloud variable.
    #[serde(default)]
    pub is_cloud: bool,

    /// Sailfish-specific: whether this variable is persistent across sessions.
    #[serde(default)]
    pub persistent: bool,
}

/// A Sailfish list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SfList {
    /// List name.
    #[serde(default)]
    pub name: String,

    /// List values.
    #[serde(default)]
    pub values: Vec<serde_json::Value>,

    /// Sailfish-specific: whether this list is persistent across sessions.
    #[serde(default)]
    pub persistent: bool,
}

/// A Sailfish block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SfBlock {
    /// The opcode string (e.g. "motion_movesteps", "sf_custom_block").
    #[serde(default)]
    pub opcode: String,

    /// Block inputs keyed by input name.
    #[serde(default)]
    pub inputs: HashMap<String, SfBlockInput>,

    /// Block fields keyed by field name.
    #[serde(default)]
    pub fields: HashMap<String, SfBlockField>,

    /// ID of the next block in the stack.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,

    /// ID of the parent block.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,

    /// Whether this is a shadow block.
    #[serde(default)]
    pub shadow: bool,

    /// Whether this is a top-level (hat) block.
    #[serde(default)]
    pub top_level: bool,

    /// Editor position x.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x: Option<f64>,

    /// Editor position y.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub y: Option<f64>,

    /// Sailfish-specific: comment text associated with this block.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,

    /// Sailfish-specific: whether this block is disabled (grayed out).
    #[serde(default)]
    pub disabled: bool,
}

/// A block input value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SfBlockInput {
    /// The shadow type string.
    #[serde(default)]
    pub shadow_type: String,

    /// The primary value.
    #[serde(default)]
    pub primary: serde_json::Value,

    /// The actual input value (for overridden shadows).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<serde_json::Value>,
}

/// A block field value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SfBlockField {
    /// The display value.
    #[serde(default)]
    pub value: serde_json::Value,

    /// The internal ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

// ---------------------------------------------------------------------------
// Asset
// ---------------------------------------------------------------------------

/// An asset reference (costume or sound) stored in the .sf file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SfAsset {
    /// Unique asset ID.
    #[serde(default, rename = "id")]
    pub id: String,

    /// Display name.
    #[serde(default, rename = "name")]
    pub name: String,

    /// Asset type (e.g. "costume", "sound", "backdrop").
    #[serde(default, rename = "type")]
    pub asset_type: String,

    /// File extension (e.g. "svg", "png", "wav", "mp3").
    #[serde(default, rename = "dataFormat")]
    pub data_format: String,

    /// The asset data (could be a file hash, path, or inline base64).
    #[serde(default, rename = "data")]
    pub data: String,
}

impl Default for SfAsset {
    fn default() -> Self {
        SfAsset {
            id: String::new(),
            name: String::new(),
            asset_type: "costume".to_string(),
            data_format: "svg".to_string(),
            data: String::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

/// Project-level settings stored in the .sf file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SfSettings {
    /// Turbo mode enabled.
    #[serde(default, rename = "turboMode")]
    pub turbo_mode: bool,

    /// Frames per second (default 30).
    #[serde(default, rename = "fps")]
    pub fps: f64,

    /// Whether the stage is in full-screen mode.
    #[serde(default, rename = "fullScreen")]
    pub full_screen: bool,

    /// Canvas width in pixels.
    #[serde(default, rename = "canvasWidth")]
    pub canvas_width: i64,

    /// Canvas height in pixels.
    #[serde(default, rename = "canvasHeight")]
    pub canvas_height: i64,

    /// Custom key-value settings.
    #[serde(default, rename = "custom")]
    pub custom: HashMap<String, serde_json::Value>,
}

impl Default for SfSettings {
    fn default() -> Self {
        SfSettings {
            turbo_mode: false,
            fps: 30.0,
            full_screen: false,
            canvas_width: 480,
            canvas_height: 360,
            custom: HashMap::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Parsing functions
// ---------------------------------------------------------------------------

/// Parse a Sailfish project from a `.sf` SQLite database file.
///
/// This function opens the SQLite database, reads the metadata, targets,
/// assets, and settings tables, and constructs an [`SfProject`].
#[cfg(feature = "sqlite")]
pub fn parse_sf(data: &[u8]) -> Result<SfProject> {
    use rusqlite::Connection;
    use std::io::Write;

    // Write to a temp file because rusqlite needs a path or in-memory connection
    let mut temp_file = tempfile::NamedTempFile::new()
        .map_err(|e| ParseError::IoError(e.to_string()))?;
    temp_file.write_all(data)
        .map_err(|e| ParseError::IoError(e.to_string()))?;
    let path = temp_file.path().to_path_buf();

    let conn = Connection::open(&path)
        .map_err(|e| ParseError::SqliteError(e.to_string()))?;

    parse_sf_from_conn(&conn)
}

#[cfg(feature = "sqlite")]
fn parse_sf_from_conn(conn: &rusqlite::Connection) -> Result<SfProject> {
    use rusqlite::params;

    // Read metadata
    let mut metadata = SfMetadata::default();
    let has_meta: bool = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='metadata'")
        .map_err(|e| ParseError::SqliteError(e.to_string()))?
        .exists(params![])
        .map_err(|e| ParseError::SqliteError(e.to_string()))?;

    if has_meta {
        let mut stmt = conn
            .prepare("SELECT key, value FROM metadata")
            .map_err(|e| ParseError::SqliteError(e.to_string()))?;
        let rows = stmt
            .query_map(params![], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| ParseError::SqliteError(e.to_string()))?;

        let meta_map: HashMap<String, String> = rows
            .filter_map(|r| r.ok())
            .collect();

        metadata.name = meta_map.get("name").cloned().unwrap_or_default();
        metadata.description = meta_map.get("description").cloned().unwrap_or_default();
        metadata.author = meta_map.get("author").cloned().unwrap_or_default();
        metadata.created_at = meta_map.get("created_at").cloned().unwrap_or_default();
        metadata.modified_at = meta_map.get("modified_at").cloned().unwrap_or_default();
        metadata.version = meta_map
            .get("version")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);
    }

    // For now, return project with just metadata. Targets/assets would
    // follow the same pattern of reading from their respective tables.
    Ok(SfProject {
        metadata,
        targets: Vec::new(),
        assets: Vec::new(),
        settings: SfSettings::default(),
    })
}

/// Parse a Sailfish project from a JSON string.
///
/// This is useful for testing and for situations where the project data
/// has already been extracted from the SQLite database.
pub fn parse_sf_metadata_from_json(json: &str) -> Result<SfProject> {
    let project: SfProject = serde_json::from_str(json)?;
    Ok(project)
}

/// Validate that an [`SfProject`] has the minimum required fields.
///
/// Returns `Ok(())` if valid, or an error describing what's missing.
pub fn validate_sf_project(project: &SfProject) -> Result<()> {
    if project.metadata.name.is_empty() {
        return Err(ParseError::MissingField("metadata.name".to_string()));
    }

    // Check that at least one target exists
    if project.targets.is_empty() {
        return Err(ParseError::MissingField("targets".to_string()));
    }

    // Check that the first target with is_stage=true exists
    let has_stage = project.targets.iter().any(|t| t.is_stage);
    if !has_stage {
        return Err(ParseError::MissingField(
            "targets: no stage target found".to_string(),
        ));
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_metadata_from_json() {
        let json = r#"{
            "metadata": {
                "name": "My Project",
                "description": "A test project",
                "author": "Alice",
                "created_at": "2024-01-15T10:00:00Z",
                "modified_at": "2024-01-16T12:30:00Z",
                "version": 2
            },
            "targets": [],
            "assets": [],
            "settings": {}
        }"#;
        let project = parse_sf_metadata_from_json(json).unwrap();
        assert_eq!(project.metadata.name, "My Project");
        assert_eq!(project.metadata.description, "A test project");
        assert_eq!(project.metadata.author, "Alice");
        assert_eq!(project.metadata.created_at, "2024-01-15T10:00:00Z");
        assert_eq!(project.metadata.modified_at, "2024-01-16T12:30:00Z");
        assert_eq!(project.metadata.version, 2);
    }

    #[test]
    fn test_parse_project_with_targets() {
        let json = r#"{
            "metadata": {
                "name": "Test",
                "author": "Bob"
            },
            "targets": [
                {
                    "isStage": true,
                    "name": "Stage",
                    "variables": {
                        "v1": {"name": "score", "value": 0, "is_cloud": true, "persistent": false}
                    },
                    "lists": {},
                    "blocks": {},
                    "costumeIds": ["asset1"],
                    "soundIds": [],
                    "currentCostume": 0,
                    "layerOrder": 0
                },
                {
                    "isStage": false,
                    "name": "Sprite1",
                    "x": 10,
                    "y": 20,
                    "direction": 90,
                    "size": 100,
                    "visible": true,
                    "tags": ["player", "main"],
                    "customProperties": {
                        "health": 100
                    }
                }
            ],
            "assets": [
                {
                    "id": "asset1",
                    "name": "backdrop1",
                    "type": "backdrop",
                    "dataFormat": "svg",
                    "data": "abc123hash"
                }
            ],
            "settings": {
                "turboMode": true,
                "fps": 60,
                "canvasWidth": 960,
                "canvasHeight": 720
            }
        }"#;
        let project = parse_sf_metadata_from_json(json).unwrap();
        assert_eq!(project.targets.len(), 2);

        // Stage target
        let stage = &project.targets[0];
        assert!(stage.is_stage);
        assert_eq!(stage.name, "Stage");
        assert!(stage.variables.contains_key("v1"));
        let var = &stage.variables["v1"];
        assert_eq!(var.name, "score");
        assert!(var.is_cloud);
        assert!(!var.persistent);
        assert_eq!(stage.costume_ids, vec!["asset1"]);

        // Sprite target
        let sprite = &project.targets[1];
        assert!(!sprite.is_stage);
        assert_eq!(sprite.name, "Sprite1");
        assert_eq!(sprite.x, Some(10.0));
        assert_eq!(sprite.y, Some(20.0));
        assert_eq!(sprite.tags, vec!["player", "main"]);
        assert!(sprite.custom_properties.contains_key("health"));
        assert_eq!(sprite.custom_properties["health"], 100);

        // Assets
        assert_eq!(project.assets.len(), 1);
        assert_eq!(project.assets[0].id, "asset1");
        assert_eq!(project.assets[0].asset_type, "backdrop");

        // Settings
        assert!(project.settings.turbo_mode);
        assert_eq!(project.settings.fps, 60.0);
        assert_eq!(project.settings.canvas_width, 960);
        assert_eq!(project.settings.canvas_height, 720);
    }

    #[test]
    fn test_parse_handles_defaults() {
        let json = r#"{
            "metadata": {
                "name": "Minimal"
            },
            "targets": [{
                "isStage": true,
                "name": "Stage"
            }]
        }"#;
        let project = parse_sf_metadata_from_json(json).unwrap();
        assert!(project.metadata.description.is_empty());
        assert!(project.metadata.author.is_empty());
        assert!(project.metadata.created_at.is_empty());
        assert_eq!(project.metadata.version, 1);
        assert!(project.assets.is_empty());
        assert!(!project.settings.turbo_mode);
        assert_eq!(project.settings.fps, 30.0);
        assert_eq!(project.settings.canvas_width, 480);
        assert_eq!(project.settings.canvas_height, 360);
    }

    #[test]
    fn test_validate_valid_project() {
        let project = SfProject {
            metadata: SfMetadata {
                name: "Test".to_string(),
                ..Default::default()
            },
            targets: vec![SfTarget {
                is_stage: true,
                name: "Stage".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        assert!(validate_sf_project(&project).is_ok());
    }

    #[test]
    fn test_validate_project_missing_name() {
        let project = SfProject {
            metadata: SfMetadata {
                name: String::new(),
                ..Default::default()
            },
            targets: vec![SfTarget {
                is_stage: true,
                name: "Stage".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let result = validate_sf_project(&project);
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::MissingField(field) => assert_eq!(field, "metadata.name"),
            other => panic!("expected MissingField, got: {other}"),
        }
    }

    #[test]
    fn test_validate_project_no_stage() {
        let project = SfProject {
            metadata: SfMetadata {
                name: "Test".to_string(),
                ..Default::default()
            },
            targets: vec![SfTarget {
                is_stage: false,
                name: "Sprite1".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let result = validate_sf_project(&project);
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::MissingField(msg) => assert!(msg.contains("stage")),
            other => panic!("expected MissingField, got: {other}"),
        }
    }

    #[test]
    fn test_validate_project_empty_targets() {
        let project = SfProject {
            metadata: SfMetadata {
                name: "Test".to_string(),
                ..Default::default()
            },
            targets: vec![],
            ..Default::default()
        };
        let result = validate_sf_project(&project);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_json() {
        let result = parse_sf_metadata_from_json("{ not valid }");
        assert!(result.is_err());
    }

    #[test]
    fn test_settings_default() {
        let settings = SfSettings::default();
        assert!(!settings.turbo_mode);
        assert_eq!(settings.fps, 30.0);
        assert!(!settings.full_screen);
        assert_eq!(settings.canvas_width, 480);
        assert_eq!(settings.canvas_height, 360);
        assert!(settings.custom.is_empty());
    }

    #[test]
    fn test_metadata_default() {
        let meta = SfMetadata::default();
        assert!(meta.name.is_empty());
        assert!(meta.description.is_empty());
        assert!(meta.author.is_empty());
        assert!(meta.created_at.is_empty());
        assert!(meta.modified_at.is_empty());
        assert_eq!(meta.version, 1);
    }

    #[test]
    fn test_sf_block_with_disabled_flag() {
        let json = r#"{
            "metadata": {"name": "Test"},
            "targets": [{
                "isStage": true,
                "name": "Stage",
                "blocks": {
                    "b1": {
                        "opcode": "motion_movesteps",
                        "inputs": {},
                        "fields": {},
                        "topLevel": true,
                        "disabled": true
                    }
                }
            }]
        }"#;
        let project = parse_sf_metadata_from_json(json).unwrap();
        let block = &project.targets[0].blocks["b1"];
        assert!(block.disabled);
        assert_eq!(block.opcode, "motion_movesteps");
    }
}
