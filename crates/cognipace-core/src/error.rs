//! Error types for CogniPace Core

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CognipaceError {
    #[error("Sensor error: {0}")]
    SensorError(String),

    #[error("Model error: {0}")]
    ModelError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Adaptation error: {0}")]
    AdaptationError(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("ONNX runtime error: {0}")]
    OnnxError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, CognipaceError>;
