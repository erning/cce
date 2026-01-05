# environment-management Specification

## Purpose
Defines how CCE manages environment configurations, including storage, discovery, loading, and validation of environment files.

## Requirements

### Requirement: ENV-001 Environment Configuration Storage
**Requirement:** The system SHALL store environment configurations as individual `.env` files in the user's XDG-compliant config directory.

**Rationale:** Provides organized, discoverable storage following platform conventions.

**Implementation Notes:**
- Primary directory: `$XDG_CONFIG_HOME/cce/` or `~/.config/cce/`
- File naming: `<environment_name>.env`
- Format: Shell-compatible .env with `export KEY=VALUE` or `KEY=VALUE`

#### Scenario: Environment file storage
- **WHEN** user creates `~/.config/cce/glm.env`
- **THEN** system recognizes "glm" as an environment name

#### Scenario: Multiple environments
- **WHEN** config directory contains `glm.env`, `kimi.env`, `minimax.env`
- **THEN** system recognizes three environments

### Requirement: ENV-002 Environment Discovery
**Requirement:** The system SHALL scan the config directory and identify all available environment files with `.env` extension.

**Rationale:** Enables users to see available environments without prior knowledge.

**Implementation Notes:**
- Scan directory for files with `.env` extension
- Extract environment name from filename (strip `.env` suffix)
- Sort alphabetically for consistent output
- Handle empty directories gracefully

#### Scenario: List environments
- **WHEN** config directory contains environments
- **THEN** return sorted list of environment names

#### Scenario: Empty directory
- **WHEN** config directory has no .env files
- **THEN** return empty list

#### Scenario: Directory does not exist
- **WHEN** config directory does not exist
- **THEN** return empty list (not an error)

### Requirement: ENV-003 Environment Validation
**Requirement:** The system SHALL validate that each environment file contains the required variable `ANTHROPIC_AUTH_TOKEN`.

**Rationale:** Ensures configurations have necessary authentication before use.

**Implementation Notes:**
- Required: `ANTHROPIC_AUTH_TOKEN`
- Optional: `ANTHROPIC_BASE_URL`
- Parse file to check variable presence
- Report missing required and optional variables separately

#### Scenario: Valid with all variables
- **WHEN** file contains both ANTHROPIC_AUTH_TOKEN and ANTHROPIC_BASE_URL
- **THEN** validation passes with no warnings

#### Scenario: Valid with required only
- **WHEN** file contains only ANTHROPIC_AUTH_TOKEN
- **THEN** validation passes
- **AND** note: missing optional ANTHROPIC_BASE_URL

#### Scenario: Invalid missing required
- **WHEN** file lacks ANTHROPIC_AUTH_TOKEN
- **THEN** validation fails
- **AND** report: missing required ANTHROPIC_AUTH_TOKEN

### Requirement: ENV-004 Environment Loading
**Requirement:** The system SHALL load environment configuration by providing the file path to the command executor.

**Rationale:** Separates concerns between environment management and command execution.

**Implementation Notes:**
- Verify file exists before loading
- Return file path for executor to use
- Handle missing files with clear error messages

#### Scenario: Load existing environment
- **WHEN** environment "glm" exists
- **AND** user requests to load it
- **THEN** return path to glm.env file

#### Scenario: Load missing environment
- **WHEN** environment "invalid" does not exist
- **AND** user requests to load it
- **THEN** return error with file path

### Requirement: ENV-005 Environment Data Structure
**Requirement:** The system SHALL represent environments with name and file path for efficient management.

**Rationale:** Minimal data structure focuses on what's needed for file-based execution model.

**Implementation Notes:**
- Environment struct contains: name (String), file_path (PathBuf)
- Created from file path during discovery
- Validation performed on-demand from file content

#### Scenario: Create environment from file
- **WHEN** file `~/.config/cce/glm.env` is discovered
- **THEN** create Environment with name="glm" and file_path pointing to file

### Requirement: ENV-006 EnvironmentManager Interface
**Requirement:** The system SHALL provide an EnvironmentManager that handles all environment operations.

**Rationale:** Centralizes environment operations for consistent behavior.

**Implementation Notes:**
- `new()`: Create manager with XDG-compliant config directory
- `config_dir()`: Return the config directory path
- `list_environments()`: Return all discovered environments
- `get_environment_file(name)`: Get file path for named environment
- `load_environment(name)`: Load environment by name

#### Scenario: Create manager
- **WHEN** EnvironmentManager::new() is called
- **THEN** manager is created with appropriate config directory

#### Scenario: List all environments
- **WHEN** manager.list_environments() is called
- **THEN** return sorted Vec of Environment structs

#### Scenario: Get environment file
- **WHEN** manager.get_environment_file("glm") is called
- **AND** glm.env exists
- **THEN** return path to glm.env

#### Scenario: Get missing environment file
- **WHEN** manager.get_environment_file("invalid") is called
- **AND** invalid.env does not exist
- **THEN** return MissingFile error
