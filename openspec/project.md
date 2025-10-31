# Project Context

## Purpose
CCE (Claude Code Environment) Manager is a shell script that allows you to manage multiple Claude Code environments with different API configurations. It enables easy switching between different API providers (GLM, Kimi, Minimax) and authentication tokens without having to manually set environment variables each time.

## Tech Stack
- **Shell**: Bash (bourne again shell)
- **Tools**: Standard Unix utilities (mkdir, basename, source, exec)
- **Configuration**: Environment files (.env format) in `~/.config/cce/`
- **Dependencies**: None - pure bash script with no external dependencies
- **Requirements**: Claude Code CLI tool must be installed and accessible as `claude`

## Project Conventions

### Code Style
- Use bash shebang `#!/bin/bash`
- Include comprehensive header comments with OVERVIEW, USE CASES, CONFIGURATION, USAGE, REQUIREMENTS, and ERROR HANDLING
- Use meaningful variable names: `ENV_NAME`, `ENV_DIR`, `ENV_FILE`
- Use `$(basename $0)` for script name references
- Use `source` for loading environment files
- Use `exec claude "$@"` to pass all arguments through to claude

### Architecture Patterns
- **Single Responsibility**: One script, one purpose - environment management
- **Configuration-Driven**: Environment files in `~/.config/cce/<name>.env`
- **Pass-Through Pattern**: Pass all non-environment arguments directly to claude command
- **Fail-Fast**: Check for required conditions early and exit with error codes
- **List Before Execute**: When no environment specified, list available options

### Testing Strategy
- **Manual Testing**: Test with different environment files
- **Error Handling**: Verify error messages for:
  - Missing environment directory
  - Missing environment file
  - Invalid environment file format
- **Functional Testing**: Test with actual claude commands using different providers
- **Cross-Platform**: Test on different Unix-like systems (Linux, macOS)

### Git Workflow
- **Branching**: Simple linear history, main branch for production
- **Commit Messages**:
  - Use imperative mood ("Fix cce download URL" not "Fixed cce download URL")
  - Include change scope when relevant
  - Keep commits focused and atomic
- **Recent Commits**:
  - `031bbab` - Fix cce download URL and update license to MIT
  - `386c2ae` - Initial implementation of CCE Manager

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
- Requires Claude Code CLI tool to be installed and in PATH
- Environment files must use `.env` extension
- Environment files must contain valid `export` statements for `ANTHROPIC_BASE_URL` and `ANTHROPIC_AUTH_TOKEN`
- Pure bash implementation - no dependencies on Python, Node.js, or other runtimes
- Must work on standard Unix-like systems (Linux, macOS)
- Configuration directory fixed at `~/.config/cce/`

## External Dependencies
- **Claude Code CLI**: Primary dependency - the script wraps this tool
- **Unix Utilities**: Standard bash and Unix tools (mkdir, basename, source, exec)
- **API Providers**:
  - GLM (BigModel): https://open.bigmodel.cn/api/anthropic
  - Kimi (Moonshot AI)
  - Minimax
- **No Build Tools**: Script is executable as-is, no compilation or build process required
