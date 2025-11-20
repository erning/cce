# configuration-loading Specification

## Purpose
TBD - created by archiving change reimplement-in-rust. Update Purpose after archive.
## Requirements
### Requirement: CONF-001 File Format Support
**Requirement:** The system SHALL support standard .env file format with key=value pairs, with optional 'export' prefix, and full shell syntax including variable references, command substitution, and conditional logic.

**Rationale:** Enables powerful environment file scripting while maintaining compatibility with existing files.

**Implementation Notes:**
- Execute .env files using shell `source` command instead of line-by-line parsing
- Support full shell syntax: variable references (${VAR}), command substitution ($()), conditionals (if/then)
- Support quoted strings with spaces and special characters
- Maintain backward compatibility with simple KEY=value format
- Parse output from `env` command to extract all environment variables

#### Scenario: Variable reference
```
Given file contains:
  BASE_URL="https://api.example.com"
  ANTHROPIC_BASE_URL="${BASE_URL}/v1"
When system parses the file
Then extract base_url: "https://api.example.com/v1"
```

#### Scenario: Command substitution
```
Given file contains:
  ANTHROPIC_BASE_URL=https://api.example.com
  ANTHROPIC_AUTH_TOKEN=$(cat ~/.token)
When system parses the file
Then extract auth_token from the file contents
```

#### Scenario: Conditional logic
```
Given file contains:
  ENV=production
  if [ "$ENV" = "production" ]; then
    export ANTHROPIC_AUTH_TOKEN=sk_prod_token
  else
    export ANTHROPIC_AUTH_TOKEN=sk_dev_token
  fi
When system parses the file
Then extract auth_token based on the conditional logic
```

#### Scenario: Backward compatibility
```
Given file contains simple KEY=value format:
  ANTHROPIC_BASE_URL=https://api.example.com
  ANTHROPIC_AUTH_TOKEN=secret123
When system parses the file
Then extract the same values as before
```

#### Scenario: Shell error handling
```
Given file contains invalid shell syntax
When system attempts to source the file
Then return error: "Failed to source environment file: [shell error]"
```

### Requirement: CONF-002 Required Fields Validation
**Requirement:** The system SHALL validate that required fields `ANTHROPIC_BASE_URL` and `ANTHROPIC_AUTH_TOKEN` are present and non-empty.

**Rationale:** Ensures environment configurations are complete before attempting to use them.

**Implementation Notes:**
- Both fields must be present in the file
- Values must not be empty strings after trimming
- URL must be a valid URL format (basic validation)
- Token must be a non-empty string

#### Scenario: Both required fields present
```
Given file contains:
  ANTHROPIC_BASE_URL=https://api.example.com
  ANTHROPIC_AUTH_TOKEN=secret123
When system validates the file
Then validation passes
```

#### Scenario: Missing ANTHROPIC_BASE_URL
```
Given file contains only:
  ANTHROPIC_AUTH_TOKEN=secret123
When system validates the file
Then validation fails with error:
  Missing required field: ANTHROPIC_BASE_URL
```

#### Scenario: Missing ANTHROPIC_AUTH_TOKEN
```
Given file contains only:
  ANTHROPIC_BASE_URL=https://api.example.com
When system validates the file
Then validation fails with error:
  Missing required field: ANTHROPIC_AUTH_TOKEN
```

#### Scenario: Empty base URL
```
Given file contains:
  ANTHROPIC_BASE_URL=
  ANTHROPIC_AUTH_TOKEN=secret123
When system validates the file
Then validation fails with error:
  Field ANTHROPIC_BASE_URL cannot be empty
```

#### Scenario: Empty auth token
```
Given file contains:
  ANTHROPIC_BASE_URL=https://api.example.com
  ANTHROPIC_AUTH_TOKEN=
When system validates the file
Then validation fails with error:
  Field ANTHROPIC_AUTH_TOKEN cannot be empty
```

