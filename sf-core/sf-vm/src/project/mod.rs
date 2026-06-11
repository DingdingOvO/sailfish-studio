//! Project data structures for the Sailfish VM.
//!
//! Contains the core types that represent a block-based program:
//! `Project`, `Target`, `Block`, `Value`, `Costume`, `Sound`.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during project operations.
#[derive(Error, Debug)]
pub enum ProjectError {
    #[error("JSON parse error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("missing required field: {0}")]
    MissingField(String),
    #[error("invalid project structure: {0}")]
    InvalidStructure(String),
}

/// Top-level project structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub targets: Vec<Target>,
    pub extensions: Vec<String>,
    pub settings: ProjectSettings,
}

/// Project-level settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    pub stage_width: u32,
    pub stage_height: u32,
    pub fps: u32,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            stage_width: 480,
            stage_height: 360,
            fps: 30,
        }
    }
}

impl Project {
    /// Create a new empty project with the given name.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            targets: Vec::new(),
            extensions: Vec::new(),
            settings: ProjectSettings::default(),
        }
    }

    /// Load a project from a JSON string.
    ///
    /// The JSON can be in Scratch 3.0 project format or a simplified Sailfish format.
    pub fn load_from_json(json: &str) -> Result<Self, ProjectError> {
        // Try direct Sailfish project format first
        if let Ok(project) = serde_json::from_str::<Project>(json) {
            return Ok(project);
        }

        // Try Scratch 3.0 format
        let value: serde_json::Value = serde_json::from_str(json)?;
        if let Some(targets_arr) = value.get("targets") {
            return Self::from_scratch3(&value, targets_arr);
        }

        Err(ProjectError::InvalidStructure(
            "could not parse as Sailfish or Scratch 3.0 format".to_string(),
        ))
    }

    /// Parse from Scratch 3.0 JSON format.
    fn from_scratch3(
        root: &serde_json::Value,
        targets_arr: &serde_json::Value,
    ) -> Result<Self, ProjectError> {
        let name = root
            .get("meta")
            .and_then(|m| m.get("projectName"))
            .and_then(|n| n.as_str())
            .unwrap_or("Untitled")
            .to_string();

        let extensions = root
            .get("extensions")
            .and_then(|e| e.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let targets = targets_arr
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|t| Target::from_scratch3_value(t).ok())
                    .collect()
            })
            .unwrap_or_default();

        Ok(Self {
            name,
            targets,
            extensions,
            settings: ProjectSettings::default(),
        })
    }

    /// Get the stage target, if any.
    pub fn stage(&self) -> Option<&Target> {
        self.targets.iter().find(|t| t.is_stage)
    }

    /// Get a mutable reference to the stage target, if any.
    pub fn stage_mut(&mut self) -> Option<&mut Target> {
        self.targets.iter_mut().find(|t| t.is_stage)
    }

    /// Get a sprite target by name.
    pub fn sprite(&self, name: &str) -> Option<&Target> {
        self.targets.iter().find(|t| !t.is_stage && t.name == name)
    }

    /// Get a target (stage or sprite) by name.
    pub fn target_by_name(&self, name: &str) -> Option<&Target> {
        self.targets.iter().find(|t| t.name == name)
    }
}

/// A target (sprite or stage) in the project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    pub name: String,
    pub variables: HashMap<String, Value>,
    pub lists: HashMap<String, Vec<Value>>,
    pub blocks: HashMap<String, Block>,
    pub costumes: Vec<Costume>,
    pub sounds: Vec<Sound>,
    pub is_stage: bool,
    pub x: f64,
    pub y: f64,
    pub direction: f64,
    pub size: f64,
    pub visible: bool,
    pub current_costume: usize,
}

impl Target {
    /// Create a new stage target.
    pub fn new_stage() -> Self {
        Self {
            name: "Stage".to_string(),
            variables: HashMap::new(),
            lists: HashMap::new(),
            blocks: HashMap::new(),
            costumes: Vec::new(),
            sounds: Vec::new(),
            is_stage: true,
            x: 0.0,
            y: 0.0,
            direction: 90.0,
            size: 100.0,
            visible: true,
            current_costume: 0,
        }
    }

