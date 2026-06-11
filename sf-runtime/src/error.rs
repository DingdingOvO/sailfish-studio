use thiserror::Error;

/// Main error type for the sf-runtime CLI.
#[derive(Error, Debug)]
pub enum SfError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("Walk error: {0}")]
    Walk(#[from] walkdir::Error),

    #[error("Project not found: {0}")]
    ProjectNotFound(String),

    #[error("Invalid file format: expected {expected}, got {actual}")]
    InvalidFormat { expected: String, actual: String },

    #[error("Missing field: {0}")]
    MissingField(String),

    #[error("Invalid project name: {0}")]
    InvalidName(String),

    #[error("Project already exists: {0}")]
    AlreadyExists(String),

    #[error("Syntax error in {file} at line {line}: {message}")]
    SyntaxError {
        file: String,
        line: usize,
        message: String,
    },

    #[error("Syntax warning in {file} at line {line}: {message}")]
    SyntaxWarning {
        file: String,
        line: usize,
        message: String,
    },

    #[error("Headed mode requires a display server")]
    NoDisplay,

    #[error("Runtime error: {0}")]
    Runtime(String),

    #[error("Package error: {0}")]
    Package(String),

    #[error("{0}")]
    Custom(String),
}

pub type Result<T> = std::result::Result<T, SfError>;
