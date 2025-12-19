# environment-management Specification

## Purpose
TBD - created by archiving change reimplement-in-rust. Update Purpose after archive.
## Requirements
### Requirement: ENV-MGMT-001 Environment Configuration Storage
**Requirement:** The system SHALL store environment configurations as individual `.env` files in the user's config directory (`~/.config/cce/`).

**Rationale:** Maintains backward compatibility with bash implementation while providing organized storage.

**Implementation Notes:**
- Directory: `~/.config/cce/` on Unix-like systems
- File naming: `<environment_name>.env`
- Format: Standard .env format with `export KEY=value` statements

#### Scenario: Environment file storage
```
Given the config directory ~/.config/cce/ exists
When a user creates a file named "glm.env" with ANTHROPIC_BASE_URL and ANTHROPIC_AUTH_TOKEN
Then the system shall recognize "glm" as a valid environment name
```

### Requirement: ENV-MGMT-002 Environment Discovery
**Requirement:** The system SHALL scan the config directory and identify all available environment files with `.env` extension.

**Rationale:** Enables users to see what environments are available without specifying a particular one.

**Implementation Notes:**
- Must handle empty directories gracefully
- Ignore non-.env files
- Sort environment names alphabetically for consistent output

#### Scenario: Listing available environments
```
Given environments "glm", "kimi", and "minimax" exist in ~/.config/cce/
When the user runs "cce" without arguments
Then the system shall display:
  glm
  kimi
  minimax
```

#### Scenario: Empty environment directory
```
Given no .env files exist in ~/.config/cce/
When the user runs "cce" without arguments
Then the system shall display "(no environment found)"
```

### Requirement: ENV-MGMT-003 Environment Validation
**Requirement:** The system SHALL validate that each environment file contains the required variable `ANTHROPIC_AUTH_TOKEN`. `ANTHROPIC_BASE_URL` is optional.

**Rationale:** Ensures authentication is configured. Base URL is optional as some providers may use defaults.

**Implementation Notes:**
- `ANTHROPIC_AUTH_TOKEN` must be present and non-empty
- `ANTHROPIC_BASE_URL` is optional (noted as missing optional if absent)
- No format validation on values

#### Scenario: Valid environment file with both variables
```
Given an environment file containing:
  export ANTHROPIC_BASE_URL="https://api.example.com"
  export ANTHROPIC_AUTH_TOKEN="secret123"
When the system validates this environment
Then it shall accept the configuration as valid
```

#### Scenario: Valid environment file with token only
```
Given an environment file containing only:
  export ANTHROPIC_AUTH_TOKEN="secret123"
When the system validates this environment
Then it shall accept the configuration as valid
And note missing optional: ANTHROPIC_BASE_URL
```

#### Scenario: Missing auth token
```
Given an environment file containing only:
  export ANTHROPIC_BASE_URL="https://api.example.com"
When the system validates this environment
Then it shall reject the configuration with error:
  Missing required: ANTHROPIC_AUTH_TOKEN
```

### Requirement: ENV-MGMT-004 Environment Loading
**Requirement:** The system SHALL load environment configuration from a specified environment file and make it available for command execution.

**Rationale:** Core functionality to apply environment settings before running claude commands.

**Implementation Notes:**
- Parse .env file format
- Extract environment variables
- Make them available to child process

#### Scenario: Loading valid environment
```
Given an environment named "glm" exists with valid configuration
When the user runs "cce glm --help"
Then the system shall load the glm environment and execute claude with those variables set
```

All requirements in this capability are new implementations of existing functionality from the Bash version.

