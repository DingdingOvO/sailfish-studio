//! Parse Scratch 3.0 `.sb3` files (ZIP format containing project.json).
//!
//! The .sb3 format is a ZIP archive containing a `project.json` file and
//! associated asset files (costumes, sounds). This module provides structures
//! for deserializing the project.json and functions for extracting it from
//! the ZIP archive.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use crate::error::{ParseError, Result};

// ---------------------------------------------------------------------------
// Top-level project structure
// ---------------------------------------------------------------------------

/// The top-level Scratch 3.0 project structure, mirroring `project.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sb3Project {
    /// All targets (sprites and the stage).
    #[serde(default)]
    pub targets: Vec<Sb3Target>,

    /// List of extension IDs loaded by this project.
    #[serde(default, rename = "extensions")]
    pub extensions: Vec<String>,

    /// Project metadata.
    #[serde(default)]
    pub meta: Sb3Meta,
}

/// Project-level metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sb3Meta {
    /// The semver version of the Scratch format.
    #[serde(default, rename = "semver")]
    pub semver: String,

    /// The VM machine name.
    #[serde(default, rename = "vm")]
    pub vm: String,

    /// Agent string (browser / OS info).
    #[serde(default, rename = "agent")]
    pub agent: String,
}

impl Default for Sb3Meta {
    fn default() -> Self {
        Sb3Meta {
            semver: "3.0.0".to_string(),
            vm: String::new(),
            agent: String::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Target (sprite / stage)
// ---------------------------------------------------------------------------

/// A Scratch target – either the Stage or a Sprite.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sb3Target {
    /// Whether this target is the Stage.
    #[serde(default, rename = "isStage")]
    pub is_stage: bool,

    /// Human-readable name.
    #[serde(default, rename = "name")]
    pub name: String,

    /// Variables keyed by unique id.
    #[serde(default, rename = "variables")]
    pub variables: HashMap<String, Sb3Variable>,

    /// Lists keyed by unique id.
    #[serde(default, rename = "lists")]
    pub lists: HashMap<String, Sb3List>,

    /// Blocks keyed by unique id.
    #[serde(default, rename = "blocks")]
    pub blocks: HashMap<String, Sb3Block>,

    /// Costumes.
    #[serde(default, rename = "costumes")]
    pub costumes: Vec<Sb3Costume>,

    /// Sounds.
    #[serde(default, rename = "sounds")]
    pub sounds: Vec<Sb3Sound>,

    /// Current costume index.
    #[serde(default, rename = "currentCostume")]
    pub current_costume: i64,

    /// Layer order in the project.
    #[serde(default, rename = "layerOrder")]
    pub layer_order: i64,

    /// Stage-specific: volume.
    #[serde(default, rename = "volume", skip_serializing_if = "Option::is_none")]
    pub volume: Option<i64>,

    /// Stage-specific: tempo.
    #[serde(default, rename = "tempo", skip_serializing_if = "Option::is_none")]
    pub tempo: Option<i64>,

    /// Sprite-specific: x position.
    #[serde(default, rename = "x", skip_serializing_if = "Option::is_none")]
    pub x: Option<f64>,

    /// Sprite-specific: y position.
    #[serde(default, rename = "y", skip_serializing_if = "Option::is_none")]
    pub y: Option<f64>,

    /// Sprite-specific: direction.
    #[serde(default, rename = "direction", skip_serializing_if = "Option::is_none")]
    pub direction: Option<f64>,

    /// Sprite-specific: size.
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
}

impl Default for Sb3Target {
    fn default() -> Self {
        Sb3Target {
            is_stage: false,
            name: String::new(),
            variables: HashMap::new(),
            lists: HashMap::new(),
            blocks: HashMap::new(),
            costumes: Vec::new(),
            sounds: Vec::new(),
            current_costume: 0,
            layer_order: 0,
            volume: None,
            tempo: None,
            x: None,
            y: None,
            direction: None,
            size: None,
            rotation_style: None,
            visible: None,
            draggable: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Variable & List
// ---------------------------------------------------------------------------

/// A Scratch variable, represented as `[name, value]` in JSON.
#[derive(Debug, Clone, PartialEq)]
pub struct Sb3Variable {
    /// Variable name.
    pub name: String,
    /// Variable value (can be any JSON value).
    pub value: serde_json::Value,
    /// Whether this is a cloud variable.
    pub is_cloud: bool,
}

/// Custom deserialization to handle Scratch's `[name, value]` tuple format
/// as well as the `{name, value, is_cloud}` object format.
impl<'de> serde::de::Deserialize<'de> for Sb3Variable {
    fn deserialize<D: serde::de::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        let val = serde_json::Value::deserialize(deserializer)?;

        // Object format: {"name": ..., "value": ..., "is_cloud": ...}
        if let Some(obj) = val.as_object() {
            let name = obj
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let value = obj.get("value").cloned().unwrap_or(serde_json::Value::Null);
            let is_cloud = obj
                .get("is_cloud")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            return Ok(Sb3Variable {
                name,
                value,
                is_cloud,
            });
        }

        // Array format: ["name", "value"] or ["name", "value", true]
        if let Some(arr) = val.as_array() {
            if arr.len() >= 2 {
                let name = arr[0].as_str().unwrap_or("").to_string();
                let value = arr[1].clone();
                let is_cloud = arr.get(2).and_then(|v| v.as_bool()).unwrap_or(false);
                return Ok(Sb3Variable {
                    name,
                    value,
                    is_cloud,
                });
            }
        }

        Err(serde::de::Error::custom(
            "expected variable as object or [name, value] array",
        ))
    }
}

impl serde::Serialize for Sb3Variable {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        // Serialize as the Scratch-compatible array format: [name, value, is_cloud?]
        use serde::ser::SerializeTuple;
        if self.is_cloud {
            let mut tup = serializer.serialize_tuple(3)?;
            tup.serialize_element(&self.name)?;
            tup.serialize_element(&self.value)?;
            tup.serialize_element(&self.is_cloud)?;
            tup.end()
        } else {
            let mut tup = serializer.serialize_tuple(2)?;
            tup.serialize_element(&self.name)?;
            tup.serialize_element(&self.value)?;
            tup.end()
        }
    }
}

/// A Scratch list, represented as `[name, [values...]]` in JSON.
#[derive(Debug, Clone, PartialEq)]
pub struct Sb3List {
    /// List name.
    pub name: String,
    /// List values.
    pub values: Vec<serde_json::Value>,
}

/// Custom deserialization to handle Scratch's `[name, [values]]` tuple format.
impl<'de> serde::de::Deserialize<'de> for Sb3List {
    fn deserialize<D: serde::de::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        let val = serde_json::Value::deserialize(deserializer)?;

        // Object format: {"name": ..., "values": [...]}
        if let Some(obj) = val.as_object() {
            let name = obj
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let values = obj
                .get("values")
                .and_then(|v| v.as_array())
                .map(|a| a.clone())
                .unwrap_or_default();
            return Ok(Sb3List { name, values });
        }

        // Array format: ["name", [values...]]
        if let Some(arr) = val.as_array() {
            if arr.len() >= 2 {
                let name = arr[0].as_str().unwrap_or("").to_string();
                let values = if let Some(vals) = arr[1].as_array() {
                    vals.clone()
                } else {
                    vec![arr[1].clone()]
                };
                return Ok(Sb3List { name, values });
            }
        }

        Err(serde::de::Error::custom(
            "expected list as object or [name, [values]] array",
        ))
    }
}

impl serde::Serialize for Sb3List {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        use serde::ser::SerializeTuple;
        let mut tup = serializer.serialize_tuple(2)?;
        tup.serialize_element(&self.name)?;
        tup.serialize_element(&self.values)?;
        tup.end()
    }
}

// ---------------------------------------------------------------------------
// Block
// ---------------------------------------------------------------------------

/// A Scratch block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sb3Block {
    /// The opcode string (e.g. "motion_movesteps").
    #[serde(default, rename = "opcode")]
    pub opcode: String,

