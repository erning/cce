# command-execution Specification

## Purpose
TBD - created by archiving change reimplement-in-rust. Update Purpose after archive.
## Requirements
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

### Requirement: EXEC-005 Command Not Found Handling
**Requirement:** The system SHALL provide clear error messages when the specified command is not found during execution.

**Rationale:** Provides clear error message when the command is not installed, with appropriate exit code.

**Implementation Notes:**
- Command availability is checked at execution time (not pre-checked)
- On Unix: shell reports command not found via exec error
- On Windows: Command::status() returns NotFound error
- Exit with code 127 (standard command not found)

#### Scenario: Command executes successfully
```
Given command "claude" is in PATH
When system executes the command
Then proceed with execution
```

#### Scenario: Command not found at execution
```
Given command "claude" is not in PATH
When system attempts to execute
Then display error:
  Error: 'claude' command not found in PATH
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

### Requirement: EXEC-007 Platform-Optimized Command Execution
**Requirement:** The system SHALL use platform-optimized execution methods to run the specified command executable, minimizing process overhead while maintaining compatibility across operating systems.

**Rationale:** Process exec replacement on Unix systems eliminates parent process overhead and provides better performance, while maintaining Windows compatibility through subprocess execution. Supports configurable command names.

**Implementation Notes:**
- Accept command name as a parameter (default: "claude")
- Use `std::os::unix::process::CommandExt::exec()` on Unix systems for process replacement
- Use `std::process::Command::status()` on Windows systems as fallback
- Preserve all environment variables and execution context
- Handle exec failures with appropriate exit codes

#### Scenario: Unix process exec with default command
- **WHEN** executing on Unix-based systems (Linux, macOS)
- **AND** no custom command is specified
- **THEN** use process exec to replace the current process with "claude"
- **AND** preserve all environment variables from the loaded configuration

#### Scenario: Unix process exec with custom command
- **WHEN** executing on Unix-based systems (Linux, macOS)
- **AND** custom command "claude-code" is specified
- **THEN** use process exec to replace the current process with "claude-code"
- **AND** preserve all environment variables from the loaded configuration

#### Scenario: Windows subprocess with custom command
- **WHEN** executing on Windows systems
- **AND** custom command "claude-code" is specified
- **THEN** use subprocess execution to run "claude-code"
- **AND** maintain the same functionality as Unix systems

#### Scenario: Command not found handling
- **WHEN** the specified command is not found during execution
- **THEN** exit with code 127 (standard command not found)
- **AND** provide appropriate error messaging including the command name

