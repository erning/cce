use crate::error::{CceError, Result};
use std::fs;
use std::path::PathBuf;

/// Required environment variables for Claude API
const REQUIRED_VARS: &[&str] = &["ANTHROPIC_AUTH_TOKEN"];
const OPTIONAL_VARS: &[&str] = &["ANTHROPIC_BASE_URL"];

/// Represents a Claude Code environment configuration (simplified for direct execution)
#[derive(Debug, Clone)]
pub struct Environment {
    pub name: String,
    pub file_path: PathBuf,
}

/// Result of environment file validation
#[derive(Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub missing_required: Vec<String>,
    pub missing_optional: Vec<String>,
    pub found_vars: Vec<String>,
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

    /// Validate the environment file contains required variables
    pub fn validate(&self) -> Result<ValidationResult> {
        let content =
            fs::read_to_string(&self.file_path).map_err(CceError::Io)?;
        Ok(validate_env_content(&content))
    }
}

/// Parse environment file and extract variable names
fn parse_env_vars(content: &str) -> Vec<String> {
    let mut vars = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // Handle "export KEY=VALUE" or "KEY=VALUE" format
        let line = line.strip_prefix("export ").unwrap_or(line);
        if let Some((key, _)) = line.split_once('=') {
            vars.push(key.trim().to_string());
        }
    }
    vars
}

/// Validate environment file content
fn validate_env_content(content: &str) -> ValidationResult {
    let found_vars = parse_env_vars(content);

    let missing_required: Vec<String> = REQUIRED_VARS
        .iter()
        .filter(|&var| !found_vars.iter().any(|v| v == *var))
        .map(|s| s.to_string())
        .collect();

    let missing_optional: Vec<String> = OPTIONAL_VARS
        .iter()
        .filter(|&var| !found_vars.iter().any(|v| v == *var))
        .map(|s| s.to_string())
        .collect();

    ValidationResult {
        is_valid: missing_required.is_empty(),
        missing_required,
        missing_optional,
        found_vars,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_env_vars() {
        let content = r#"
# Comment
export ANTHROPIC_AUTH_TOKEN="test_token"
ANTHROPIC_BASE_URL=https://api.example.com
OTHER_VAR=value
"#;
        let vars = parse_env_vars(content);
        assert_eq!(vars.len(), 3);
        assert!(vars.contains(&"ANTHROPIC_AUTH_TOKEN".to_string()));
        assert!(vars.contains(&"ANTHROPIC_BASE_URL".to_string()));
        assert!(vars.contains(&"OTHER_VAR".to_string()));
    }

    #[test]
    fn test_validate_env_content_valid() {
        let content = r#"
export ANTHROPIC_AUTH_TOKEN="test_token"
export ANTHROPIC_BASE_URL="https://api.example.com"
"#;
        let result = validate_env_content(content);
        assert!(result.is_valid);
        assert!(result.missing_required.is_empty());
        assert!(result.missing_optional.is_empty());
    }

    #[test]
    fn test_validate_env_content_missing_required() {
        let content = r#"
export ANTHROPIC_BASE_URL="https://api.example.com"
"#;
        let result = validate_env_content(content);
        assert!(!result.is_valid);
        assert!(result
            .missing_required
            .contains(&"ANTHROPIC_AUTH_TOKEN".to_string()));
    }

    #[test]
    fn test_validate_env_content_missing_optional() {
        let content = r#"
export ANTHROPIC_AUTH_TOKEN="test_token"
"#;
        let result = validate_env_content(content);
        assert!(result.is_valid);
        assert!(result
            .missing_optional
            .contains(&"ANTHROPIC_BASE_URL".to_string()));
    }
}