    /// Block inputs keyed by input name.
    #[serde(default, rename = "inputs")]
    pub inputs: HashMap<String, Sb3Input>,

    /// Block fields keyed by field name.
    #[serde(default, rename = "fields")]
    pub fields: HashMap<String, Sb3Field>,

    /// ID of the next block in the stack (null if last).
    #[serde(default, rename = "next", skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,

    /// ID of the parent block (null if top-level / hat).
    #[serde(default, rename = "parent", skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,

    /// Whether this block has a shadow (input reporter).
    #[serde(default, rename = "shadow")]
    pub shadow: bool,

    /// Whether this is a top-level block (hat block or standalone).
    #[serde(default, rename = "topLevel")]
    pub top_level: bool,

    /// The x coordinate of the block in the editor (for top-level blocks).
    #[serde(default, rename = "x", skip_serializing_if = "Option::is_none")]
    pub x: Option<f64>,

    /// The y coordinate of the block in the editor (for top-level blocks).
    #[serde(default, rename = "y", skip_serializing_if = "Option::is_none")]
    pub y: Option<f64>,

    /// Optional comment ID this block is associated with.
    #[serde(default, rename = "comment", skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,

    /// Mutation data for procedure blocks.
    #[serde(default, rename = "mutation", skip_serializing_if = "Option::is_none")]
    pub mutation: Option<Sb3Mutation>,
}

