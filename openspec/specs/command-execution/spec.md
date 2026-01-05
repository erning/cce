# command-execution Specification

## Purpose
Defines how CCE executes commands with environment configurations loaded. The system uses shell source to load environment files and exec/subprocess to run the target command.

## Requirements

### Requirement: EXEC-001 Shell Source Execution Model
**Requirement:** The system SHALL execute environment files using shell source command followed by exec to run the target command with those environment variables.

**Rationale:** Shell source provides full compatibility with shell syntax in environment files, including variable expansion, command substitution, and conditionals.

**Implementation Notes:**
- Build shell command: `. '{env_file}' && exec {command} {args}`
- Escape single quotes in file path for shell safety
- Use `/bin/sh -c` to execute the shell command
- Shell handles all environment variable setup

#### Scenario: Execute with environment via shell
- **WHEN** environment file `/path/to/glm.env` is loaded
- **AND** command is "claude" with args "--help"
- **THEN** execute shell command: `. '/path/to/glm.env' && exec claude --help`

#### Scenario: File path with special characters
- **WHEN** environment file path contains single quotes
- **THEN** escape single quotes properly for shell

### Requirement: EXEC-002 Unix Process Replacement
**Requirement:** On Unix systems, the system SHALL use process exec to replace the current process with the target command.

**Rationale:** Process replacement eliminates parent process overhead and provides clean process hierarchy.

**Implementation Notes:**
- Use `std::os::unix::process::CommandExt::exec()` on Unix
- exec() replaces the current process, never returns on success
- Handle exec failure with appropriate exit codes
- Exit code 127 for command not found

#### Scenario: Successful process replacement
- **WHEN** executing on Unix (Linux, macOS)
- **AND** command exists and is executable
- **THEN** replace current process with the command
- **AND** preserve all environment variables

#### Scenario: Command not found
- **WHEN** the command is not in PATH
- **AND** exec fails with NotFound error
- **THEN** exit with code 127

#### Scenario: Other execution errors
- **WHEN** exec fails for other reasons
- **THEN** exit with code 1
- **AND** display error message

### Requirement: EXEC-003 Windows Subprocess Fallback
**Requirement:** On Windows systems, the system SHALL parse environment files and run commands as subprocesses.

**Rationale:** Windows does not support Unix exec(), so subprocess execution with manual environment setup is required.

**Implementation Notes:**
- Parse environment file to extract KEY=VALUE pairs
- Support both `export KEY=VALUE` and `KEY=VALUE` formats
- Handle quoted values (single and double quotes)
- Set environment variables on the child process
- Return exit code from subprocess

#### Scenario: Windows execution
- **WHEN** executing on Windows
- **AND** environment file contains variables
- **THEN** parse file and set environment variables
- **AND** spawn subprocess with those variables
- **AND** return subprocess exit code

#### Scenario: Parse quoted values
- **WHEN** environment file contains `KEY="value with spaces"`
- **THEN** extract value without quotes: `value with spaces`

### Requirement: EXEC-004 Command Passthrough
**Requirement:** The system SHALL pass all provided arguments directly to the target command unchanged.

**Rationale:** Maintains full compatibility with the underlying command and its options.

**Implementation Notes:**
- Preserve argument order
- Join arguments with spaces for shell command
- Shell handles quote preservation and special characters

#### Scenario: Simple arguments
- **WHEN** args are `["--help"]`
- **THEN** command receives `--help`

#### Scenario: Multiple arguments
- **WHEN** args are `["--model", "claude-3", "--output", "file.txt"]`
- **THEN** command receives all arguments in order

#### Scenario: No arguments
- **WHEN** args are empty
- **THEN** command is executed with no arguments

### Requirement: EXEC-005 Exit Code Propagation
**Requirement:** The system SHALL propagate the exit code from the executed command to the caller.

**Rationale:** Allows scripts and users to detect command success or failure.

**Implementation Notes:**
- On Unix with exec: current process is replaced, exit code is automatic
- On Windows: capture subprocess exit code and call exit()
- Exit code 127 for command not found
- Exit code 1 for other errors

#### Scenario: Successful command
- **WHEN** command exits with code 0
- **THEN** system exits with code 0

#### Scenario: Failed command
- **WHEN** command exits with non-zero code
- **THEN** system exits with same code

#### Scenario: Command not found
- **WHEN** command is not found
- **THEN** system exits with code 127

### Requirement: EXEC-006 Standard Streams
**Requirement:** The system SHALL connect stdout and stderr from the command directly to the user's terminal.

**Rationale:** Provides transparent interaction with the command, showing all output in real-time.

**Implementation Notes:**
- On Unix with exec: streams are inherited automatically
- On Windows subprocess: inherit parent's stdio handles
- No buffering or modification of output

#### Scenario: Command output
- **WHEN** command writes to stdout
- **THEN** user sees output immediately

#### Scenario: Command errors
- **WHEN** command writes to stderr
- **THEN** user sees errors immediately

### Requirement: EXEC-007 Configurable Command Name
**Requirement:** The system SHALL accept a configurable command name, defaulting to "claude".

**Rationale:** Allows users to use alternative executables while maintaining backward compatibility.

**Implementation Notes:**
- Command name is passed from CLI layer
- Default is "claude"
- Used in shell command construction

#### Scenario: Default command
- **WHEN** no custom command specified
- **THEN** use "claude" as the command

#### Scenario: Custom command
- **WHEN** custom command "claude-code" is specified
- **THEN** use "claude-code" in shell command
