# Command Execution Specification

## Overview
The Command Execution capability handles the actual execution of claude CLI commands with the appropriate environment variables set from the selected environment configuration.

## ADDED Requirements

### Requirement: EXEC-001 Environment Variable Setup
**Requirement:** The system SHALL set `ANTHROPIC_BASE_URL` and `ANTHROPIC_AUTH_TOKEN` environment variables before executing the claude command.

**Rationale:** Core functionality to apply environment configuration to the claude CLI execution context.

**Implementation Notes:**
- Set variables in the child process environment, not parent
- Clear any existing values for these variables
- Variables persist for the duration of the claude command execution

#### Scenario: Execute with environment variables
```
Given environment "glm" with:
  ANTHROPIC_BASE_URL="https://open.bigmodel.cn/api/anthropic"
  ANTHROPIC_AUTH_TOKEN="glm_secret_123"
When user runs: cce glm --help
Then execute claude with:
  ANTHROPIC_BASE_URL="https://open.bigmodel.cn/api/anthropic"
  ANTHROPIC_AUTH_TOKEN="glm_secret_123"
```

### Requirement: EXEC-002 Command Passthrough
**Requirement:** The system SHALL pass all arguments (except the environment name) directly to the claude command unchanged.

**Rationale:** Maintains full compatibility with claude CLI and its various options and commands.

**Implementation Notes:**
- Preserve argument order
- Preserve argument quoting
- Support any number of arguments (including zero)
- Support special characters in arguments

#### Scenario: Simple argument passthrough
```
When user runs: cce glm "explain rust ownership"
Then execute: claude "explain rust ownership"
```

#### Scenario: Multiple arguments with flags
```
When user runs: cce minimax --model mini-max-latest --output file.txt
Then execute: claude --model mini-max-latest --output file.txt
```

#### Scenario: No additional arguments
```
When user runs: cce kimi
Then execute: claude (with no arguments)
```

#### Scenario: Arguments with special characters
```
When user runs: cce glm 'echo "hello $WORLD"'
Then execute: claude 'echo "hello $WORLD"'
With single quotes preserved
```

### Requirement: EXEC-003 Exit Code Propagation
**Requirement:** The system SHALL propagate the exit code from the claude command to the caller.

**Rationale:** Allows scripts and users to detect failures in claude commands.

**Implementation Notes:**
- Pass through zero exit codes (success)
- Pass through non-zero exit codes (errors)
- Do not modify the exit code

#### Scenario: Successful execution
```
Given claude command exits with code 0
When system executes the command
Then system exits with code 0
```

#### Scenario: Failed execution
```
Given claude command exits with code 1
When system executes the command
Then system exits with code 1
```

#### Scenario: Permission error
```
Given claude command exits with code 126
When system executes the command
Then system exits with code 126
```

### Requirement: EXEC-004 Standard Streams Handling
**Requirement:** The system SHALL connect stdout and stderr from the claude command directly to the user's terminal.

**Rationale:** Provides transparent interaction with claude CLI, showing all output in real-time.

**Implementation Notes:**
- Stream stdout directly to parent's stdout
- Stream stderr directly to parent's stderr
- Do not buffer or modify output
- Maintain line buffering for interactive use

#### Scenario: Command produces stdout
```
When claude outputs "Hello, Claude!" to stdout
Then user sees "Hello, Claude!" immediately
```

#### Scenario: Command produces stderr
```
When claude writes "Warning: Using deprecated feature" to stderr
Then user sees "Warning: Using deprecated feature" immediately
```

#### Scenario: Mixed output
```
When claude writes to both stdout and stderr
Then both streams are displayed in real-time without mixing
```

### Requirement: EXEC-005 Claude CLI Presence Check
**Requirement:** The system SHALL verify that the claude CLI is available in PATH before attempting execution.

**Rationale:** Provides clear error message when claude is not installed, rather than confusing errors.

**Implementation Notes:**
- Check for claude executable in PATH
- Use `which claude` or equivalent
- Provide helpful error message if not found

#### Scenario: Claude CLI found in PATH
```
Given "which claude" returns /usr/local/bin/claude
When system checks for claude
Then proceed to execution
```

#### Scenario: Claude CLI not found
```
Given "which claude" returns no results
When system attempts to execute
Then display error:
  Error: 'claude' command not found in PATH
  Please ensure Claude Code CLI is installed
And exit with code 127
```

### Requirement: EXEC-006 Multiple Environment Support
**Requirement:** The system SHALL correctly load and apply different environment configurations for different execution requests.

**Rationale:** Core functionality to switch between different API providers and accounts.

**Implementation Notes:**
- Load fresh configuration for each execution
- Do not persist environment variables between invocations
- Allow switching environments in rapid succession

#### Scenario: Sequential environment usage
```
Given environments "glm" and "kimi" exist
When user runs: cce glm --version
Then execute with glm configuration

When user runs: cce kimi --version
Then execute with kimi configuration
```

#### Scenario: Same environment multiple times
```
Given environment "minimax" exists
When user runs: cce minimax command1
And then runs: cce minimax command2
Then both commands use the same minimax configuration
```

### Requirement: EXEC-007 Native Command Execution
**Requirement:** The system SHALL use Rust's `std::process::Command` for command execution.

**Rationale:** Native execution is more reliable and secure than shell execution.

**Implementation Notes:**
- Use Command::new("claude")
- Pass arguments as vector
- Set environment variables via envs()
- Wait for completion and propagate exit code

#### Scenario: Native execution
```
When system executes claude command
Then use Command::new("claude") with arguments
Instead of shell execution
```

## Removed Requirements
None.

## Error Handling

### Configuration Loading Errors
- If environment file fails to load, display error and exit
- Do not attempt execution with invalid configuration

### Permission Errors
- If claude is not executable, display error
- Exit with code 126 (command cannot execute)

### Runtime Errors
- If claude crashes or panics, propagate the exit code
- Do not mask failures from claude

## Success Criteria
1. All environment variables are correctly set for claude execution
2. Arguments pass through unchanged
3. Exit codes are properly propagated
4. Output streams are connected correctly
5. Clear error messages when claude is not available

## Performance Considerations

### Execution Overhead
- Minimize overhead between user command and claude execution
- Only load configuration once per invocation
- No unnecessary shell spawns

### Memory Usage
- Environment variables only exist in child process
- Parent process memory footprint remains small
- No memory leaks between executions

## References
- Current bash implementation: `/cce` (lines 87-91)
- Rust std::process::Command: https://doc.rust-lang.org/std/process/struct.Command.html
- Environment variable handling: std::env::env
