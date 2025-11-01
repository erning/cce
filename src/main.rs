use clap::Parser;

mod config;
mod error;
mod manager;
mod executor;

use crate::error::Result;
use crate::manager::EnvironmentManager;

#[derive(Parser, Debug)]
#[command(name = "cce")]
#[command(about = "Claude Code Environment Manager")]
#[command(version = "2.0.0")]
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

fn list_environments() -> Result<()> {
    println!("Usage: cce <name> [claude-code arguments...]");

    let manager = EnvironmentManager::new()
        .map_err(|e| {
            eprintln!("Error: {}", e);
            e
        })?;

    let environments = manager.list_environments()?;

    if environments.is_empty() {
        println!("No environments found.");
        println!("Create environment files in: {}", manager.config_dir().display());

        // Check if directory exists
        if !manager.config_dir().exists() {
            println!("Directory does not exist yet.");
        }
    } else {
        for env in environments {
            println!("  {}", env.name);
        }
    }

    Ok(())
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