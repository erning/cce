# CLI Interface Specification

## Overview
The CLI Interface capability defines the command-line interface for the CCE Manager, ensuring users can interact with the tool through a consistent and intuitive command structure.

## ADDED Requirements

### Requirement: CLI-001 List Environments Subcommand
**Requirement:** The system SHALL list all available environments when executed without an environment name argument.

**Rationale:** Maintains backward compatibility with bash version's behavior of listing environments when no argument provided.

**Implementation Notes:**
- Display environments in alphabetical order
- Show one environment name per line
- Include helpful usage information
- Exit with code 1 (error) when no environment specified

#### Scenario: List environments successfully
```
Given environments "glm" and "kimi" exist
When user runs "cce" without arguments
Then display:
  Usage: cce <name> [claude-code arguments...]
    glm
    kimi
And exit with code 1
```

#### Scenario: No environments available
```
Given no environments exist in ~/.config/cce/
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

## Removed Requirements
None.

## Compatibility Matrix

| Bash Version | Rust Version | Compatible |
|--------------|--------------|------------|
| `cce` (no args) | `cce` (no args) | ✓ |
| `cce <name> <args...>` | `cce <name> <args...>` | ✓ |
| `cce <name>` | `cce <name>` | ✓ |
| - | `cce list` | New |
| - | `cce run <name>` | New (alternative) |

## Success Criteria
1. All bash version CLI interactions work identically
2. Clear error messages for all error cases
3. Pass-through arguments work correctly
4. Exit codes match expectations
5. Help and usage information is clear and helpful

## References
- Current bash implementation: `/cce` (lines 61-91)
- CLI parsing: clap crate (Rust)
- Pass-through execution: std::process::Command
