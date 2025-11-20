<!-- OPENSPEC:START -->
# OpenSpec Instructions

These instructions are for AI assistants working in this project.

Always open `@/openspec/AGENTS.md` when the request:
- Mentions planning or proposals (words like proposal, spec, change, plan)
- Introduces new capabilities, breaking changes, architecture shifts, or big performance/security work
- Sounds ambiguous and you need the authoritative spec before coding

Use `@/openspec/AGENTS.md` to learn:
- How to create and apply change proposals
- Spec format and conventions
- Project structure and guidelines

Keep this managed block so 'openspec update' can refresh the instructions.

<!-- OPENSPEC:END -->

# Agent Guidelines for CCE Project

## Build/Lint/Test Commands

**Core Commands:**
- `cargo build` - Build debug version
- `cargo build --release` - Build optimized release version
- `cargo test` - Run all tests
- `cargo test <test_name>` - Run specific test by name
- `cargo clippy` - Run linting checks
- `cargo fmt` - Format code

**Development:**
- `./cce` - Run the script version
- `cargo run -- <args>` - Run rust version with arguments

## Code Style Guidelines

**Imports & Dependencies:**
- Group imports: standard library → external crates → local modules
- Use `use crate::` for local module imports
- Dependencies: `clap` (CLI), `thiserror` (errors), `tempfile` (testing)

**Naming Conventions:**
- Functions/variables: `snake_case`
- Types: `PascalCase` 
- Constants: `SCREAMING_SNAKE_CASE`
- Files: `snake_case.rs`

**Error Handling:**
- Use `thiserror::Error` derive for error types
- Define custom error enum `CceError` with `#[error("message")]` attributes
- Return `Result<T>` from functions that can fail
- Use `?` operator for propagateable errors

**Code Organization:**
- Keep modules small and focused
- Use `#[cfg(test)]` modules for tests
- Public items need `///` doc comments
- Follow XDG Base Directory specification for config paths

**Testing:**
- Write unit tests in `#[cfg(test)]` modules
- Use `tempfile` for file system testing
- Test with `cargo test` or specific test names
- Include validation tests for environment parsing

**Security:**
- Never log or expose API tokens
- Validate all input paths and URLs
- Use proper error handling for file operations