    /// Create a new sprite target with the given name.
    pub fn new_sprite(name: &str) -> Self {
        Self {
            name: name.to_string(),
            variables: HashMap::new(),
            lists: HashMap::new(),
            blocks: HashMap::new(),
            costumes: Vec::new(),
            sounds: Vec::new(),
            is_stage: false,
            x: 0.0,
            y: 0.0,
            direction: 90.0,
            size: 100.0,
            visible: true,
            current_costume: 0,
        }
    }

    /// Parse a target from a Scratch 3.0 JSON value.
    fn from_scratch3_value(value: &serde_json::Value) -> Result<Self, ProjectError> {
        let name = value
            .get("name")
            .and_then(|n| n.as_str())
            .ok_or_else(|| ProjectError::MissingField("name".to_string()))?
            .to_string();

        let is_stage = value
            .get("isStage")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mut variables = HashMap::new();
        if let Some(vars) = value.get("variables").and_then(|v| v.as_object()) {
            for (_id, var_data) in vars {
                if let Some(arr) = var_data.as_array() {
                    if arr.len() >= 2 {
                        let var_name = arr[0].as_str().unwrap_or("").to_string();
                        let var_val = Value::from_json(&arr[1]);
                        variables.insert(var_name, var_val);
                    }
                }
            }
        }

        let blocks = HashMap::new();
        if let Some(_blocks_val) = value.get("blocks") {
            // Parse blocks from Scratch format
            if let Some(blocks_obj) = value.get("blocks").and_then(|v| v.as_object()) {
                // We'll skip full block parsing here for simplicity;
                // the compiler will handle it separately.
                let _ = blocks_obj;
            }
        }

        let mut costumes = Vec::new();
        if let Some(costumes_arr) = value.get("costumes").and_then(|v| v.as_array()) {
            for c in costumes_arr {
                if let Some(costume) = Costume::from_json(c) {
                    costumes.push(costume);
                }
            }
        }

        let mut sounds = Vec::new();
        if let Some(sounds_arr) = value.get("sounds").and_then(|v| v.as_array()) {
            for s in sounds_arr {
                if let Some(sound) = Sound::from_json(s) {
                    sounds.push(sound);
                }
            }
        }

        Ok(Self {
            name,
            variables,
            lists: HashMap::new(),
            blocks,
            costumes,
            sounds,
            is_stage,
            x: 0.0,
            y: 0.0,
            direction: 90.0,
            size: 100.0,
            visible: true,
            current_costume: 0,
        })
    }
}

/// A block in the program.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub id: String,
    pub opcode: String,
    pub inputs: HashMap<String, BlockInput>,
    pub fields: HashMap<String, BlockField>,
    pub next: Option<String>,
    pub parent: Option<String>,
    pub top_level: bool,
    pub shadow: bool,
}

/// A block input (references to other blocks or values).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockInput {
    pub input_type: String,
    pub value: Option<Value>,
    pub block_id: Option<String>,
}

/// A block field (dropdown or text field value).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockField {
    pub value: String,
    pub field_id: Option<String>,
}

impl Block {
    /// Create a new block with the given ID and opcode.
    pub fn new(id: &str, opcode: &str) -> Self {
        Self {
            id: id.to_string(),
            opcode: opcode.to_string(),
            inputs: HashMap::new(),
            fields: HashMap::new(),
            next: None,
            parent: None,
            top_level: false,
            shadow: false,
        }
    }

    /// Create a new top-level block (hat block).
    pub fn new_top_level(id: &str, opcode: &str) -> Self {
        Self {
            id: id.to_string(),
            opcode: opcode.to_string(),
            inputs: HashMap::new(),
            fields: HashMap::new(),
            next: None,
            parent: None,
            top_level: true,
            shadow: false,
        }
    }

    /// Set the next block in the chain.
    pub fn with_next(mut self, next: &str) -> Self {
        self.next = Some(next.to_string());
        self
    }

    /// Set the parent block.
    pub fn with_parent(mut self, parent: &str) -> Self {
        self.parent = Some(parent.to_string());
        self
    }

    /// Add an input to the block.
    pub fn with_input(mut self, name: &str, input: BlockInput) -> Self {
        self.inputs.insert(name.to_string(), input);
        self
    }

    /// Add a field to the block.
    pub fn with_field(mut self, name: &str, value: &str) -> Self {
        self.fields.insert(
            name.to_string(),
            BlockField {
                value: value.to_string(),
                field_id: None,
            },
        );
        self
    }

