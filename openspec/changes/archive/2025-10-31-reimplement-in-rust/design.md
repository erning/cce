# Design Document: CCE Manager Rust Reimplementation

## Architecture Overview

```
┌─────────────────────────────────────────┐
│                 CLI Layer                │
│  (clap for argument parsing & commands) │
└────────────────┬────────────────────────┘
                 │
┌────────────────▼────────────────────────┐
│            Core Logic Layer             │
│  Environment Manager & Command Executor │
└────────────────┬────────────────────────┘
                 │
┌────────────────▼────────────────────────┐
│           Configuration Layer           │
│      (.env parsing & validation)        │
└────────────────┬────────────────────────┘
                 │
┌────────────────▼────────────────────────┐
│            File System Layer            │
│  ~/.config/cce/ environment files      │
└─────────────────────────────────────────┘
```

## Component Design

### 1. CLI Layer (`src/main.rs`)

**Technology Choice: `clap` v4**
- Mature, well-maintained CLI parsing library
- Derive-based API reduces boilerplate
- Built-in help generation and error messages
- Support for subcommands (list, run)

**Design Decisions:**
- Use derive API for concise code
- Define subcommands explicitly for clarity
- Maintain compatibility with bash version's CLI interface where possible

### 2. Configuration Management (`src/config.rs`)

**Technology Choice: `dotenvy`**
- Drop-in replacement for deprecated `dotenv`
- No-fuss environment file parsing
- Pure Rust implementation

**Data Structure:**
```rust
struct Environment {
    name: String,
    base_url: String,
    auth_token: String,
}
```

**Validation Rules:**
1. Both `ANTHROPIC_BASE_URL` and `ANTHROPIC_AUTH_TOKEN` must be present
2. URLs must be valid (basic validation)
3. Tokens must be non-empty

### 3. Environment Manager (`src/manager.rs`)

**Responsibilities:**
- Scan `~/.config/cce/` directory for `.env` files
- Load and parse environment configurations
- Validate environment data
- Provide listing functionality

**Design Pattern:**
- Repository pattern for environment storage
- Clear separation between loading and validation
- Error accumulation for better error reporting

### 4. Command Executor (`src/executor.rs`)

**Technology Choice: `std::process::Command`**
- Built-in, no external dependencies
- Proper environment variable handling
- Preserves exit codes from claude CLI

**Design:**
- Accept environment + arguments
- Set environment variables before execution
- Forward all stdout/stderr to user
- Pass through exit code from claude

## Error Handling Strategy

### Error Type Hierarchy
```
Box<dyn Error>  (top level)
├── ConfigError (configuration issues)
│   ├── MissingDirectory
│   ├── MissingFile
│   ├── InvalidFormat
│   └── ValidationFailed
├── ExecutionError (runtime issues)
│   ├── ClaudeNotFound
│   └── ExecutionFailed
└── IoError (filesystem issues)
```

### User Experience
- All errors result in clear, actionable messages
- No stack traces for expected errors
- Suggest solutions when possible
- Preserve bash version's helpful error messages

## Key Trade-offs

### 1. Compatibility vs. Improvement
**Decision:** Prioritize backward compatibility
- Keep exact same config directory: `~/.config/cce/`
- Support existing `.env` file format
- Similar CLI interface (where reasonable)
- Allow gradual migration

**Rationale:** Minimize friction for existing users

### 2. Feature Set vs. Complexity
**Decision:** Start with feature parity, add enhancements later
- Phase 1: Match bash functionality exactly
- Future: Add validation, templates, etc.

**Rationale:** Reduces implementation risk, ensures core functionality works

### 3. Dependencies vs. Self-Contained
**Decision:** Use standard Rust ecosystem crates
- clap, dotenvy, anyhow are battle-tested
- Small number of well-maintained dependencies
- Cargo handles dependency resolution

**Rationale:** Faster development, better maintained code

### 4. Error Handling Verbosity
**Decision:** Detailed error messages with suggestions
- Explain what went wrong
- Suggest how to fix it
- Show file paths when relevant

**Rationale:** Better developer experience

## Implementation Details

### .env File Parsing
```rust
// Parse key=value pairs
// Support 'export KEY=value' format (common but not required)
// Ignore comments (# prefix)
// Reject invalid format
```

### Environment Variable Setting
```rust
// Before executing claude:
// ANTHROPIC_BASE_URL=<value>
// ANTHROPIC_AUTH_TOKEN=<value>
std::env::set_var("ANTHROPIC_BASE_URL", env.base_url);
std::env::set_var("ANTHROPIC_AUTH_TOKEN", env.auth_token);
```

### Cross-Platform Considerations
- Path handling: Use `dirs::config_dir()` for config directory
- Path separators: Handle platform differences
- Executable name: `cce` on Unix, `cce.exe` on Windows
- Line endings: Accept both LF and CRLF in .env files

## Testing Strategy

### Unit Tests
- Configuration parsing
- Validation logic
- Error cases
- File system operations (with temp directories)

### Integration Tests
- End-to-end CLI usage
- Real .env file compatibility
- Argument passthrough to claude

### Property Tests
- Environment name handling
- File path handling
- Round-trip parsing/validation

## Performance Considerations

### Startup Time
- Lazy loading of configurations (only load when needed)
- Minimize file I/O before command execution
- Cargo build optimizations

### Runtime Overhead
- Bash spawns subshell for each operation
- Rust binary loads once, minimal overhead
- Expected improvement: 2-5x faster

### Memory Usage
- Small footprint (~5-10MB vs ~20MB for bash + subshell)
- No memory leaks (Rust safety guarantees)
- Constant memory usage

## Future Extensibility

### Plugin Architecture (Future)
- Support for custom providers
- Provider-specific validation
- Custom command hooks

### Configuration Format Evolution (Future)
- Support JSON/YAML configs alongside .env
- Configuration inheritance
- Environment variable interpolation

### Features Roadmap
1. Environment templates
2. Interactive creation wizard
3. Configuration validation before use
4. Sync configurations across devices
5. Provider health checks

## Migration Path

### For Users
1. Install Rust binary alongside bash version
2. Test with same environment files
3. Migrate aliases or shell wrappers gradually
4. Remove bash version when comfortable

### For Developers
1. New development happens on Rust version
2. Bug fixes backported if critical
3. Bash version maintained only for legacy support

## Security Considerations

### Environment Variables
- Never log auth tokens
- Clear sensitive data from memory when possible
- Validate all inputs before use

### File Permissions
- Check .env file permissions
- Warn on world-readable config files
- Respect user's umask

### Supply Chain Security
- Minimal dependencies
- Pin dependency versions
- Audit dependencies regularly
