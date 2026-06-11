//! # sf-parser
//!
//! Parsing library for Sailfish Studio project files.
//!
//! Supports three formats:
//! - **`.sb3`** – Scratch 3.0 ZIP archives containing `project.json`
//! - **`.sf`** – Sailfish SQLite-based project files
//! - **`.sfl`** – Sailfish text language (human-readable source code)
//!
//! # Quick start
//!
//! ```rust,ignore
//! use sf_parser::sb3::parse_sb3_json;
//! use sf_parser::sf::parse_sf_metadata_from_json;
//! use sf_parser::sfl::{tokenize, parse as parse_sfl};
//!
//! // Parse a Scratch project from JSON
//! let sb3_project = parse_sb3_json(r#"{"targets": []}"#)?;
//!
//! // Parse a Sailfish project from JSON
//! let sf_project = parse_sf_metadata_from_json(r#"{"metadata": {"name": "Test"}}"#)?;
//!
//! // Parse .sfl source code
//! let sfl_ast = parse_sfl("let x = 42;")?;
//! ```

pub mod error;
pub mod sb3;
pub mod sf;
pub mod sfl;

// Re-export the most commonly used types and functions.
pub use error::{ParseError, Result};

// sb3 re-exports
pub use sb3::{
    parse_sb3_json, Sb3Block, Sb3Costume, Sb3Field, Sb3Input, Sb3List, Sb3Meta, Sb3Mutation,
    Sb3Project, Sb3Sound, Sb3Target, Sb3Variable,
};

// sf re-exports
pub use sf::{
    parse_sf_metadata_from_json, validate_sf_project, SfAsset, SfBlock, SfBlockField,
    SfBlockInput, SfList, SfMetadata, SfProject, SfSettings, SfTarget, SfVariable,
};

// sfl re-exports
pub use sfl::{tokenize, parse, Parser, SflExpr, SflProgram, SflStatement, SflToken};

/// Detect the format of a project file based on its extension and magic bytes.
///
/// Returns a string identifying the format: "sb3", "sf", or "sfl".
pub fn detect_format(filename: &str, data: &[u8]) -> Option<String> {
    // Check by extension first
    let lower = filename.to_lowercase();
    if lower.ends_with(".sb3") {
        return Some("sb3".to_string());
    }
    if lower.ends_with(".sf") {
        return Some("sf".to_string());
    }
    if lower.ends_with(".sfl") {
        return Some("sfl".to_string());
    }

    // Check by magic bytes
    if data.len() >= 4 {
        // ZIP files start with PK\x03\x04
        if &data[0..4] == b"PK\x03\x04" {
            return Some("sb3".to_string());
        }
        // SQLite files start with "SQLite format 3\000"
        if data.len() >= 16 && &data[0..16] == b"SQLite format 3\x00" {
            return Some("sf".to_string());
        }
    }

    // If it looks like text, try .sfl
    if !data.is_empty() && data.iter().all(|&b| b == 0 || (b >= 0x20 && b < 0x7f) || b == b'\n' || b == b'\r' || b == b'\t') {
        return Some("sfl".to_string());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_sb3_by_extension() {
        assert_eq!(
            detect_format("project.sb3", b"anything"),
            Some("sb3".to_string())
        );
    }

    #[test]
    fn test_detect_sf_by_extension() {
        assert_eq!(
            detect_format("project.sf", b"anything"),
            Some("sf".to_string())
        );
    }

    #[test]
    fn test_detect_sfl_by_extension() {
        assert_eq!(
            detect_format("project.sfl", b"anything"),
            Some("sfl".to_string())
        );
    }

    #[test]
    fn test_detect_sb3_by_magic_bytes() {
        let zip_magic = b"PK\x03\x04rest of data";
        assert_eq!(
            detect_format("unknown", zip_magic),
            Some("sb3".to_string())
        );
    }

    #[test]
    fn test_detect_sf_by_magic_bytes() {
        let mut sqlite_magic = Vec::new();
        sqlite_magic.extend_from_slice(b"SQLite format 3\x00");
        sqlite_magic.extend_from_slice(b"rest of data");
        assert_eq!(
            detect_format("unknown", &sqlite_magic),
            Some("sf".to_string())
        );
    }

    #[test]
    fn test_detect_sfl_by_text_content() {
        let text = b"let x = 42;";
        assert_eq!(
            detect_format("unknown", text),
            Some("sfl".to_string())
        );
    }

    #[test]
    fn test_detect_unknown_format() {
        // Binary data that doesn't match any format
        let binary: &[u8] = &[0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]; // PNG header
        assert_eq!(detect_format("unknown", binary), None);
    }

    #[test]
    fn test_detect_case_insensitive() {
        assert_eq!(
            detect_format("Project.SB3", b"data"),
            Some("sb3".to_string())
        );
        assert_eq!(
            detect_format("Project.SF", b"data"),
            Some("sf".to_string())
        );
    }
}