    /// Get a numeric input value.
    pub fn get_input_number(&self, name: &str) -> f64 {
        self.inputs
            .get(name)
            .and_then(|i| i.value.as_ref())
            .and_then(|v| v.as_number())
            .unwrap_or(0.0)
    }

    /// Get a string input value.
    pub fn get_input_string(&self, name: &str) -> Option<String> {
        self.inputs
            .get(name)
            .and_then(|i| i.value.as_ref())
            .and_then(|v| v.as_string())
    }

    /// Get a field value.
    pub fn get_field(&self, name: &str) -> Option<String> {
        self.fields.get(name).map(|f| f.value.clone())
    }
}

/// A value that can be stored and manipulated in the VM.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Null,
    List(Vec<Value>),
}

impl Value {
    /// Parse a value from a JSON value.
    pub fn from_json(v: &serde_json::Value) -> Self {
        match v {
            serde_json::Value::Number(n) => {
                Value::Number(n.as_f64().unwrap_or(0.0))
            }
            serde_json::Value::String(s) => Value::String(s.clone()),
            serde_json::Value::Bool(b) => Value::Bool(*b),
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Array(arr) => {
                Value::List(arr.iter().map(Value::from_json).collect())
            }
            _ => Value::Null,
        }
    }

    /// Try to get a number from this value.
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            Value::String(s) => s.parse::<f64>().ok(),
            Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            Value::Null => Some(0.0),
            Value::List(_) => None,
        }
    }

    /// Try to get a string from this value.
    pub fn as_string(&self) -> Option<String> {
        match self {
            Value::Number(n) => Some(format_number(*n)),
            Value::String(s) => Some(s.clone()),
            Value::Bool(b) => Some(if *b {
                "true".to_string()
            } else {
                "false".to_string()
            }),
            Value::Null => Some(String::new()),
            Value::List(_) => None,
        }
    }

    /// Try to get a bool from this value.
    pub fn as_bool(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Null => false,
            Value::List(v) => !v.is_empty(),
        }
    }

    /// Is this value null?
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }
}

/// Format a number, removing trailing zeros for integers.
fn format_number(n: f64) -> String {
    if n == n.floor() && n.is_finite() {
        format!("{}", n as i64)
    } else {
        format!("{}", n)
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::List(a), Value::List(b)) => a == b,
            // Cross-type comparisons following Scratch semantics
            (Value::Number(a), Value::String(b)) => {
                if let Ok(n) = b.parse::<f64>() {
                    *a == n
                } else {
                    false
                }
            }
            (Value::String(a), Value::Number(b)) => {
                if let Ok(n) = a.parse::<f64>() {
                    n == *b
                } else {
                    false
                }
            }
            (Value::Bool(a), Value::Number(b)) => {
                (if *a { 1.0 } else { 0.0 }) == *b
            }
            (Value::Number(a), Value::Bool(b)) => {
                *a == (if *b { 1.0 } else { 0.0 })
            }
            _ => false,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", format_number(*n)),
            Value::String(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Null => write!(f, ""),
            Value::List(items) => {
                let strs: Vec<String> = items.iter().map(|v| v.to_string()).collect();
                write!(f, "[{}]", strs.join(", "))
            }
        }
    }
}

/// A costume asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Costume {
    pub name: String,
    pub asset_id: String,
    pub data_format: String,
    pub rotation_center_x: f64,
    pub rotation_center_y: f64,
}

impl Costume {
    /// Create a new costume.
    pub fn new(name: &str, asset_id: &str, data_format: &str) -> Self {
        Self {
            name: name.to_string(),
            asset_id: asset_id.to_string(),
            data_format: data_format.to_string(),
            rotation_center_x: 0.0,
            rotation_center_y: 0.0,
        }
    }

    /// Parse from a JSON value.
    fn from_json(v: &serde_json::Value) -> Option<Self> {
        Some(Self {
            name: v.get("name")?.as_str()?.to_string(),
            asset_id: v
                .get("assetId")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            data_format: v
                .get("dataFormat")
                .and_then(|v| v.as_str())
                .unwrap_or("png")
                .to_string(),
            rotation_center_x: v
                .get("rotationCenterX")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            rotation_center_y: v
                .get("rotationCenterY")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
        })
    }
}

/// A sound asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sound {
    pub name: String,
    pub asset_id: String,
    pub data_format: String,
    pub sample_rate: u32,
    pub sample_count: u32,
}

