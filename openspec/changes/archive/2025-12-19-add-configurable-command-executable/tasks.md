## 1. Implementation

- [x] 1.1 Add `--command` / `-c` parameter to `Cli` struct with default value "claude"
- [x] 1.2 Modify `run_environment` function signature to accept command parameter
- [x] 1.3 Modify `CommandExecutor::execute` function signature to accept executable name parameter
- [x] 1.4 Update Unix implementation in `executor.rs` to use configurable command name
- [x] 1.5 Update Windows implementation in `executor.rs` to use configurable command name
- [x] 1.6 Update `list_environments` fzf selection to pass command parameter

## 2. Testing

- [x] 2.1 Run `cargo build` to verify compilation
- [x] 2.2 Run `cargo test` to verify tests pass
- [x] 2.3 Manual test default behavior (without --command parameter)
- [x] 2.4 Manual test custom command (with --command parameter)
