## 1. Implementation
- [x] 1.1 Replace Rust line-by-line parsing with shell source execution
- [x] 1.2 Update Environment::from_file() to use Command::new("sh")
- [x] 1.3 Implement parse_env_line() function for env command output
- [x] 1.4 Add ShellCommandFailed error type to CceError
- [x] 1.5 Update imports (remove unused Path, add process::Command)

## 2. Testing
- [x] 2.1 Create test fixtures for different .env formats
- [x] 2.2 Add integration tests for basic KEY=value format
- [x] 2.3 Add tests for variable reference (FULL_URL="${BASE_URL}/v1")
- [x] 2.4 Add tests for command substitution (TOKEN=$(echo 'test'))
- [x] 2.5 Add tests for complex shell logic (conditionals)
- [x] 2.6 Verify all existing tests still pass

## 3. Validation
- [x] 3.1 Run cargo test - all 11 tests pass
- [x] 3.2 Run cargo clippy - no warnings
- [x] 3.3 Build release version successfully
- [x] 3.4 Test with actual .env files containing shell syntax

## 4. Documentation
- [x] 4.1 Update version to 2.0.3 in Cargo.toml and main.rs
- [x] 4.2 Remove old parse_line() and strip_quotes() functions
- [x] 4.3 Clean up unused imports

## 5. Dependencies
- [x] 5.1 Upgrade clap to 4.5.53
- [x] 5.2 Upgrade thiserror to 2.0.17
- [x] 5.3 Upgrade tempfile to 3.23.0
- [x] 5.4 Verify all upgrades work correctly