impl Sound {
    /// Create a new sound.
    pub fn new(name: &str, asset_id: &str, data_format: &str) -> Self {
        Self {
            name: name.to_string(),
            asset_id: asset_id.to_string(),
            data_format: data_format.to_string(),
            sample_rate: 44100,
            sample_count: 0,
        }
    }

    /// Parse from a JSON value.
    fn from_json(v: &serde_json::Value) -> Option<Self> {
        Some(Self {
            name: v.get("name")?.as_str()?.to_string(),
            asset_id: v
                .get("assetId")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            data_format: v
                .get("dataFormat")
                .and_then(|v| v.as_str())
                .unwrap_or("wav")
                .to_string(),
            sample_rate: v
                .get("rate")
                .and_then(|v| v.as_u64())
                .unwrap_or(44100) as u32,
            sample_count: v
                .get("sampleCount")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_new() {
        let project = Project::new("TestProject");
        assert_eq!(project.name, "TestProject");
        assert!(project.targets.is_empty());
        assert!(project.extensions.is_empty());
        assert_eq!(project.settings.stage_width, 480);
        assert_eq!(project.settings.stage_height, 360);
        assert_eq!(project.settings.fps, 30);
    }

    #[test]
    fn test_project_load_from_json_sailfish_format() {
        let json = r#"{
            "name": "MyProject",
            "targets": [],
            "extensions": ["pen"],
            "settings": {
                "stage_width": 480,
                "stage_height": 360,
                "fps": 30
            }
        }"#;
        let project = Project::load_from_json(json).expect("should parse");
        assert_eq!(project.name, "MyProject");
        assert!(project.targets.is_empty());
        assert_eq!(project.extensions, vec!["pen"]);
    }

    #[test]
    fn test_project_load_from_json_scratch3_format() {
        let json = r#"{
            "targets": [
                {
                    "isStage": true,
                    "name": "Stage",
                    "variables": {},
                    "blocks": {},
                    "costumes": [],
                    "sounds": []
                },
                {
                    "isStage": false,
                    "name": "Sprite1",
                    "variables": {
                        "var1": ["myVar", 42]
                    },
                    "blocks": {},
                    "costumes": [
                        {
                            "name": "costume1",
                            "assetId": "abc123",
                            "dataFormat": "svg",
                            "rotationCenterX": 50,
                            "rotationCenterY": 50
                        }
                    ],
                    "sounds": []
                }
            ],
            "extensions": ["pen"],
            "meta": {
                "projectName": "ScratchProject"
            }
        }"#;
        let project = Project::load_from_json(json).expect("should parse Scratch3");
        assert_eq!(project.name, "ScratchProject");
        assert_eq!(project.targets.len(), 2);
        assert!(project.targets[0].is_stage);
        assert_eq!(project.targets[0].name, "Stage");
        assert!(!project.targets[1].is_stage);
        assert_eq!(project.targets[1].name, "Sprite1");
        assert!(project.targets[1].variables.contains_key("myVar"));
        assert_eq!(project.targets[1].costumes.len(), 1);
    }

    #[test]
    fn test_project_stage_accessors() {
        let mut project = Project::new("Test");
        project.targets.push(Target::new_stage());
        project.targets.push(Target::new_sprite("Cat"));

        assert!(project.stage().is_some());
        assert_eq!(project.stage().unwrap().name, "Stage");
        assert!(project.sprite("Cat").is_some());
        assert!(project.sprite("Dog").is_none());
        assert!(project.target_by_name("Stage").is_some());
        assert!(project.target_by_name("Cat").is_some());
    }

    #[test]
    fn test_target_new_stage() {
        let stage = Target::new_stage();
        assert_eq!(stage.name, "Stage");
        assert!(stage.is_stage);
        assert!(stage.blocks.is_empty());
        assert!(stage.variables.is_empty());
    }

    #[test]
    fn test_target_new_sprite() {
        let sprite = Target::new_sprite("Cat");
        assert_eq!(sprite.name, "Cat");
        assert!(!sprite.is_stage);
        assert_eq!(sprite.direction, 90.0);
        assert_eq!(sprite.size, 100.0);
        assert!(sprite.visible);
    }

    #[test]
    fn test_block_new() {
        let block = Block::new("block1", "motion_forward");
        assert_eq!(block.id, "block1");
        assert_eq!(block.opcode, "motion_forward");
        assert!(block.next.is_none());
        assert!(block.parent.is_none());
        assert!(!block.top_level);
        assert!(!block.shadow);
    }

    #[test]
    fn test_block_builder_pattern() {
        let block = Block::new_top_level("hat1", "event_whenflagclicked")
            .with_next("block1")
            .with_field("KEY_OPTION", "space");

        assert!(block.top_level);
        assert_eq!(block.next.as_deref(), Some("block1"));
        assert_eq!(block.get_field("KEY_OPTION").as_deref(), Some("space"));
    }

    #[test]
    fn test_block_inputs_and_fields() {
        let block = Block::new("block1", "motion_forward")
            .with_input(
                "STEPS",
                BlockInput {
                    input_type: "shadow".to_string(),
                    value: Some(Value::Number(10.0)),
                    block_id: None,
                },
            )
            .with_field("VARIABLE", "x");

        assert_eq!(block.get_input_number("STEPS"), 10.0);
        assert_eq!(block.get_field("VARIABLE").as_deref(), Some("x"));
        assert_eq!(block.get_input_number("MISSING"), 0.0);
    }

    #[test]
    fn test_value_number() {
        let v = Value::Number(42.5);
        assert_eq!(v.as_number(), Some(42.5));
        assert_eq!(v.as_string(), Some("42.5".to_string()));
        assert!(v.as_bool());
    }

    #[test]
    fn test_value_string() {
        let v = Value::String("hello".to_string());
        assert_eq!(v.as_string(), Some("hello".to_string()));
        assert!(v.as_bool());
    }

    #[test]
    fn test_value_string_as_number() {
        let v = Value::String("3.14".to_string());
        assert_eq!(v.as_number(), Some(3.14));
    }

    #[test]
    fn test_value_bool() {
        let v = Value::Bool(true);
        assert!(v.as_bool());
        assert_eq!(v.as_number(), Some(1.0));
        let v2 = Value::Bool(false);
        assert!(!v2.as_bool());
        assert_eq!(v2.as_number(), Some(0.0));
    }

    #[test]
    fn test_value_null() {
        let v = Value::Null;
        assert!(v.is_null());
        assert!(!v.as_bool());
        assert_eq!(v.as_number(), Some(0.0));
    }

    #[test]
    fn test_value_list() {
        let v = Value::List(vec![Value::Number(1.0), Value::String("hi".to_string())]);
        assert!(v.as_bool());
        let empty = Value::List(vec![]);
        assert!(!empty.as_bool());
    }

    #[test]
    fn test_value_equality() {
        assert_eq!(Value::Number(1.0), Value::Number(1.0));
        assert_eq!(Value::String("a".to_string()), Value::String("a".to_string()));
        assert_eq!(Value::Bool(true), Value::Bool(true));
        assert_eq!(Value::Null, Value::Null);
        // Cross-type
        assert_eq!(Value::Number(1.0), Value::Bool(true));
        assert_eq!(Value::String("5".to_string()), Value::Number(5.0));
    }

    #[test]
    fn test_value_display() {
        assert_eq!(format!("{}", Value::Number(42.0)), "42");
        assert_eq!(format!("{}", Value::Number(3.14)), "3.14");
        assert_eq!(format!("{}", Value::String("hi".to_string())), "hi");
        assert_eq!(format!("{}", Value::Bool(true)), "true");
        assert_eq!(format!("{}", Value::Null), "");
    }

    #[test]
    fn test_costume_new() {
        let c = Costume::new("cat", "abc123", "svg");
        assert_eq!(c.name, "cat");
        assert_eq!(c.asset_id, "abc123");
        assert_eq!(c.data_format, "svg");
    }

    #[test]
    fn test_sound_new() {
        let s = Sound::new("meow", "def456", "wav");
        assert_eq!(s.name, "meow");
        assert_eq!(s.asset_id, "def456");
        assert_eq!(s.sample_rate, 44100);
    }

    #[test]
    fn test_project_load_invalid_json() {
        let result = Project::load_from_json("not json at all");
        assert!(result.is_err());
    }

    #[test]
    fn test_project_stage_mut() {
        let mut project = Project::new("Test");
        project.targets.push(Target::new_stage());
        if let Some(stage) = project.stage_mut() {
            stage.name = "Background".to_string();
        }
        assert_eq!(project.stage().unwrap().name, "Background");
    }
}
