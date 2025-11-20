use crate::config::Environment;
use crate::error::Result;
use std::process::Command;

#[cfg(unix)]
use std::os::unix::process::CommandExt;

/// Executes the claude command with environment variables
pub struct CommandExecutor;

impl CommandExecutor {
    /// Execute claude command with the given environment and arguments
    pub fn execute(env: &Environment, args: &[String]) -> Result<i32> {
        let mut command = Command::new("claude");
        command.args(args);

        // Set all environment variables from the .env file
        for (key, value) in &env.env_vars {
            command.env(key, value);
        }

        // Use exec on Unix systems to replace current process, fallback to subprocess on Windows
        #[cfg(unix)]
        {
            use std::process::exit;
            
            let err = command.exec();
            // exec only returns on error, so we need to handle the error and exit
            let error_code = match err.kind() {
                std::io::ErrorKind::NotFound => {
                    eprintln!("Error: claude command not found");
                    127  // Standard command not found exit code
                }
                _ => {
                    eprintln!("Error executing claude: {}", err);
                    1
                }
            };
            exit(error_code);
        }

        // Windows fallback: use subprocess (exec not available)
        #[cfg(not(unix))]
        {
            let status = command.status().map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    CceError::ClaudeNotFound
                } else {
                    CceError::ExecutionFailed(e.to_string())
                }
            })?;

            // Return the exit code
            Ok(status.code().unwrap_or(1))
        }
    }
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
