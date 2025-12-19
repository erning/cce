# cli-interface Specification

## Purpose
TBD - created by archiving change reimplement-in-rust. Update Purpose after archive.
## Requirements
### Requirement: CLI-001 List Environments Subcommand
**Requirement:** When executed without an environment name argument, the system SHALL attempt to use fzf for interactive environment selection. If fzf is not available or the user cancels the selection, it SHALL list all available environments. Any arguments provided after `--` SHALL be passed to the command after environment selection.

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
- Pass any arguments after `--` to the executed command

#### Scenario: Interactive selection with fzf available
```
Given fzf is installed and environments "glm" and "kimi" exist
When user runs "cce" without arguments
Then display fzf interface for environment selection
When user navigates to "glm" and presses Enter
Then load glm environment and execute claude
```

#### Scenario: Interactive selection with arguments
```
Given fzf is installed and environments "glm" and "kimi" exist
When user runs "cce -- --help"
Then display fzf interface for environment selection
When user navigates to "glm" and presses Enter
Then load glm environment and execute "claude --help"
```

#### Scenario: Interactive selection with fzf unavailable
```
Given fzf is not installed and environments "glm" and "kimi" exist
When user runs "cce" without arguments
Then display:
  Usage: cce <name> [claude-code arguments...]
    glm
    kimi
And exit with code 1
```

#### Scenario: User cancels interactive selection
```
Given fzf is installed and environments "glm" and "kimi" exist
When user runs "cce" without arguments
Then display fzf interface for environment selection
When user presses ESC or q
Then display:
  Usage: cce <name> [claude-code arguments...]
    glm
    kimi
And exit with code 1
```

#### Scenario: No environments available with fzf
```
Given fzf is installed but no environments exist in ~/.config/cce/
When user runs "cce" without arguments
Then display:
  Usage: cce <name> [claude-code arguments...]
    (no environment found)
And exit with code 1
```

### Requirement: CLI-002 Run Environment Subcommand
**Requirement:** The system SHALL execute the claude command with the specified environment's configuration when an environment name is provided.

**Rationale:** Core functionality to use a specific environment for claude commands.

**Implementation Notes:**
- Accept environment name as first positional argument
- Accept zero or more additional arguments to pass through to claude
- Environment name must be valid (contain only safe characters)

#### Scenario: Execute with environment
```
Given environment "glm" exists with valid configuration
When user runs "cce glm --help"
Then load glm environment and execute: claude --help
With environment variables set from glm.env
```

#### Scenario: Execute with environment and multiple arguments
```
Given environment "minimax" exists
When user runs "cce minimax "explain rust ownership""
Then load minimax environment and execute: claude "explain rust ownership"
```

#### Scenario: Execute with no additional arguments
```
Given environment "kimi" exists
When user runs "cce kimi"
Then load kimi environment and execute: claude (with no arguments)
```

### Requirement: CLI-003 Help and Usage Information
**Requirement:** The system SHALL provide clear usage information and help text.

**Rationale:** Improves user experience and reduces confusion about how to use the tool.

**Implementation Notes:**
- Show usage format in error messages
- Provide examples of common usage
- Display available environments when no name provided

#### Scenario: Display usage on missing environment
```
When user runs "cce"
Then show usage message:
  Usage: cce <name> [claude-code arguments...]

  Available environments:
    glm
    kimi
    minimax
```

### Requirement: CLI-004 Error Handling for Invalid Inputs
**Requirement:** The system SHALL provide clear error messages for invalid input and exit with appropriate error codes.

**Rationale:** Helps users understand and fix problems quickly.

**Implementation Notes:**
- Exit code 1: General error (missing environment, invalid input)
- Exit code 2: Usage error (invalid command-line usage)
- Exit code 127: Command not found (claude CLI not in PATH)
- Clear, actionable error messages

#### Scenario: Environment file not found
```
Given no environment named "invalid" exists
When user runs "cce invalid"
Then display error:
  Error: File ~/.config/cce/invalid.env not found
And exit with code 1
```

#### Scenario: Config directory does not exist
```
Given ~/.config/cce/ does not exist
When user runs "cce"
Then display:
  Usage: cce <name> [claude-code arguments...]
    (directory ~/.config/cce/ does not exist)
And exit with code 1
```

### Requirement: CLI-005 Pass-Through Arguments
**Requirement:** The system SHALL pass all arguments after the environment name directly to the claude command.

**Rationale:** Allows full flexibility in using claude with different commands and options.

**Implementation Notes:**
- Preserve argument order
- Support quoted arguments
- Support arguments with special characters
- Support empty arguments (no arguments after environment name)

#### Scenario: Pass complex arguments
```
When user runs: cce glm "write a function" --output file.txt
Then execute: claude "write a function" --output file.txt
With glm environment variables set
```

#### Scenario: Pass arguments with quotes
```
When user runs: cce minimax 'echo "hello world"'
Then execute: claude 'echo "hello world"'
```

### Requirement: CLI-006 Positional Argument Structure
**Requirement:** The CLI SHALL use positional arguments for environment selection, with optional flags for configuration.

**Rationale:** Simple and intuitive CLI design. Environment name as first positional argument, remaining arguments passed to the command.

**Implementation Notes:**
- First positional argument is the environment name (optional)
- If no environment name, show fzf selection or list
- If argument starts with `-`, treat as args for passthrough (not environment name)
- Remaining arguments after environment name passed to command

#### Scenario: Environment as positional argument
```
Given environment "glm" exists
When user runs: cce glm --help
Then load glm environment and execute: claude --help
```

#### Scenario: No arguments shows selection/list
```
When user runs: cce
Then show fzf selection (if available) or list environments
```

### Requirement: CLI-007 Configurable Command Executable
**Requirement:** The system SHALL accept an optional `--command` (or `-c`) parameter to specify the executable name to run, with a default value of "claude".

**Rationale:** Allows users to use alternative executables (e.g., `claude-code`, custom wrapper scripts) while maintaining backward compatibility.

**Implementation Notes:**
- Add `--command` / `-c` as a global option in the CLI struct
- Default value is "claude" to maintain backward compatibility
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

