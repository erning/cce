use crate::config::Environment;
use crate::error::{CceError, Result};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Helper function to get the config directory from HOME/.config
fn get_home_config_dir() -> Result<PathBuf> {
    let home = env::var("HOME")
        .map_err(|_| CceError::MissingDirectory(PathBuf::from("~")))?;
    Ok(PathBuf::from(home).join(".config").join("cce"))
}

/// Manages loading and listing of environments
pub struct EnvironmentManager {
    config_dir: PathBuf,
}

impl EnvironmentManager {
    /// Create a new environment manager
    pub fn new() -> Result<Self> {
        // Use XDG Base Directory specification
        // Check XDG_CONFIG_HOME first, then fall back to HOME/.config
        let config_dir = if let Ok(xdg_config) = env::var("XDG_CONFIG_HOME") {
            if !xdg_config.trim().is_empty() {
                PathBuf::from(xdg_config).join("cce")
            } else {
                // XDG_CONFIG_HOME is set but empty, fall back to HOME/.config
                get_home_config_dir()?
            }
        } else {
            // XDG_CONFIG_HOME not set, use HOME/.config
            get_home_config_dir()?
        };

        Ok(Self { config_dir })
    }

    /// Get the config directory path
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    /// List all available environments
    pub fn list_environments(&self) -> Result<Vec<Environment>> {
        if !self.config_dir.exists() {
            return Ok(Vec::new());
        }

        if !self.config_dir.is_dir() {
            return Err(CceError::MissingDirectory(self.config_dir.clone()));
        }

        let mut environments = Vec::new();

        // Read all .env files in the config directory
        for entry in fs::read_dir(&self.config_dir).map_err(CceError::Io)? {
            let entry = entry.map_err(CceError::Io)?;
            let path = entry.path();

            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == "env" {
                        let file_name = path
                            .file_name()
                            .and_then(|s| s.to_str())
                            .ok_or_else(|| {
                                CceError::InvalidFormat(path.clone())
                            })?;

                        // Remove .env extension to get the environment name
                        let name = file_name
                            .strip_suffix(".env")
                            .unwrap_or(file_name)
                            .to_string();

                        match Environment::from_file(path, name.clone()) {
                            Ok(env) => environments.push(env),
                            Err(e) => {
                                eprintln!("Warning: Failed to load environment '{}': {}", name, e);
                            }
                        }
                    }
                }
            }
        }

        // Sort by name for consistent output
        environments.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(environments)
    }

    /// Get the environment file path for a specific environment name
    pub fn get_environment_file(&self, name: &str) -> Result<PathBuf> {
        let env_file = self.config_dir.join(format!("{}.env", name));

        if !env_file.exists() {
            return Err(CceError::MissingFile(env_file));
        }

        Ok(env_file)
    }

    /// Load a specific environment by name
    pub fn load_environment(&self, name: &str) -> Result<Environment> {
        let env_file = self.get_environment_file(name)?;
        Environment::from_file(env_file, name.to_string())
    }
}

impl Default for EnvironmentManager {
    fn default() -> Self {
        Self::new().expect("Failed to create environment manager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_list_environments_empty() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = EnvironmentManager {
            config_dir: temp_dir.path().to_path_buf(),
        };
        assert_eq!(manager.list_environments().unwrap().len(), 0);
    }

    #[test]
    fn test_environment_manager_new() {
        let manager = EnvironmentManager::new().unwrap();
        assert!(manager.config_dir().ends_with("cce"));
    }

    #[test]
    fn test_list_environments_with_files() {
        let temp_dir = tempfile::tempdir().unwrap();

        // Create test env files
        let env1_path = temp_dir.path().join("test1.env");
        let env2_path = temp_dir.path().join("test2.env");

        let mut file1 = fs::File::create(&env1_path).unwrap();
        writeln!(file1, "export ANTHROPIC_AUTH_TOKEN=token1").unwrap();

        let mut file2 = fs::File::create(&env2_path).unwrap();
        writeln!(file2, "export ANTHROPIC_AUTH_TOKEN=token2").unwrap();

        let manager = EnvironmentManager {
            config_dir: temp_dir.path().to_path_buf(),
        };

        let envs = manager.list_environments().unwrap();
        assert_eq!(envs.len(), 2);
        assert!(envs.iter().any(|e| e.name == "test1"));
        assert!(envs.iter().any(|e| e.name == "test2"));
    }

    #[test]
    fn test_list_environments_sorted() {
        let temp_dir = tempfile::tempdir().unwrap();

        // Create test env files in reverse order
        for name in &["zebra", "alpha", "middle"] {
            let path = temp_dir.path().join(format!("{}.env", name));
            let mut file = fs::File::create(&path).unwrap();
            writeln!(file, "export ANTHROPIC_AUTH_TOKEN=token").unwrap();
        }

        let manager = EnvironmentManager {
            config_dir: temp_dir.path().to_path_buf(),
        };

        let envs = manager.list_environments().unwrap();
        assert_eq!(envs.len(), 3);
        assert_eq!(envs[0].name, "alpha");
        assert_eq!(envs[1].name, "middle");
        assert_eq!(envs[2].name, "zebra");
    }

    #[test]
    fn test_get_environment_file() {
        let temp_dir = tempfile::tempdir().unwrap();

        let env_path = temp_dir.path().join("myenv.env");
        let mut file = fs::File::create(&env_path).unwrap();
        writeln!(file, "export ANTHROPIC_AUTH_TOKEN=token").unwrap();

        let manager = EnvironmentManager {
            config_dir: temp_dir.path().to_path_buf(),
        };

        let result = manager.get_environment_file("myenv");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), env_path);
    }

    #[test]
    fn test_get_environment_file_not_found() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = EnvironmentManager {
            config_dir: temp_dir.path().to_path_buf(),
        };

        let result = manager.get_environment_file("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_load_environment() {
        let temp_dir = tempfile::tempdir().unwrap();

        let env_path = temp_dir.path().join("loadtest.env");
        let mut file = fs::File::create(&env_path).unwrap();
        writeln!(file, "export ANTHROPIC_AUTH_TOKEN=token").unwrap();

        let manager = EnvironmentManager {
            config_dir: temp_dir.path().to_path_buf(),
        };

        let env = manager.load_environment("loadtest").unwrap();
        assert_eq!(env.name, "loadtest");
        assert_eq!(env.file_path, env_path);
    }

    #[test]
    fn test_ignores_non_env_files() {
        let temp_dir = tempfile::tempdir().unwrap();

        // Create various files
        fs::File::create(temp_dir.path().join("valid.env")).unwrap();
        fs::File::create(temp_dir.path().join("readme.txt")).unwrap();
        fs::File::create(temp_dir.path().join("config.json")).unwrap();

        let manager = EnvironmentManager {
            config_dir: temp_dir.path().to_path_buf(),
        };

        let envs = manager.list_environments().unwrap();
        assert_eq!(envs.len(), 1);
        assert_eq!(envs[0].name, "valid");
    }
}
