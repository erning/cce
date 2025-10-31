use crate::error::{CceError, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

/// Represents a Claude Code environment configuration
#[derive(Debug, Clone)]
pub struct Environment {
    pub name: String,
    pub base_url: String,
    pub auth_token: String,
}

impl Environment {
    /// Validate the environment configuration
    pub fn validate(&self) -> Result<()> {
        if self.base_url.is_empty() {
            return Err(CceError::ValidationFailed(
                "ANTHROPIC_BASE_URL is required".to_string(),
            ));
        }

        if self.auth_token.is_empty() {
            return Err(CceError::ValidationFailed(
                "ANTHROPIC_AUTH_TOKEN is required".to_string(),
            ));
        }

        // Basic URL validation
        if !self.base_url.starts_with("http://") && !self.base_url.starts_with("https://") {
            return Err(CceError::ValidationFailed(
                "ANTHROPIC_BASE_URL must be a valid HTTP or HTTPS URL".to_string(),
            ));
        }

        Ok(())
    }

    /// Load environment from a .env file
    pub fn from_file(path: PathBuf, name: String) -> Result<Self> {
        let file = File::open(&path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                CceError::MissingFile(path.clone())
            } else {
                CceError::Io(e)
            }
        })?;

        let reader = BufReader::new(file);
        let mut vars = HashMap::new();

        for line in reader.lines() {
            let line = line.map_err(CceError::Io)?;
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse KEY=value or export KEY=value
            let (key, value) = parse_line(&line, &path)?;
            vars.insert(key, value);
        }

        let base_url = vars
            .get("ANTHROPIC_BASE_URL")
            .cloned()
            .unwrap_or_default();
        let auth_token = vars
            .get("ANTHROPIC_AUTH_TOKEN")
            .cloned()
            .unwrap_or_default();

        let env = Environment {
            name,
            base_url,
            auth_token,
        };

        env.validate()?;
        Ok(env)
    }
}

/// Parse a single line from a .env file
fn parse_line(line: &str, path: &PathBuf) -> Result<(String, String)> {
    // Support both "KEY=value" and "export KEY=value" formats
    let line = line.strip_prefix("export ").unwrap_or(line);

    if let Some((key, value)) = line.split_once('=') {
        let key = key.trim().to_string();
        let value = value.trim().to_string();
        // Remove surrounding quotes if present
        let value = strip_quotes(&value);
        Ok((key, value))
    } else {
        Err(CceError::InvalidFormat(path.clone()))
    }
}

/// Remove surrounding quotes from a value
fn strip_quotes(s: &str) -> String {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_environment_validation() {
        let env = Environment {
            name: "test".to_string(),
            base_url: "https://example.com".to_string(),
            auth_token: "token".to_string(),
        };
        assert!(env.validate().is_ok());

        let env = Environment {
            name: "test".to_string(),
            base_url: "".to_string(),
            auth_token: "token".to_string(),
        };
        assert!(env.validate().is_err());

        let env = Environment {
            name: "test".to_string(),
            base_url: "invalid-url".to_string(),
            auth_token: "token".to_string(),
        };
        assert!(env.validate().is_err());
    }

    #[test]
    fn test_parse_line() {
        let path = PathBuf::from("test.env");
        let (key, value) = parse_line("KEY=value", &path).unwrap();
        assert_eq!(key, "KEY");
        assert_eq!(value, "value");

        let (key, value) = parse_line("export KEY=value", &path).unwrap();
        assert_eq!(key, "KEY");
        assert_eq!(value, "value");

        let (key, value) = parse_line("KEY=\"quoted value\"", &path).unwrap();
        assert_eq!(key, "KEY");
        assert_eq!(value, "quoted value");
    }
}