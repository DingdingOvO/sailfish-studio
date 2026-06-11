//! Custom error types for the sf-parser crate.

use thiserror::Error;

/// The main error type for all parser operations.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ParseError {
    /// Failed to read or extract from a ZIP archive (.sb3).
    #[error("ZIP error: {0}")]
    ZipError(String),

    /// Failed to find project.json inside the ZIP archive.
    #[error("missing project.json in archive")]
    MissingProjectJson,

    /// Failed to deserialize JSON content.
    #[error("JSON deserialization error: {0}")]
    JsonError(String),

    /// Failed to open or read from the SQLite database (.sf).
    #[error("SQLite error: {0}")]
    SqliteError(String),

    /// Missing required table in the SQLite database.
    #[error("missing table '{0}' in database")]
    MissingTable(String),

    /// A required field is missing from the parsed data.
    #[error("missing required field: {0}")]
    MissingField(String),

    /// An invalid value was encountered during parsing.
    #[error("invalid value for field '{field}': {value}")]
    InvalidValue { field: String, value: String },

    /// A tokenization error in the .sfl text format.
    #[error("tokenization error at position {position}: {message}")]
    TokenizeError { position: usize, message: String },

    /// A parsing error in the .sfl text format.
    #[error("parse error at line {line}, column {col}: {message}")]
    ParseSyntaxError {
        line: usize,
        col: usize,
        message: String,
    },

    /// An unexpected token was encountered.
    #[error("unexpected token: expected {expected}, found {found}")]
    UnexpectedToken { expected: String, found: String },

    /// An I/O error occurred.
    #[error("I/O error: {0}")]
    IoError(String),

    /// A generic error with a message.
    #[error("{0}")]
    Other(String),
}

impl From<serde_json::Error> for ParseError {
    fn from(err: serde_json::Error) -> Self {
        ParseError::JsonError(err.to_string())
    }
}

/// Convenience type alias for parser results.
pub type Result<T> = std::result::Result<T, ParseError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_messages() {
        let err = ParseError::ZipError("corrupt archive".to_string());
        assert_eq!(format!("{err}"), "ZIP error: corrupt archive");

        let err = ParseError::MissingProjectJson;
        assert_eq!(format!("{err}"), "missing project.json in archive");

        let err = ParseError::JsonError("unexpected token".to_string());
        assert_eq!(format!("{err}"), "JSON deserialization error: unexpected token");

        let err = ParseError::MissingField("name".to_string());
        assert_eq!(format!("{err}"), "missing required field: name");

        let err = ParseError::InvalidValue {
            field: "opcode".to_string(),
            value: "???".to_string(),
        };
        assert_eq!(format!("{err}"), "invalid value for field 'opcode': ???");

        let err = ParseError::TokenizeError {
            position: 42,
            message: "bad char".to_string(),
        };
        assert_eq!(format!("{err}"), "tokenization error at position 42: bad char");

        let err = ParseError::ParseSyntaxError {
            line: 3,
            col: 10,
            message: "expected semicolon".to_string(),
        };
        assert_eq!(
            format!("{err}"),
            "parse error at line 3, column 10: expected semicolon"
        );

        let err = ParseError::UnexpectedToken {
            expected: "Number".to_string(),
            found: "Identifier".to_string(),
        };
        assert_eq!(
            format!("{err}"),
            "unexpected token: expected Number, found Identifier"
        );

        let err = ParseError::Other("something went wrong".to_string());
        assert_eq!(format!("{err}"), "something went wrong");
    }

    #[test]
    fn test_from_serde_json_error() {
        let json_err: serde_json::Error = serde_json::from_str::<i32>("not a number").unwrap_err();
        let parse_err: ParseError = json_err.into();
        assert!(matches!(parse_err, ParseError::JsonError(_)));
        assert!(parse_err.to_string().contains("expected"));
    }

    #[test]
    fn test_result_type_alias() {
        fn returns_ok() -> Result<String> {
            Ok("success".to_string())
        }
        fn returns_err() -> Result<String> {
            Err(ParseError::MissingField("test".to_string()))
        }
        assert!(returns_ok().is_ok());
        assert!(returns_err().is_err());
    }

    #[test]
    fn test_error_clone_and_equality() {
        let err1 = ParseError::MissingField("name".to_string());
        let err2 = err1.clone();
        assert_eq!(err1, err2);
    }
}