impl Default for Sb3Block {
    fn default() -> Self {
        Sb3Block {
            opcode: String::new(),
            inputs: HashMap::new(),
            fields: HashMap::new(),
            next: None,
            parent: None,
            shadow: false,
            top_level: false,
            x: None,
            y: None,
            comment: None,
            mutation: None,
        }
    }
}

/// A block input (argument). In Scratch JSON this is an array:
/// `[shadow_type, shadow_block_or_value]` or
/// `[shadow_type, shadow_block, input_block]`.
#[derive(Debug, Clone, PartialEq)]
pub struct Sb3Input {
    /// The shadow type string (e.g. "input_value_rounded").
    pub shadow_type: String,

    /// The shadow / primary value (can be a block ID string or a literal array).
    pub primary: serde_json::Value,

    /// The actual input value (when different from shadow, e.g. for overridden shadows).
    pub input: Option<serde_json::Value>,
}

/// Custom deserialization for Sb3Input from Scratch's array format.
impl<'de> serde::de::Deserialize<'de> for Sb3Input {
    fn deserialize<D: serde::de::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        let val = serde_json::Value::deserialize(deserializer)?;

        // Object format: {"shadowType": ..., "primary": ..., "input": ...}
        if let Some(obj) = val.as_object() {
            let shadow_type = obj
                .get("shadowType")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let primary = obj.get("primary").cloned().unwrap_or(serde_json::Value::Null);
            let input = obj.get("input").cloned();
            return Ok(Sb3Input {
                shadow_type,
                primary,
                input,
            });
        }

        // Array format: [shadow_type, primary] or [shadow_type, primary, input]
        if let Some(arr) = val.as_array() {
            if arr.len() >= 2 {
                let shadow_type = arr[0].as_str().unwrap_or("").to_string();
                let primary = arr[1].clone();
                let input = arr.get(2).cloned();
                return Ok(Sb3Input {
                    shadow_type,
                    primary,
                    input,
                });
            }
        }

        Err(serde::de::Error::custom(
            "expected input as object or [shadow_type, primary(, input)] array",
        ))
    }
}

impl serde::Serialize for Sb3Input {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        use serde::ser::SerializeTuple;
        if self.input.is_some() {
            let mut tup = serializer.serialize_tuple(3)?;
            tup.serialize_element(&self.shadow_type)?;
            tup.serialize_element(&self.primary)?;
            tup.serialize_element(self.input.as_ref().unwrap())?;
            tup.end()
        } else {
            let mut tup = serializer.serialize_tuple(2)?;
            tup.serialize_element(&self.shadow_type)?;
            tup.serialize_element(&self.primary)?;
            tup.end()
        }
    }
}

/// A block field value. In Scratch JSON this is an array: `[value, id]`.
#[derive(Debug, Clone, PartialEq)]
pub struct Sb3Field {
    /// The display value.
    pub value: serde_json::Value,

    /// The internal ID (e.g. variable/list ID).
    pub id: Option<String>,
}

