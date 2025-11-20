use crate::error::{CceError, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

/// Represents a Claude Code environment configuration
#[derive(Debug, Clone)]
pub struct Environment {
    pub name: String,
    pub env_vars: HashMap<String, String>,
}

impl Environment {
    /// Validate the environment configuration
    pub fn validate(&self) -> Result<()> {
        // Validate BASE_URL
        let base_url = self
            .env_vars
            .get("ANTHROPIC_BASE_URL")
            .map(|s| s.as_str())
            .unwrap_or("");

        if base_url.is_empty() {
            return Err(CceError::ValidationFailed(
                "ANTHROPIC_BASE_URL is required".to_string(),
            ));
        }

        // Validate authentication (either ANTHROPIC_AUTH_TOKEN or ANTHROPIC_API_KEY is required)
        let has_auth_token = self
            .env_vars
            .get("ANTHROPIC_AUTH_TOKEN")
            .is_some_and(|v| !v.is_empty());
        let has_api_key = self
            .env_vars
            .get("ANTHROPIC_API_KEY")
            .is_some_and(|v| !v.is_empty());

        if !has_auth_token && !has_api_key {
            return Err(CceError::ValidationFailed(
                "Either ANTHROPIC_AUTH_TOKEN or ANTHROPIC_API_KEY is required".to_string(),
            ));
        }

        // Basic URL validation
        if !base_url.starts_with("http://")
            && !base_url.starts_with("https://")
        {
            return Err(CceError::ValidationFailed(
                "ANTHROPIC_BASE_URL must be a valid HTTP or HTTPS URL"
                    .to_string(),
            ));
        }

        Ok(())
    }

    /// Load environment from a .env file using shell source
    pub fn from_file(path: PathBuf, name: String) -> Result<Self> {
        // Check if file exists
        if !path.exists() {
            return Err(CceError::MissingFile(path.clone()));
        }

        // Build shell command: source the env file and print all environment variables
        // Using 'env' command to print all variables after sourcing
        let shell_cmd = format!(
            "source '{}' && env",
            path.to_string_lossy().replace('\'', "'\\''")
        );

        // Execute the shell command
        let output = Command::new("sh")
            .arg("-c")
            .arg(&shell_cmd)
            .output()
            .map_err(CceError::Io)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CceError::ShellCommandFailed(format!(
                "Failed to source environment file: {}",
                stderr
            )));
        }

        // Parse the env output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut vars = HashMap::new();

        for line in stdout.lines() {
            // Parse KEY=value format
            if let Some((key, value)) = parse_env_line(line) {
                vars.insert(key, value);
            }
        }

        let env = Environment {
            name,
            env_vars: vars,
        };

        env.validate()?;
        Ok(env)
    }
}

/// Parse a line from 'env' command output (format: KEY=value)
fn parse_env_line(line: &str) -> Option<(String, String)> {
    // Skip empty lines
    if line.is_empty() {
        return None;
    }

    // Parse KEY=value format (value may contain '=')
    if let Some((key, value)) = line.split_once('=') {
        let key = key.trim().to_string();
        let value = value.to_string();
        Some((key, value))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_validation() {
        // Test valid environment with ANTHROPIC_AUTH_TOKEN
        let mut env_vars = HashMap::new();
        env_vars.insert(
            "ANTHROPIC_BASE_URL".to_string(),
            "https://example.com".to_string(),
        );
        env_vars
            .insert("ANTHROPIC_AUTH_TOKEN".to_string(), "token".to_string());

        let env = Environment {
            name: "test".to_string(),
            env_vars,
        };
        assert!(env.validate().is_ok());

        // Test valid environment with ANTHROPIC_API_KEY
        let mut env_vars = HashMap::new();
        env_vars.insert(
            "ANTHROPIC_BASE_URL".to_string(),
            "https://example.com".to_string(),
        );
        env_vars
            .insert("ANTHROPIC_API_KEY".to_string(), "key123".to_string());

        let env = Environment {
            name: "test".to_string(),
            env_vars,
        };
        assert!(env.validate().is_ok());

        // Test missing ANTHROPIC_BASE_URL
        let env_vars = HashMap::new();
        let env = Environment {
            name: "test".to_string(),
            env_vars,
        };
        assert!(env.validate().is_err());

        // Test missing both authentication tokens
        let mut env_vars = HashMap::new();
        env_vars.insert(
            "ANTHROPIC_BASE_URL".to_string(),
            "https://example.com".to_string(),
        );

        let env = Environment {
            name: "test".to_string(),
            env_vars,
        };
        assert!(env.validate().is_err());

        // Test invalid URL
        let mut env_vars = HashMap::new();
        env_vars.insert(
            "ANTHROPIC_BASE_URL".to_string(),
            "invalid-url".to_string(),
        );
        env_vars
            .insert("ANTHROPIC_API_KEY".to_string(), "key123".to_string());

        let env = Environment {
            name: "test".to_string(),
            env_vars,
        };
        assert!(env.validate().is_err());
    }

    #[test]
    fn test_parse_env_line() {
        // Test basic KEY=value parsing
        let (key, value) = parse_env_line("KEY=value").unwrap();
        assert_eq!(key, "KEY");
        assert_eq!(value, "value");

        // Test value with equals sign
        let (key, value) = parse_env_line("KEY=value=with=equals").unwrap();
        assert_eq!(key, "KEY");
        assert_eq!(value, "value=with=equals");

        // Test empty line
        assert_eq!(parse_env_line(""), None);

        // Test line without equals sign
        assert_eq!(parse_env_line("INVALID_LINE"), None);
    }
}
