## 1. Implementation

- [x] 1.1 Update `list_environments` to accept args parameter
- [x] 1.2 Pass args to `run_environment` after fzf selection
- [x] 1.3 Update main function to pass cli.args to list_environments
- [x] 1.4 Handle case where "name" starts with `-` (treat as args)

## 2. Testing

- [x] 2.1 Run `cargo build` to verify compilation
- [x] 2.2 Run `cargo test` to verify tests pass
- [x] 2.3 Manual test `cce -- --help` shows fzf, then passes --help after selection
