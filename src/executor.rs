use crate::config::Environment;
use crate::error::{CceError, Result};
use std::process::Command;

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

        // Execute the command
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