<!-- OPENSPEC:START -->
# OpenSpec Instructions

These instructions are for AI assistants working in this project.

Always open `@/openspec/AGENTS.md` when the request:
- Mentions planning or proposals (words like proposal, spec, change, plan)
- Introduces new capabilities, breaking changes, architecture shifts, or big performance/security work
- Sounds ambiguous and you need the authoritative spec before coding

Use `@/openspec/AGENTS.md` to learn:
- How to create and apply change proposals
- Spec format and conventions
- Project structure and guidelines

Keep this managed block so 'openspec update' can refresh the instructions.

<!-- OPENSPEC:END -->

# Agent Guidelines for CCE Project

## Build/Lint/Test Commands

**Core Commands:**
- `cargo build` - Build debug version
- `cargo build --release` - Build optimized release version
- `cargo test` - Run all tests
- `cargo test <test_name>` - Run specific test by name
- `cargo clippy` - Run linting checks
- `cargo fmt` - Format code
- `cargo fmt --check` - Check formatting without modifying

**Development:**
- `cargo run -- <args>` - Run with arguments (e.g., `cargo run -- glm --help`)
- `cargo run -- --validate` - Validate all environments
- `cargo run -- --help` - Show help

**Release:**
- `cargo build --release` - Optimized build with LTO
- Binary output: `target/release/cce`

## Project Structure

```
src/
├── main.rs       # CLI entry point, argument parsing with clap
├── lib.rs        # Library exports (config, error, executor, manager)
├── config.rs     # Environment struct, validation logic
├── manager.rs    # EnvironmentManager for file discovery and loading
├── executor.rs   # CommandExecutor for shell-based execution
└── error.rs      # CceError enum with thiserror derives
```

## Code Style Guidelines

**Imports & Dependencies:**
- Group imports: standard library → external crates → local modules
- Use `use crate::` for local module imports
- Key dependencies:
  - `clap` (4.x) - CLI argument parsing with derive macros
  - `thiserror` (2.x) - Error type derivation
  - `tempfile` (dev) - File system testing

**Naming Conventions:**
- Functions/variables: `snake_case`
- Types/Structs/Enums: `PascalCase` 
- Constants: `SCREAMING_SNAKE_CASE`
- Files: `snake_case.rs`

**Error Handling:**
- Use `thiserror::Error` derive for error types
- Define custom error enum `CceError` with `#[error("message")]` attributes
- Return `Result<T, CceError>` (aliased as `Result<T>`) from fallible functions
- Use `?` operator for error propagation
- Map errors with `.map_err()` when context needed

**Code Organization:**
- Keep modules small and focused (single responsibility)
- Use `#[cfg(test)]` modules for unit tests
- Public items need `///` doc comments
- Follow XDG Base Directory specification for config paths

**Testing:**
- Write unit tests in `#[cfg(test)]` modules within each file
- Use `tempfile::tempdir()` for file system testing
- Run with `cargo test` or `cargo test <test_name>`
- Tests for: validation, env parsing, file discovery, error handling

**Platform Considerations:**
- Unix: Use `CommandExt::exec()` for process replacement
- Windows: Use subprocess with manual env parsing (compile-time conditional)
- Use `#[cfg(unix)]` and `#[cfg(not(unix))]` for platform-specific code

**Security:**
- Never log or expose API tokens
- Validate all input paths
- Escape file paths properly for shell execution
- Use proper error handling for file operations

## CLI Structure (clap)

```rust
#[derive(Parser, Debug)]
#[command(name = "cce")]
#[command(about = "Claude Code Environment Manager")]
struct Cli {
    #[arg(long)]
    version: bool,

    #[arg(short = 'h', long)]
    help: bool,

    #[arg(short, long, default_value = "claude")]
    command: String,

    name: Option<String>,

    #[arg(long)]
    validate: bool,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}
```

## Key Patterns

**Shell Execution Model:**
```rust
// Build shell command for sourcing env file and executing command
let shell_cmd = format!(
    ". '{}' && exec {} {}",
    env_file.to_string_lossy().replace('\'', "'\\''"),
    command,
    args.join(" ")
);
```

**XDG Config Directory:**
```rust
// Check XDG_CONFIG_HOME first, then fall back to HOME/.config
let config_dir = if let Ok(xdg_config) = env::var("XDG_CONFIG_HOME") {
    if !xdg_config.trim().is_empty() {
        PathBuf::from(xdg_config).join("cce")
    } else {
        get_home_config_dir()?
    }
} else {
    get_home_config_dir()?
};
```

**Environment Validation:**
```rust
const REQUIRED_VARS: &[&str] = &["ANTHROPIC_AUTH_TOKEN"];
const OPTIONAL_VARS: &[&str] = &["ANTHROPIC_BASE_URL"];
```
