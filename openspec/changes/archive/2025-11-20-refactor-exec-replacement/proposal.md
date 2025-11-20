# Change: Replace Subprocess Execution with Process Exec

## Why
The current implementation spawns Claude as a subprocess using `std::process::Command::status()`, which creates unnecessary process overhead. By using process exec replacement on Unix systems, we can eliminate the parent CCE process and directly replace it with the Claude process, improving performance and reducing resource usage.

## What Changes
- **MODIFIED**: `src/executor.rs` - Replace subprocess execution with platform-specific exec
- **ADDED**: Unix-specific exec implementation using `std::os::unix::process::CommandExt::exec()`
- **PRESERVED**: Windows compatibility with existing subprocess fallback
- **UPDATED**: Error handling for exec failure scenarios with standard exit codes

## Impact
- **Affected specs**: `execution` capability
- **Affected code**: `src/executor.rs` - CommandExecutor::execute() method
- **Performance**: Eliminates process overhead on Unix systems
- **Compatibility**: Maintains Windows support with subprocess fallback
- **Behavior**: No user-facing changes, same functionality with better performance
