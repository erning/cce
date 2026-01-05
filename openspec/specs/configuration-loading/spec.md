# configuration-loading Specification

## Purpose
Defines how CCE loads, parses, and validates environment configuration files. Environment files use shell-compatible .env format and are executed via shell source command for full shell syntax support.

## Requirements

### Requirement: CONF-001 File Format Support
**Requirement:** The system SHALL support standard .env file format with key=value pairs, with optional 'export' prefix, and full shell syntax through shell source execution.

**Rationale:** Enables powerful environment file scripting while maintaining compatibility with existing files.

**Implementation Notes:**
- Environment files are executed via shell `source` command at runtime
- Support `export KEY=VALUE` and `KEY=VALUE` formats for static validation
- Full shell syntax supported at runtime: variable references, command substitution, conditionals
- Static validation only parses export statements for variable presence checking
- Shell handles all value expansion and special character processing

#### Scenario: Simple key-value format
- **WHEN** file contains `ANTHROPIC_AUTH_TOKEN=secret123`
- **AND** system validates the file
- **THEN** recognize ANTHROPIC_AUTH_TOKEN as present

#### Scenario: Export prefix format
- **WHEN** file contains `export ANTHROPIC_AUTH_TOKEN="my-token"`
- **AND** system validates the file
- **THEN** recognize ANTHROPIC_AUTH_TOKEN as present

#### Scenario: Shell syntax at runtime
- **WHEN** file contains shell constructs like `$(cat ~/.token)` or `${BASE}/path`
- **AND** system executes with this environment
- **THEN** shell expands these at runtime

#### Scenario: Comments ignored
- **WHEN** file contains lines starting with `#`
- **AND** system parses the file
- **THEN** skip comment lines

### Requirement: CONF-002 Required Fields Validation
**Requirement:** The system SHALL validate that the required field `ANTHROPIC_AUTH_TOKEN` is present. `ANTHROPIC_BASE_URL` is optional.

**Rationale:** Ensures environment configurations have authentication before attempting to use them. Base URL is optional as some providers may use defaults.

**Implementation Notes:**
- `ANTHROPIC_AUTH_TOKEN` must be present in the file
- `ANTHROPIC_BASE_URL` is optional (reported as missing optional if not present)
- Validation checks for variable name presence, not value content
- Value validation delegated to the underlying command at runtime

#### Scenario: Both fields present
- **WHEN** file contains both ANTHROPIC_AUTH_TOKEN and ANTHROPIC_BASE_URL
- **AND** system validates the file
- **THEN** validation passes with no warnings

#### Scenario: Only token present (valid)
- **WHEN** file contains only ANTHROPIC_AUTH_TOKEN
- **AND** system validates the file
- **THEN** validation passes
- **AND** note missing optional: ANTHROPIC_BASE_URL

#### Scenario: Missing ANTHROPIC_AUTH_TOKEN
- **WHEN** file contains only ANTHROPIC_BASE_URL
- **AND** system validates the file
- **THEN** validation fails
- **AND** report missing required: ANTHROPIC_AUTH_TOKEN

### Requirement: CONF-003 XDG Base Directory Compliance
**Requirement:** The system SHALL read environment files from the XDG-compliant config directory with proper fallback behavior.

**Rationale:** Ensures cross-platform compatibility using XDG Base Directory specification.

**Implementation Notes:**
- Check `$XDG_CONFIG_HOME` environment variable first
- If set and non-empty, use `$XDG_CONFIG_HOME/cce/`
- If not set or empty, use `$HOME/.config/cce/`
- Handle missing HOME variable gracefully with error

#### Scenario: Default config location
- **WHEN** XDG_CONFIG_HOME is not set
- **AND** HOME is `/home/user`
- **THEN** use `/home/user/.config/cce/` as config directory

#### Scenario: Custom XDG_CONFIG_HOME
- **WHEN** XDG_CONFIG_HOME is `/custom/config`
- **THEN** use `/custom/config/cce/` as config directory

#### Scenario: Empty XDG_CONFIG_HOME falls back
- **WHEN** XDG_CONFIG_HOME is set but empty
- **THEN** use `$HOME/.config/cce/` as config directory

### Requirement: CONF-004 Environment File Discovery
**Requirement:** The system SHALL discover all `.env` files in the config directory and treat filenames (without extension) as environment names.

**Rationale:** Enables automatic environment discovery without manual registration.

**Implementation Notes:**
- Scan config directory for files with `.env` extension
- Environment name is filename without `.env` suffix
- Ignore non-.env files
- Sort environments alphabetically for consistent display

#### Scenario: Multiple environments
- **WHEN** config directory contains `glm.env`, `kimi.env`, `minimax.env`
- **THEN** discover environments: "glm", "kimi", "minimax"
- **AND** return them in alphabetical order

#### Scenario: Ignore non-env files
- **WHEN** config directory contains `glm.env` and `readme.txt`
- **THEN** only discover "glm" environment

#### Scenario: Empty directory
- **WHEN** config directory exists but contains no .env files
- **THEN** return empty environment list

### Requirement: CONF-005 File Reading and Error Handling
**Requirement:** The system SHALL handle file system errors gracefully with clear error messages.

**Rationale:** Provides actionable feedback when configuration files cannot be read.

**Implementation Notes:**
- Check file existence before reading
- Handle I/O errors with descriptive messages
- Report file path in error messages

#### Scenario: File not found
- **WHEN** environment file does not exist
- **AND** system attempts to load it
- **THEN** return error with file path

#### Scenario: Read existing file
- **WHEN** environment file exists and is readable
- **AND** system reads the file
- **THEN** return file content for validation

#### Scenario: Directory not found
- **WHEN** config directory does not exist
- **AND** system lists environments
- **THEN** return empty list (not an error)
