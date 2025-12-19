# Change: Add Configurable Command Executable

## Why
Currently cce hardcodes the execution of the `claude` command. Users may need to use alternative executable names (e.g., `claude-code`, custom wrapper scripts), requiring a configurable option.

## What Changes
- Add optional `--command` / `-c` CLI parameter to specify the executable name to run
- Default value remains `claude` to ensure backward compatibility
- Modify `CommandExecutor` to accept a configurable command name

## Impact
- Affected specs: `cli-interface`, `command-execution`
- Affected code: `src/main.rs`, `src/executor.rs`
