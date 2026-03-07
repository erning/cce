use clap::Parser;
use std::io::Write;
use std::process::Command;

use cce::config::Environment;
use cce::error::{CceError, Result};
use cce::executor::CommandExecutor;
use cce::manager::EnvironmentManager;

#[derive(Parser, Debug)]
#[command(name = "cce")]
#[command(about = "Claude Code Environment Manager")]
#[command(version, disable_version_flag = true, disable_help_flag = true)]
struct Cli {
    /// Print version
    #[arg(long)]
    version: bool,

    /// Print help
    #[arg(short = 'h', long)]
    help: bool,

    /// Command executable to run (default: claude)
    #[arg(short, long, default_value = "claude")]
    command: String,

    /// Environment name
    name: Option<String>,

    /// Validate environment files
    #[arg(long)]
    validate: bool,

    /// Arguments to pass to claude command
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    if cli.help {
        print_help();
        return Ok(());
    }

    if cli.version {
        println!("cce {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    if cli.validate {
        validate_environments(cli.name.as_deref())
    } else if let Some(name) = cli.name {
        // If name starts with '-', treat it as an arg, not an environment name
        if name.starts_with('-') {
            let mut args = vec![name];
            args.extend(cli.args);
            list_environments(&cli.command, &args)
        } else {
            run_environment(&name, &cli.command, &cli.args)
        }
    } else {
        list_environments(&cli.command, &cli.args)
    }
}

/// Validate environment files
fn validate_environments(name: Option<&str>) -> Result<()> {
    let manager = EnvironmentManager::new()?;

    let environments = if let Some(name) = name {
        vec![manager.load_environment(name)?]
    } else {
        manager.list_environments()?
    };

    if environments.is_empty() {
        println!("No environments found to validate.");
        return Ok(());
    }

    let mut all_valid = true;

    for env in &environments {
        print!("Validating {}... ", env.name);
        match env.validate() {
            Ok(result) => {
                if result.is_valid {
                    println!("✓ OK");
                    if !result.missing_optional.is_empty() {
                        println!(
                            "  Note: Missing optional vars: {}",
                            result.missing_optional.join(", ")
                        );
                    }
                } else {
                    println!("✗ INVALID");
                    println!(
                        "  Missing required: {}",
                        result.missing_required.join(", ")
                    );
                    all_valid = false;
                }
            }
            Err(e) => {
                println!("✗ ERROR: {}", e);
                all_valid = false;
            }
        }
    }

    if all_valid {
        println!("\nAll environments are valid.");
    } else {
        println!("\nSome environments have issues.");
        std::process::exit(1);
    }

    Ok(())
}

/// Check if fzf is available on the system
fn is_fzf_available() -> bool {
    Command::new("fzf")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

/// Interactive environment selection using fzf
fn select_environment_fzf(
    environments: &[Environment],
) -> Result<Option<String>> {
    let mut command = Command::new("fzf")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .map_err(|e| {
            CceError::ExecutionFailed(format!("Failed to spawn fzf: {}", e))
        })?;

    // Write environment names to fzf's stdin
    if let Some(stdin) = command.stdin.as_mut() {
        for env in environments {
            writeln!(stdin, "{}", env.name).map_err(|e| {
                CceError::ExecutionFailed(format!(
                    "Failed to write to fzf: {}",
                    e
                ))
            })?;
        }
    }

    // Wait for fzf to complete and get the output
    let output = command.wait_with_output().map_err(|e| {
        CceError::ExecutionFailed(format!("Failed to read fzf output: {}", e))
    })?;

    // If no output, user cancelled (ESC/q or empty selection)
    if output.stdout.is_empty() {
        return Ok(None);
    }

    // Parse the selected environment name (trim newline)
    let selected = String::from_utf8(output.stdout).map_err(|e| {
        CceError::ExecutionFailed(format!("Invalid UTF-8 from fzf: {}", e))
    })?;
    let selected = selected.trim().to_string();

    if selected.is_empty() {
        Ok(None)
    } else {
        Ok(Some(selected))
    }
}

/// Print usage information
fn print_usage() {
    println!("Usage: cce [OPTIONS] [NAME] [-- ARGS...]");
}

/// Print help information
fn print_help() {
    println!("Claude Code Environment Manager\n");
    print_usage();
    println!("\nArguments:");
    println!("  [NAME]      Environment name");
    println!("  [ARGS...]   Arguments to pass to command\n");
    println!("Options:");
    println!(
        "  -c, --command <CMD>  Command executable to run [default: claude]"
    );
    println!("      --validate       Validate environment files");
    println!("      --version        Print version");
    println!("  -h, --help           Print help");
}

/// Print list of environments
fn print_environments(environments: &[Environment]) {
    print_usage();
    for env in environments {
        println!("  {}", env.name);
    }
}

fn list_environments(command: &str, args: &[String]) -> Result<()> {
    let manager = EnvironmentManager::new()?;

    let environments = manager.list_environments()?;

    if environments.is_empty() {
        print_usage();
        println!("No environments found.");
        println!(
            "Create environment files in: {}",
            manager.config_dir().display()
        );

        // Check if directory exists
        if !manager.config_dir().exists() {
            println!("Directory does not exist yet.");
        }

        return Ok(());
    }

    // Try to use fzf for interactive selection
    if is_fzf_available() {
        if let Some(selected_env) = select_environment_fzf(&environments)? {
            // User selected an environment, run it
            run_environment(&selected_env, command, args)
        } else {
            // User cancelled, show list
            print_environments(&environments);
            Ok(())
        }
    } else {
        // fzf not available, show list
        print_environments(&environments);
        Ok(())
    }
}

fn run_environment(name: &str, command: &str, args: &[String]) -> Result<()> {
    let manager = EnvironmentManager::new()?;
    let env_file = manager.get_environment_file(name)?;
    let exit_code = CommandExecutor::execute(&env_file, command, args)?;

    std::process::exit(exit_code);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parse_no_args() {
        let cli = Cli::try_parse_from(["cce"]).unwrap();
        assert!(!cli.help);
        assert!(!cli.version);
        assert!(!cli.validate);
        assert_eq!(cli.command, "claude");
        assert!(cli.name.is_none());
        assert!(cli.args.is_empty());
    }

    #[test]
    fn test_cli_parse_help() {
        let cli = Cli::try_parse_from(["cce", "--help"]).unwrap();
        assert!(cli.help);
    }

    #[test]
    fn test_cli_parse_version() {
        let cli = Cli::try_parse_from(["cce", "--version"]).unwrap();
        assert!(cli.version);
    }

    #[test]
    fn test_cli_parse_validate() {
        let cli = Cli::try_parse_from(["cce", "--validate"]).unwrap();
        assert!(cli.validate);
        assert!(cli.name.is_none());
    }

    #[test]
    fn test_cli_parse_validate_with_name() {
        let cli =
            Cli::try_parse_from(["cce", "--validate", "myenv"]).unwrap();
        assert!(cli.validate);
        assert_eq!(cli.name.as_deref(), Some("myenv"));
    }

    #[test]
    fn test_cli_parse_env_name() {
        let cli = Cli::try_parse_from(["cce", "myenv"]).unwrap();
        assert_eq!(cli.name.as_deref(), Some("myenv"));
        assert_eq!(cli.command, "claude");
    }

    #[test]
    fn test_cli_parse_custom_command() {
        let cli =
            Cli::try_parse_from(["cce", "-c", "my-claude", "myenv"]).unwrap();
        assert_eq!(cli.command, "my-claude");
        assert_eq!(cli.name.as_deref(), Some("myenv"));
    }

    #[test]
    fn test_cli_parse_trailing_args() {
        let cli =
            Cli::try_parse_from(["cce", "myenv", "--", "--flag", "value"])
                .unwrap();
        assert_eq!(cli.name.as_deref(), Some("myenv"));
        assert_eq!(cli.args, vec!["--flag", "value"]);
    }

    #[test]
    fn test_validate_environments_empty_dir() {
        let temp_dir = tempfile::tempdir().unwrap();
        std::env::set_var(
            "XDG_CONFIG_HOME",
            temp_dir.path().to_str().unwrap(),
        );
        let result = validate_environments(None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_environments_empty_dir() {
        let temp_dir = tempfile::tempdir().unwrap();
        std::env::set_var(
            "XDG_CONFIG_HOME",
            temp_dir.path().to_str().unwrap(),
        );
        let result = list_environments("claude", &[]);
        assert!(result.is_ok());
    }
}
