# Change: Update Environment Loading to Use Shell Source

## Why
The current implementation parses .env files line-by-line in Rust, which limits the supported syntax to basic `KEY=value` format. Users need more flexible shell syntax including variable references, command substitution, and conditional logic in their environment files.

## What Changes
- Replace Rust-based line-by-line parsing with shell `source` command execution
- Enable full shell syntax support in .env files (variable references, command substitution, conditionals)
- Maintain backward compatibility with existing simple KEY=value formats
- Add comprehensive test coverage for shell features

## Impact
- Affected specs: configuration-loading
- Affected code: src/config.rs (Environment::from_file implementation)
- No breaking changes - existing .env files continue to work
- New capabilities: shell syntax support in environment files