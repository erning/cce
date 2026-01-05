# cli-interface Specification

## Purpose
Defines the command-line interface for the CCE (Claude Code Environment) Manager, including argument parsing, environment selection, help/version display, and validation commands.

## Requirements

### Requirement: CLI-001 List Environments Command
**Requirement:** When executed without an environment name argument, the system SHALL attempt to use fzf for interactive environment selection. If fzf is not available or the user cancels the selection, it SHALL list all available environments. Any arguments provided SHALL be passed to the command after environment selection.

**Rationale:** Enhance user experience by allowing interactive selection while maintaining backward compatibility. Allow users to specify command arguments before selecting an environment.

**Implementation Notes:**
- Detect fzf availability using std::process::Command
- Display fzf interface for environment selection
- Accept keyboard navigation (arrows, j/k, Ctrl+N/Ctrl+P)
- Accept Enter to confirm selection
- Accept ESC or q to cancel and fall back to list view
- Display environments in alphabetical order in fzf
- If fzf unavailable or selection cancelled, show traditional list view
- Include helpful usage information
- Pass any trailing arguments to the executed command

#### Scenario: Interactive selection with fzf available
- **WHEN** fzf is installed and environments "glm" and "kimi" exist
- **AND** user runs `cce` without arguments
- **THEN** display fzf interface for environment selection
- **WHEN** user navigates to "glm" and presses Enter
- **THEN** load glm environment and execute claude

#### Scenario: Interactive selection with arguments
- **WHEN** fzf is installed and environments "glm" and "kimi" exist
- **AND** user runs `cce -- --help`
- **THEN** display fzf interface for environment selection
- **WHEN** user navigates to "glm" and presses Enter
- **THEN** load glm environment and execute `claude --help`

#### Scenario: Interactive selection with fzf unavailable
- **WHEN** fzf is not installed and environments "glm" and "kimi" exist
- **AND** user runs `cce` without arguments
- **THEN** display usage and environment list

#### Scenario: User cancels interactive selection
- **WHEN** fzf is installed and environments exist
- **AND** user runs `cce` without arguments
- **AND** user presses ESC or q in fzf
- **THEN** display usage and environment list

#### Scenario: No environments available
- **WHEN** no environments exist in config directory
- **AND** user runs `cce` without arguments
- **THEN** display usage message with "No environments found"
- **AND** show config directory path

### Requirement: CLI-002 Run Environment Command
**Requirement:** The system SHALL execute the specified command with the selected environment's configuration when an environment name is provided as the first positional argument.

**Rationale:** Core functionality to use a specific environment for command execution.

**Implementation Notes:**
- Accept environment name as first positional argument
- Accept zero or more additional arguments to pass through to the command
- Environment name must not start with `-` (treated as passthrough arg otherwise)

#### Scenario: Execute with environment
- **WHEN** environment "glm" exists with valid configuration
- **AND** user runs `cce glm --help`
- **THEN** load glm environment and execute `claude --help`

#### Scenario: Execute with environment and multiple arguments
- **WHEN** environment "minimax" exists
- **AND** user runs `cce minimax "explain rust ownership"`
- **THEN** load minimax environment and execute `claude "explain rust ownership"`

#### Scenario: Execute with no additional arguments
- **WHEN** environment "kimi" exists
- **AND** user runs `cce kimi`
- **THEN** load kimi environment and execute `claude` with no arguments

#### Scenario: Argument starting with dash treated as passthrough
- **WHEN** user runs `cce --help`
- **AND** fzf is available
- **THEN** show fzf selection (since `--help` starts with `-`)
- **WHEN** user selects an environment
- **THEN** execute command with `--help` argument

### Requirement: CLI-003 Help and Version Information
**Requirement:** The system SHALL provide `--help` (or `-h`) and `--version` flags for displaying help text and version information respectively.

**Rationale:** Standard CLI conventions for user assistance.

**Implementation Notes:**
- `--help` / `-h` displays usage, arguments, and options
- `--version` displays package version from Cargo.toml
- These flags are processed before environment logic

#### Scenario: Display help
- **WHEN** user runs `cce --help` or `cce -h`
- **THEN** display help text with usage, arguments, and options