/// Custom deserialization for Sb3Field from Scratch's array format.
impl<'de> serde::de::Deserialize<'de> for Sb3Field {
    fn deserialize<D: serde::de::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        let val = serde_json::Value::deserialize(deserializer)?;

        // Object format: {"value": ..., "id": ...}
        if let Some(obj) = val.as_object() {
            let value = obj.get("value").cloned().unwrap_or(serde_json::Value::Null);
            let id = obj
                .get("id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            return Ok(Sb3Field { value, id });
        }

        // Array format: [value] or [value, id]
        if let Some(arr) = val.as_array() {
            if !arr.is_empty() {
                let value = arr[0].clone();
                let id = arr.get(1).and_then(|v| v.as_str()).map(|s| s.to_string());
                return Ok(Sb3Field { value, id });
            }
        }

        Err(serde::de::Error::custom(
            "expected field as object or [value(, id)] array",
        ))
    }
}

impl serde::Serialize for Sb3Field {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        use serde::ser::SerializeTuple;
        if self.id.is_some() {
            let mut tup = serializer.serialize_tuple(2)?;
            tup.serialize_element(&self.value)?;
            tup.serialize_element(self.id.as_ref().unwrap())?;
            tup.end()
        } else {
            let mut tup = serializer.serialize_tuple(1)?;
            tup.serialize_element(&self.value)?;
            tup.end()
        }
    }
}

/// Mutation data for procedure (custom block) definitions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sb3Mutation {
    /// Whether the procedure returns a value.
    #[serde(default, rename = "returns", skip_serializing_if = "Option::is_none")]
    pub returns: Option<String>,

    /// The proc code string (prototype signature).
    #[serde(default, rename = "proccode", skip_serializing_if = "Option::is_none")]
    pub proccode: Option<String>,

    /// Argument names as a space-separated string.
    #[serde(default, rename = "argumentnames", skip_serializing_if = "Option::is_none")]
    pub argument_names: Option<String>,

    /// Argument defaults as a space-separated string.
    #[serde(
        default,
        rename = "argumentdefaults",
        skip_serializing_if = "Option::is_none"
    )]
    pub argument_defaults: Option<String>,

    /// Whether the block is warp (run without screen refresh).
    #[serde(default, rename = "warp", skip_serializing_if = "Option::is_none")]
    pub warp: Option<String>,
}

// ---------------------------------------------------------------------------
// Costume & Sound
// ---------------------------------------------------------------------------

/// A costume (appearance) asset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sb3Costume {
    /// The display name.
    #[serde(default, rename = "name")]
    pub name: String,

    /// The MD5 hash + extension of the asset file.
    #[serde(default, rename = "md5ext")]
    pub md5ext: String,

    /// The asset file extension (e.g. "svg", "png").
    #[serde(default, rename = "dataFormat")]
    pub data_format: String,

    /// Rotation center X.
    #[serde(default, rename = "rotationCenterX")]
    pub rotation_center_x: f64,

    /// Rotation center Y.
    #[serde(default, rename = "rotationCenterY")]
    pub rotation_center_y: f64,

    /// Bitmap resolution (for bitmap costumes).
    #[serde(
        default,
        rename = "bitmapResolution",
        skip_serializing_if = "Option::is_none"
    )]
    pub bitmap_resolution: Option<i64>,

    /// The asset ID (used in some Scratch versions).
    #[serde(default, rename = "assetId", skip_serializing_if = "Option::is_none")]
    pub asset_id: Option<String>,
}

impl Default for Sb3Costume {
    fn default() -> Self {
        Sb3Costume {
            name: String::new(),
            md5ext: String::new(),
            data_format: "svg".to_string(),
            rotation_center_x: 0.0,
            rotation_center_y: 0.0,
            bitmap_resolution: None,
            asset_id: None,
        }
    }
}

