use clap::Parser;
use std::io::Write;
use std::process::Command;

mod config;
mod error;
mod manager;
mod executor;

use crate::error::Result;
use crate::manager::EnvironmentManager;

#[derive(Parser, Debug)]
#[command(name = "cce")]
#[command(about = "Claude Code Environment Manager")]
#[command(version = "2.0.1")]
#[command(disable_help_flag = true)]
#[command(disable_version_flag = true)]
struct Cli {
    /// Environment name
    name: Option<String>,

    /// Arguments to pass to claude command
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(name) = cli.name {
        run_environment(&name, &cli.args)
    } else {
        list_environments()
    }
}

/// Check if fzf is available on the system
fn is_fzf_available() -> bool {
    Command::new("which")
        .arg("fzf")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

/// Interactive environment selection using fzf
fn select_environment_fzf(environments: &[crate::config::Environment]) -> Result<Option<String>> {
    let mut command = Command::new("fzf")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .map_err(|e| crate::error::CceError::ExecutionFailed(format!("Failed to spawn fzf: {}", e)))?;

    // Write environment names to fzf's stdin
    if let Some(stdin) = command.stdin.as_mut() {
        for env in environments {
            writeln!(stdin, "{}", env.name)
                .map_err(|e| crate::error::CceError::ExecutionFailed(format!("Failed to write to fzf: {}", e)))?;
        }
    }

    // Wait for fzf to complete and get the output
    let output = command.wait_with_output()
        .map_err(|e| crate::error::CceError::ExecutionFailed(format!("Failed to read fzf output: {}", e)))?;

    // If no output, user cancelled (ESC/q or empty selection)
    if output.stdout.is_empty() {
        return Ok(None);
    }

    // Parse the selected environment name (trim newline)
    let selected = String::from_utf8(output.stdout)
        .map_err(|e| crate::error::CceError::ExecutionFailed(format!("Invalid UTF-8 from fzf: {}", e)))?;
    let selected = selected.trim().to_string();

    if selected.is_empty() {
        Ok(None)
    } else {
        Ok(Some(selected))
    }
}

/// Print usage information
fn print_usage() {
    println!("Usage: cce-2.0.1 <name> [claude-code arguments...]");
}

/// Print list of environments
fn print_environments(environments: &[crate::config::Environment]) {
    print_usage();
    for env in environments {
        println!("  {}", env.name);
    }
}

fn list_environments() -> Result<()> {
    let manager = EnvironmentManager::new()
        .map_err(|e| {
            eprintln!("Error: {}", e);
            e
        })?;

    let environments = manager.list_environments()?;

    if environments.is_empty() {
        print_usage();
        println!("No environments found.");
        println!("Create environment files in: {}", manager.config_dir().display());

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
            run_environment(&selected_env, &Vec::new())
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

fn run_environment(name: &str, args: &[String]) -> Result<()> {
    let manager = EnvironmentManager::new()
        .map_err(|e| {
            eprintln!("Error: {}", e);
            e
        })?;

    let environment = manager.load_environment(name)
        .map_err(|e| {
            eprintln!("Error loading environment '{}': {}", name, e);
            e
        })?;

    let exit_code = executor::CommandExecutor::execute(&environment, args)
        .map_err(|e| {
            eprintln!("Error executing command: {}", e);
            e
        })?;

    std::process::exit(exit_code);
}