#### Scenario: Display version
- **WHEN** user runs `cce --version`
- **THEN** display `cce {version}` from package metadata

### Requirement: CLI-004 Environment Validation Command
**Requirement:** The system SHALL provide a `--validate` flag that validates environment files without executing any command.

**Rationale:** Allows users to check their environment configurations for errors before use.

**Implementation Notes:**
- `--validate` without environment name validates all environments
- `--validate` with environment name validates only that environment
- Display validation status for each environment (OK, INVALID, ERROR)
- Report missing required and optional variables
- Exit with code 1 if any validation fails

#### Scenario: Validate all environments
- **WHEN** user runs `cce --validate`
- **THEN** validate each environment file in config directory
- **AND** display status for each (e.g., "glm... OK", "kimi... INVALID")

#### Scenario: Validate specific environment
- **WHEN** user runs `cce --validate glm`
- **THEN** validate only the "glm" environment
- **AND** display validation result

#### Scenario: Validation shows missing optional
- **WHEN** environment has ANTHROPIC_AUTH_TOKEN but no ANTHROPIC_BASE_URL
- **AND** user runs `cce --validate`
- **THEN** show "OK" with note about missing optional variable

#### Scenario: Validation fails for missing required
- **WHEN** environment lacks ANTHROPIC_AUTH_TOKEN
- **AND** user runs `cce --validate`
- **THEN** show "INVALID" with missing required variable listed
- **AND** exit with code 1

### Requirement: CLI-005 Configurable Command Executable
**Requirement:** The system SHALL accept an optional `--command` (or `-c`) parameter to specify the executable name to run, with a default value of "claude".

**Rationale:** Allows users to use alternative executables (e.g., `claude-code`, custom wrapper scripts) while maintaining backward compatibility.

**Implementation Notes:**
- Add `--command` / `-c` as a global option
- Default value is "claude" for backward compatibility
- The parameter value is passed to the command executor
- Works with both direct environment execution and fzf selection

#### Scenario: Default executable behavior
- **WHEN** user runs `cce glm` without `--command` parameter
- **THEN** execute the default `claude` command with glm environment

#### Scenario: Custom executable specified
- **WHEN** user runs `cce --command claude-code glm`
- **THEN** execute `claude-code` command with glm environment

#### Scenario: Short flag for custom executable
- **WHEN** user runs `cce -c my-claude glm --help`
- **THEN** execute `my-claude --help` with glm environment

#### Scenario: Custom executable with fzf selection
- **WHEN** user runs `cce --command claude-code` without environment name
- **AND** fzf is available
- **THEN** show fzf selection interface
- **WHEN** user selects an environment
- **THEN** execute `claude-code` with the selected environment

### Requirement: CLI-006 Pass-Through Arguments
**Requirement:** The system SHALL pass all arguments after the environment name directly to the command unchanged.

**Rationale:** Allows full flexibility in using the underlying command with various options.

**Implementation Notes:**
- Preserve argument order
- Support quoted arguments
- Support arguments with special characters
- Support arguments starting with `-` after environment name

#### Scenario: Pass complex arguments
- **WHEN** user runs `cce glm "write a function" --output file.txt`
- **THEN** execute `claude "write a function" --output file.txt`

#### Scenario: Pass arguments with quotes
- **WHEN** user runs `cce minimax 'echo "hello world"'`
- **THEN** execute `claude 'echo "hello world"'`

### Requirement: CLI-007 Error Handling
**Requirement:** The system SHALL provide clear error messages for invalid input and exit with appropriate error codes.

**Rationale:** Helps users understand and fix problems quickly.

**Implementation Notes:**
- Exit code 0: Success
- Exit code 1: General error (missing environment, validation failure)
- Exit code 127: Command not found
- Clear, actionable error messages

#### Scenario: Environment file not found
- **WHEN** no environment named "invalid" exists
- **AND** user runs `cce invalid`
- **THEN** display error with file path
- **AND** exit with code 1

#### Scenario: Config directory does not exist
- **WHEN** config directory does not exist
- **AND** user runs `cce`
- **THEN** display usage with note about missing directory