/// A sound asset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sb3Sound {
    /// The display name.
    #[serde(default, rename = "name")]
    pub name: String,

    /// The MD5 hash + extension of the asset file.
    #[serde(default, rename = "md5ext")]
    pub md5ext: String,

    /// The asset file extension (e.g. "wav", "mp3").
    #[serde(default, rename = "dataFormat")]
    pub data_format: String,

    /// Sample rate in Hz.
    #[serde(default, rename = "rate")]
    pub rate: i64,

    /// Number of samples.
    #[serde(default, rename = "sampleCount")]
    pub sample_count: i64,

    /// The asset ID.
    #[serde(default, rename = "assetId", skip_serializing_if = "Option::is_none")]
    pub asset_id: Option<String>,
}

impl Default for Sb3Sound {
    fn default() -> Self {
        Sb3Sound {
            name: String::new(),
            md5ext: String::new(),
            data_format: "wav".to_string(),
            rate: 48000,
            sample_count: 0,
            asset_id: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Parsing functions
// ---------------------------------------------------------------------------

/// Parse a Scratch 3.0 project from raw `.sb3` ZIP data.
///
/// The function extracts `project.json` from the ZIP archive and
/// deserializes it into an [`Sb3Project`].
#[cfg(feature = "zip")]
pub fn parse_sb3(data: &[u8]) -> Result<Sb3Project> {
    use std::io::Read;

    let reader = std::io::Cursor::new(data);
    let mut archive =
        zip::ZipArchive::new(reader).map_err(|e| ParseError::ZipError(e.to_string()))?;

    let mut project_json_bytes: Option<Vec<u8>> = None;
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| ParseError::ZipError(e.to_string()))?;
        if file.name() == "project.json" {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)
                .map_err(|e| ParseError::IoError(e.to_string()))?;
            project_json_bytes = Some(buf);
            break;
        }
    }

    let json_bytes = project_json_bytes.ok_or(ParseError::MissingProjectJson)?;
    let json_str = String::from_utf8(json_bytes)
        .map_err(|e| ParseError::JsonError(format!("invalid UTF-8 in project.json: {e}")))?;

    parse_sb3_json(&json_str)
}

