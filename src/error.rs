use thiserror::Error;

/// Application error types
#[derive(Debug, Error)]
pub enum AppError {
    /// Config file related errors
    #[error("Config error: {0}")]
    Config(String),

    /// Database connection related errors
    #[error("Database connection error: {0}")]
    DatabaseConnection(String),

    /// Docker container related errors
    #[error("Docker error: {0}")]
    Docker(String),

    /// Unknown database type errors
    #[error("Unknown database type: {0}")]
    UnknownDatabaseType(String),

    /// Alias not found errors
    #[error("Alias '{0}' not found")]
    AliasNotFound(String),

    /// I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// YAML serialization/deserialization errors
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// Other errors
    #[error("Error: {0}")]
    Other(String),
}

/// Result type alias
pub type Result<T> = std::result::Result<T, AppError>;
