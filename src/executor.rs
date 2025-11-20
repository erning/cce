use crate::error::Result;
use std::path::Path;
use std::process::Command;

#[cfg(unix)]
use std::os::unix::process::CommandExt;

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

        // Windows fallback: source env file then run claude normally
        #[cfg(not(unix))]
        {
            // First source the env file and capture environment
            let env_output = Command::new("cmd")
                .args([&"/c",&format!("call '{}' && set", env_file.to_string_lossy())])
                .output()
                .map_err(|e| CceError::ExecutionFailed(format!("Failed to source env file: {}", e)))?;
            
            if !env_output.status.success() {
                return Err(CceError::ShellCommandFailed("Failed to source environment file".to_string()));
            }
            
            // Parse environment variables and run claude
            let env_vars = parse_windows_env_output(&String::from_utf8_lossy(&env_output.stdout)
            );
            
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

#[cfg(not(unix))]
fn parse_windows_env_output(output: &str) -> Vec<(String, String)> {
    let mut vars = Vec::new();
    for line in output.lines() {
        if let Some((key, value)) = line.split_once('=') {
            vars.push((key.to_string(), value.to_string()));
        }
    }
    vars
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        let executor = CommandExecutor;
        // Basic test to ensure the executor can be created
        // Just verify it can be instantiated
        let _ = executor;
    }
}