/// Parse a Scratch 3.0 project from a raw JSON string.
///
/// This is useful for testing and for situations where the `project.json`
/// has already been extracted from the ZIP archive.
pub fn parse_sb3_json(json: &str) -> Result<Sb3Project> {
    let project: Sb3Project = serde_json::from_str(json)?;
    Ok(project)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_project() {
        let json = r#"{
            "targets": [],
            "extensions": [],
            "meta": {
                "semver": "3.0.0",
                "vm": "0.1.0",
                "agent": "test"
            }
        }"#;
        let project = parse_sb3_json(json).unwrap();
        assert!(project.targets.is_empty());
        assert!(project.extensions.is_empty());
        assert_eq!(project.meta.semver, "3.0.0");
        assert_eq!(project.meta.vm, "0.1.0");
        assert_eq!(project.meta.agent, "test");
    }

    #[test]
    fn test_parse_project_with_stage() {
        let json = r#"{
            "targets": [{
                "isStage": true,
                "name": "Stage",
                "variables": {
                    "var1": ["my var", 0]
                },
                "lists": {
                    "list1": ["my list", [1, 2, 3]]
                },
                "blocks": {},
                "costumes": [
                    {
                        "name": "backdrop1",
                        "md5ext": "abc.svg",
                        "dataFormat": "svg",
                        "rotationCenterX": 240,
                        "rotationCenterY": 180
                    }
                ],
                "sounds": [],
                "currentCostume": 0,
                "layerOrder": 0,
                "volume": 100,
                "tempo": 60
            }],
            "extensions": ["pen"],
            "meta": {
                "semver": "3.0.0",
                "vm": "",
                "agent": ""
            }
        }"#;
        let project = parse_sb3_json(json).unwrap();
        assert_eq!(project.targets.len(), 1);
        assert_eq!(project.extensions, vec!["pen"]);

        let stage_target = &project.targets[0];
        assert!(stage_target.is_stage);
        assert_eq!(stage_target.name, "Stage");
        assert!(stage_target.variables.contains_key("var1"));
        assert!(stage_target.lists.contains_key("list1"));

        // Check variable parsed from array format
        let var = &stage_target.variables["var1"];
        assert_eq!(var.name, "my var");
        assert_eq!(var.value, 0);

        // Check list parsed from array format
        let list = &stage_target.lists["list1"];
        assert_eq!(list.name, "my list");
        assert_eq!(list.values.len(), 3);

        assert_eq!(stage_target.costumes.len(), 1);
        assert_eq!(stage_target.costumes[0].name, "backdrop1");
        assert_eq!(stage_target.costumes[0].rotation_center_x, 240.0);
        assert_eq!(stage_target.costumes[0].rotation_center_y, 180.0);
        assert_eq!(stage_target.volume, Some(100));
        assert_eq!(stage_target.tempo, Some(60));
    }

    #[test]
    fn test_parse_project_with_sprite_and_blocks() {
        let json = r#"{
            "targets": [{
                "isStage": false,
                "name": "Sprite1",
                "variables": {},
                "lists": {},
                "blocks": {
                    "block1": {
                        "opcode": "event_whenflagclicked",
                        "inputs": {},
                        "fields": {},
                        "next": "block2",
                        "parent": null,
                        "shadow": false,
                        "topLevel": true,
                        "x": 100,
                        "y": 200
                    },
                    "block2": {
                        "opcode": "motion_movesteps",
                        "inputs": {
                            "STEPS": ["input_value_rounded", [10, 10]]
                        },
                        "fields": {},
                        "next": null,
                        "parent": "block1",
                        "shadow": false,
                        "topLevel": false
                    }
                },
                "costumes": [
                    {
                        "name": "costume1",
                        "md5ext": "def.svg",
                        "dataFormat": "svg",
                        "rotationCenterX": 50,
                        "rotationCenterY": 50
                    }
                ],
                "sounds": [
                    {
                        "name": "pop",
                        "md5ext": "ghi.wav",
                        "dataFormat": "wav",
                        "rate": 48000,
                        "sampleCount": 1234
                    }
                ],
                "currentCostume": 0,
                "layerOrder": 1,
                "x": 0,
                "y": 0,
                "direction": 90,
                "size": 100,
                "rotationStyle": "all around",
                "visible": true,
                "draggable": false
            }],
            "extensions": [],
            "meta": {
                "semver": "3.0.0"
            }
        }"#;
        let project = parse_sb3_json(json).unwrap();
        assert_eq!(project.targets.len(), 1);

        let sprite_target = &project.targets[0];
        assert!(!sprite_target.is_stage);
        assert_eq!(sprite_target.name, "Sprite1");
        assert_eq!(sprite_target.blocks.len(), 2);

        // Check hat block
        let block1 = &sprite_target.blocks["block1"];
        assert_eq!(block1.opcode, "event_whenflagclicked");
        assert!(block1.top_level);
        assert_eq!(block1.next, Some("block2".to_string()));
        assert_eq!(block1.x, Some(100.0));
        assert_eq!(block1.y, Some(200.0));

        // Check stack block
        let block2 = &sprite_target.blocks["block2"];
        assert_eq!(block2.opcode, "motion_movesteps");
        assert!(!block2.top_level);
        assert_eq!(block2.parent, Some("block1".to_string()));
        assert!(block2.inputs.contains_key("STEPS"));

        // Check sprite-specific fields
        assert_eq!(sprite_target.x, Some(0.0));
        assert_eq!(sprite_target.y, Some(0.0));
        assert_eq!(sprite_target.direction, Some(90.0));
        assert_eq!(sprite_target.size, Some(100.0));
        assert_eq!(
            sprite_target.rotation_style,
            Some("all around".to_string())
        );
        assert_eq!(sprite_target.visible, Some(true));
        assert_eq!(sprite_target.draggable, Some(false));

        // Check sound
        assert_eq!(sprite_target.sounds.len(), 1);
        assert_eq!(sprite_target.sounds[0].name, "pop");
        assert_eq!(sprite_target.sounds[0].rate, 48000);
        assert_eq!(sprite_target.sounds[0].sample_count, 1234);
    }

    #[test]
    fn test_parse_handles_missing_optional_fields() {
        let json = r#"{
            "targets": [{
                "isStage": false,
                "name": "Minimal"
            }]
        }"#;
        let project = parse_sb3_json(json).unwrap();
        assert_eq!(project.targets.len(), 1);
        let target = &project.targets[0];
        assert_eq!(target.name, "Minimal");
        assert!(!target.is_stage);
        assert!(target.variables.is_empty());
        assert!(target.lists.is_empty());
        assert!(target.blocks.is_empty());
        assert!(target.costumes.is_empty());
        assert!(target.sounds.is_empty());
        assert_eq!(target.current_costume, 0);
        assert!(target.x.is_none());
        assert!(target.y.is_none());
        assert!(target.visible.is_none());
    }

    #[test]
    fn test_parse_invalid_json_returns_error() {
        let result = parse_sb3_json("not valid json!!!");
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::JsonError(msg) => {
                assert!(msg.contains("expected") || msg.contains("invalid"));
            }
            other => panic!("expected JsonError, got: {other}"),
        }
    }

    #[test]
    fn test_parse_cloud_variable() {
        let json = r#"{
            "targets": [{
                "isStage": true,
                "name": "Stage",
                "variables": {
                    "cloud1": ["☁ score", 0, true]
                }
            }]
        }"#;
        let project = parse_sb3_json(json).unwrap();
        let var = &project.targets[0].variables["cloud1"];
        assert_eq!(var.name, "☁ score");
        assert!(var.is_cloud);
    }

    #[test]
    fn test_project_default_meta() {
        let json = r#"{
            "targets": []
        }"#;
        let project = parse_sb3_json(json).unwrap();
        assert_eq!(project.meta.semver, "3.0.0");
        assert!(project.meta.vm.is_empty());
        assert!(project.meta.agent.is_empty());
    }

    #[test]
    fn test_sb3_block_default() {
        let block = Sb3Block::default();
        assert!(block.opcode.is_empty());
        assert!(block.inputs.is_empty());
        assert!(block.fields.is_empty());
        assert!(block.next.is_none());
        assert!(block.parent.is_none());
        assert!(!block.shadow);
        assert!(!block.top_level);
    }

    #[test]
    fn test_sb3_costume_default() {
        let costume = Sb3Costume::default();
        assert!(costume.name.is_empty());
        assert_eq!(costume.data_format, "svg");
        assert_eq!(costume.rotation_center_x, 0.0);
        assert_eq!(costume.rotation_center_y, 0.0);
    }

    #[test]
    fn test_variable_object_format() {
        let json = r#"{
            "targets": [{
                "isStage": true,
                "name": "Stage",
                "variables": {
                    "v1": {"name": "score", "value": 42, "is_cloud": false}
                }
            }]
        }"#;
        let project = parse_sb3_json(json).unwrap();
        let var = &project.targets[0].variables["v1"];
        assert_eq!(var.name, "score");
        assert_eq!(var.value, 42);
        assert!(!var.is_cloud);
    }

    #[test]
    fn test_list_object_format() {
        let json = r#"{
            "targets": [{
                "isStage": true,
                "name": "Stage",
                "lists": {
                    "l1": {"name": "items", "values": ["a", "b", "c"]}
                }
            }]
        }"#;
        let project = parse_sb3_json(json).unwrap();
        let list = &project.targets[0].lists["l1"];
        assert_eq!(list.name, "items");
        assert_eq!(list.values.len(), 3);
    }

    #[test]
    fn test_block_with_mutation() {
        let json = r#"{
            "targets": [{
                "isStage": false,
                "name": "Sprite1",
                "blocks": {
                    "proc1": {
                        "opcode": "procedures_definition",
                        "inputs": {},
                        "fields": {},
                        "next": null,
                        "parent": null,
                        "shadow": false,
                        "topLevel": true,
                        "mutation": {
                            "proccode": "my block %s",
                            "argumentnames": "[\"x\"]",
                            "argumentdefaults": "[\"\"]",
                            "warp": "true"
                        }
                    }
                }
            }]
        }"#;
        let project = parse_sb3_json(json).unwrap();
        let block = &project.targets[0].blocks["proc1"];
        assert!(block.mutation.is_some());
        let mutation = block.mutation.as_ref().unwrap();
        assert_eq!(mutation.proccode, Some("my block %s".to_string()));
        assert_eq!(mutation.warp, Some("true".to_string()));
    }
}
