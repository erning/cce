# Project Context

## Purpose
CCE (Claude Code Environment) Manager is a CLI tool that allows you to manage multiple Claude Code environments with different API configurations. It enables easy switching between different API providers (GLM, Kimi, Minimax) and authentication tokens without having to manually set environment variables each time.

## Tech Stack
- **Language**: Rust (Edition 2021)
- **CLI Framework**: clap 4.x with derive macros
- **Error Handling**: thiserror for custom error types
- **Testing**: tempfile for file system testing
- **Configuration**: Environment files (.env format) in `~/.config/cce/`
- **Build**: Cargo with release profile optimization (LTO, single codegen unit)
- **Requirements**: Claude Code CLI tool must be installed and accessible as `claude` (or custom command via `-c` flag)

## Project Structure
```
src/
├── main.rs       # CLI entry point, argument parsing, orchestration
├── lib.rs        # Library exports
├── config.rs     # Environment struct, validation logic
├── manager.rs    # EnvironmentManager for file discovery and loading
├── executor.rs   # CommandExecutor for shell-based execution
└── error.rs      # CceError enum with thiserror derives
```

## Project Conventions

### Code Style
- **Imports**: Group by standard library -> external crates -> local modules
- **Naming**: snake_case for functions/variables, PascalCase for types
- **Error Handling**: Use `Result<T, CceError>` with `?` propagation
- **Documentation**: `///` doc comments for public items
- **Testing**: `#[cfg(test)]` modules with tempfile for file system tests

### Architecture Patterns
- **Single Responsibility**: Each module handles one concern (config, manager, executor, error)
- **XDG Base Directory**: Configuration in `$XDG_CONFIG_HOME/cce/` or `$HOME/.config/cce/`
- **Shell Source Execution**: Environment files executed via shell `source` command for full shell syntax support
- **Process Replacement**: Unix exec() for minimal overhead, Windows fallback for compatibility
- **Interactive Selection**: fzf integration for environment selection when available

### Command Execution Model
The executor uses shell source to load environment files:
```rust
// Shell command pattern
". '{env_file}' && exec {command} {args}"
```
This approach:
- Supports full shell syntax (variables, command substitution, conditionals)
- Preserves environment file compatibility with bash scripts
- Uses exec for process replacement on Unix (minimal overhead)
- Falls back to subprocess on Windows with env file parsing

### Testing Strategy
- **Unit Tests**: In-module `#[cfg(test)]` blocks
- **File System Tests**: Use `tempfile` crate for isolated testing
- **Validation Tests**: Test env file parsing and validation
- **Run Tests**: `cargo test` or `cargo test <test_name>`

### Git Workflow
- **Branching**: Simple linear history, main branch for production
- **Commit Messages**: Imperative mood, focused and atomic
- **OpenSpec**: Use openspec workflow for significant changes

## Domain Context
- **API Provider Management**: Supports multiple Claude API-compatible providers
- **Environment Isolation**: Each environment has separate base URL and auth token
- **Developer Tool**: Designed for developers who work with multiple Claude API providers
- **Use Cases**:
  - Switch between GLM, Kimi, and Minimax providers
  - Manage multiple API keys for different accounts/projects
  - Testing different API endpoints
  - Maintaining dev/prod environment separation

## Important Constraints
- Requires Claude Code CLI tool to be installed and in PATH (or custom command)
- Environment files must use `.env` extension
- Environment files must contain `ANTHROPIC_AUTH_TOKEN` (required)
- `ANTHROPIC_BASE_URL` is optional
- Configuration directory follows XDG spec: `$XDG_CONFIG_HOME/cce/` or `~/.config/cce/`
- Unix-optimized with Windows fallback support

## External Dependencies
- **Claude Code CLI**: Primary dependency - the tool wraps this command
- **fzf** (optional): For interactive environment selection
- **API Providers**:
  - GLM (BigModel): https://open.bigmodel.cn/api/anthropic
  - Kimi (Moonshot AI)
  - Minimax
