use clap::{Parser, Subcommand};

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
#[command(infer_subcommands = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Environment name (used with 'run' command)
    name: Option<String>,

    /// Arguments to pass to claude command
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List all available environments
    List,
    /// Run claude with a specific environment
    Run {
        /// Environment name
        name: String,
        /// Arguments to pass to claude command
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::List) => list_environments(),
        Some(Commands::Run { name, args }) => run_environment(&name, &args),
        None => {
            // When no subcommand is given, the first positional arg is treated as name
            if let Some(name) = cli.name {
                run_environment(&name, &cli.args)
            } else {
                list_environments()
            }
        }
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

    println!("Using environment: {}", environment.name);

    let exit_code = executor::CommandExecutor::execute(&environment, args)
        .map_err(|e| {
            eprintln!("Error executing command: {}", e);
            e
        })?;

    std::process::exit(exit_code);
}