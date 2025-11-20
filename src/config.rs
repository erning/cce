use crate::error::{CceError, Result};
use std::path::PathBuf;

/// Represents a Claude Code environment configuration (simplified for direct execution)
#[derive(Debug, Clone)]
pub struct Environment {
    pub name: String,
    pub file_path: PathBuf,
}

impl Environment {
    /// Create a new environment from file path
    pub fn from_file(path: PathBuf, name: String) -> Result<Self> {
        if !path.exists() {
            return Err(CceError::MissingFile(path.clone()));
        }

        Ok(Environment {
            name,
            file_path: path,
        })
    }
}
