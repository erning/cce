use crate::error::Result;
use std::path::Path;
use std::process::Command;

#[cfg(unix)]
use std::os::unix::process::CommandExt;

#[cfg(not(unix))]
use crate::error::CceError;

/// Executes the claude command with environment variables loaded from .env file
pub struct CommandExecutor;

impl CommandExecutor {
    /// Execute claude command with environment variables loaded from the given .env file
    pub fn execute(env_file: &Path, args: &[String]) -> Result<i32> {
        // Build shell command: source the env file and exec claude with args
        let shell_cmd = format!(
            ". '{}' && exec claude {}",
            env_file.to_string_lossy().replace('\'', "'\\''"),
            args.join(" ")
        );

        #[cfg(unix)]
        {
            use std::process::exit;
            
            let mut command = Command::new("sh");
            command.arg("-c").arg(&shell_cmd);
            
            let err = command.exec();
            // exec only returns on error, so we need to handle the error and exit
            let error_code = match err.kind() {
                std::io::ErrorKind::NotFound => {
                    eprintln!("Error: sh command not found");
                    127  // Standard command not found exit code
                }
                _ => {
                    eprintln!("Error executing shell command: {}", err);
                    1
                }
            };
            exit(error_code);
        }

        // Windows fallback: parse env file and run claude with environment variables
        #[cfg(not(unix))]
        {
            // Read and parse the env file
            let content = std::fs::read_to_string(env_file)
                .map_err(|e| CceError::ExecutionFailed(format!("Failed to read env file: {}", e)))?;

            let env_vars = parse_env_file(&content);

            let mut command = Command::new("claude");
            command.args(args);

            for (key, value) in env_vars {
                command.env(key, value);
            }

            let status = command.status().map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    CceError::ClaudeNotFound
                } else {
                    CceError::ExecutionFailed(e.to_string())
                }
            })?;

            Ok(status.code().unwrap_or(1))
        }
    }
}

/// Parse environment file content and extract KEY=VALUE pairs
#[cfg(not(unix))]
fn parse_env_file(content: &str) -> Vec<(String, String)> {
    let mut vars = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // Handle "export KEY=VALUE" or "KEY=VALUE" format
        let line = line.strip_prefix("export ").unwrap_or(line);
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            // Remove surrounding quotes if present
            let value = value
                .strip_prefix('"')
                .and_then(|v| v.strip_suffix('"'))
                .or_else(|| value.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))
                .unwrap_or(value);
            vars.push((key.to_string(), value.to_string()));
        }
    }
    vars
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_executor_creation() {
        let executor = CommandExecutor;
        let _ = executor;
    }

    #[cfg(not(unix))]
    mod windows_tests {
        use super::*;

        #[test]
        fn test_parse_env_file_basic() {
            let content = r#"
export KEY1="value1"
KEY2=value2
"#;
            let vars = parse_env_file(content);
            assert_eq!(vars.len(), 2);
            assert!(vars.iter().any(|(k, v)| k == "KEY1" && v == "value1"));
            assert!(vars.iter().any(|(k, v)| k == "KEY2" && v == "value2"));
        }

        #[test]
        fn test_parse_env_file_with_comments() {
            let content = r#"
# This is a comment
export KEY1="value1"
# Another comment
KEY2=value2
"#;
            let vars = parse_env_file(content);
            assert_eq!(vars.len(), 2);
        }

        #[test]
        fn test_parse_env_file_with_quotes() {
            let content = r#"
KEY1="double quoted"
KEY2='single quoted'
KEY3=unquoted
"#;
            let vars = parse_env_file(content);
            assert_eq!(vars.len(), 3);
            assert!(vars.iter().any(|(k, v)| k == "KEY1" && v == "double quoted"));
            assert!(vars.iter().any(|(k, v)| k == "KEY2" && v == "single quoted"));
            assert!(vars.iter().any(|(k, v)| k == "KEY3" && v == "unquoted"));
        }

        #[test]
        fn test_parse_env_file_empty() {
            let content = "";
            let vars = parse_env_file(content);
            assert!(vars.is_empty());
        }

        #[test]
        fn test_parse_env_file_only_comments() {
            let content = r#"
# Comment 1
# Comment 2
"#;
            let vars = parse_env_file(content);
            assert!(vars.is_empty());
        }
    }
}
