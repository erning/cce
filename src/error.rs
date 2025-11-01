use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CceError {
    #[error("Configuration directory not found: {0}")]
    MissingDirectory(PathBuf),

    #[error("Environment file not found: {0}")]
    MissingFile(PathBuf),

    #[error("Invalid environment file format: {0}")]
    InvalidFormat(PathBuf),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Claude CLI not found. Please install it first.")]
    ClaudeNotFound,

    #[error("Failed to execute command: {0}")]
    ExecutionFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, CceError>;