### Requirement: CONF-003 URL Format Validation
**Requirement:** The system SHALL validate that ANTHROPIC_BASE_URL is a valid URL.

**Rationale:** Prevents runtime errors when making API calls with malformed URLs.

**Implementation Notes:**
- Must parse as valid URL (scheme + host)
- Support http and https schemes
- Reject malformed URLs

#### Scenario: Valid HTTPS URL
```
Given ANTHROPIC_BASE_URL="https://api.example.com/anthropic"
When system validates the URL
Then validation passes
```

#### Scenario: Valid HTTP URL
```
Given ANTHROPIC_BASE_URL="http://localhost:8080"
When system validates the URL
Then validation passes
```

#### Scenario: Invalid URL (missing scheme)
```
Given ANTHROPIC_BASE_URL="api.example.com"
When system validates the URL
Then validation fails with error:
  Invalid URL format for ANTHROPIC_BASE_URL
```

#### Scenario: Invalid URL (malformed)
```
Given ANTHROPIC_BASE_URL="ht!tp://[invalid]"
When system validates the URL
Then validation fails with error:
  Invalid URL format for ANTHROPIC_BASE_URL
```

### Requirement: CONF-004 Environment File Reading
**Requirement:** The system SHALL read environment files from the XDG-compliant config directory with proper error handling.

**Rationale:** Ensures cross-platform compatibility using XDG Base Directory specification. On Unix-like systems, respects `$XDG_CONFIG_HOME` environment variable; defaults to `$HOME/.config/cce`.

**Implementation Notes:**
- Check `$XDG_CONFIG_HOME` environment variable first; if set and non-empty, use `$XDG_CONFIG_HOME/cce`
- If `$XDG_CONFIG_HOME` is not set or empty, use `$HOME/.config/cce`
- Check file existence before reading
- Handle I/O errors gracefully
- Provide clear error messages for file system issues

#### Scenario: Read existing file
```
Given file ~/.config/cce/glm.env exists and contains valid configuration
When system reads the file
Then return parsed configuration with all fields
```

#### Scenario: File not found
```
Given file ~/.config/cce/nonexistent.env does not exist
When system attempts to read the file
Then return error:
  File ~/.config/cce/nonexistent.env not found
```

#### Scenario: Permission denied
```
Given file ~/.config/cce/restricted.env exists but is not readable
When system attempts to read the file
Then return error:
  Permission denied reading ~/.config/cce/restricted.env
```

#### Scenario: Custom XDG_CONFIG_HOME
```
Given XDG_CONFIG_HOME=/custom/path and file /custom/path/cce/test.env exists with valid configuration
When system reads the file
Then return parsed configuration from /custom/path/cce/test.env
```

#### Scenario: Empty XDG_CONFIG_HOME falls back to HOME
```
Given XDG_CONFIG_HOME="" and file ~/.config/cce/fallback.env exists with valid configuration
When system reads the file
Then return parsed configuration from ~/.config/cce/fallback.env
```

### Requirement: CONF-005 Special Character Handling
**Requirement:** The system SHALL properly handle special characters in environment values.

**Rationale:** Ensures compatibility with tokens and URLs that may contain special characters.

**Implementation Notes:**
- Support quoted strings with spaces
- Support escaped characters (\n, \t, \r, \", \')
- Reject values with embedded null bytes
- Trim whitespace from unquoted values

#### Scenario: Quoted value with spaces
```
Given ANTHROPIC_AUTH_TOKEN="Bearer token with spaces"
When system parses the file
Then extract auth_token: "Bearer token with spaces"
```

#### Scenario: Escaped quotes
```
Given ANTHROPIC_AUTH_TOKEN="secret\"with\"quotes"
When system parses the file
Then extract auth_token: "secret\"with\"quotes"
```

#### Scenario: URL with special characters
```
Given ANTHROPIC_BASE_URL="https://api.example.com/path?key=value"
When system parses and validates
Then validation passes
```

All requirements in this capability are new implementations of existing functionality from the Bash version.

