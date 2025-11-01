# cli-interface Specification

## Purpose
TBD - created by archiving change reimplement-in-rust. Update Purpose after archive.
## Requirements
### Requirement: CLI-001 List Environments Subcommand
**Requirement:** When executed without an environment name argument, the system SHALL attempt to use fzf for interactive environment selection. If fzf is not available or the user cancels the selection, it SHALL list all available environments.

**Rationale:** Enhance user experience by allowing interactive selection while maintaining backward compatibility.

**Implementation Notes:**
- Detect fzf availability using std::process::Command
- Display fzf interface for environment selection
- Accept keyboard navigation (arrows, j/k, Ctrl+N/Ctrl+P)
- Accept Enter to confirm selection
- Accept ESC or q to cancel and fall back to list view
- Display environments in alphabetical order in fzf
- If fzf unavailable or selection cancelled, show traditional list view
- Include helpful usage information

#### Scenario: Interactive selection with fzf available
```
Given fzf is installed and environments "glm" and "kimi" exist
When user runs "cce" without arguments
Then display fzf interface for environment selection
When user navigates to "glm" and presses Enter
Then load glm environment and execute claude
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

### Requirement: CLI-006 Subcommand Structure (New Style)
**Requirement:** The CLI SHALL support subcommand pattern (list, run) for better structure and extensibility.

**Rationale:** Provides better organization and allows for future enhancements while maintaining backward compatibility with single-argument usage.

**Implementation Notes:**
- Accept both old style (positional) and new style (subcommands)
- Default to "run" subcommand when environment name provided
- "list" subcommand shows available environments

#### Scenario: Backward compatible usage
```
Given environment "glm" exists
When user runs: cce glm --help
Then interpret as: cce run glm --help
```

#### Scenario: New subcommand style
```
When user runs: cce list
Then show available environments
```

