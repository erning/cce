# Configuration Loading Specification

## Overview
The Configuration Loading capability handles reading, parsing, and validating environment configuration files in the .env format.

## ADDED Requirements

### Requirement: CONF-001 File Format Support
**Requirement:** The system SHALL support standard .env file format with key=value pairs, with optional 'export' prefix.

**Rationale:** Maintains compatibility with existing environment files and follows .env file conventions.

**Implementation Notes:**
- Parse lines with format: `KEY=value` or `export KEY=value`
- Ignore empty lines and lines starting with #
- Support quoted values (both single and double quotes)
- Support environment variable interpolation ($VAR or ${VAR})

#### Scenario: Standard .env format
```
Given file contains:
  ANTHROPIC_BASE_URL=https://api.example.com
  ANTHROPIC_AUTH_TOKEN=secret123
When system parses the file
Then extract:
  base_url: "https://api.example.com"
  auth_token: "secret123"
```

#### Scenario: Export prefix format
```
Given file contains:
  export ANTHROPIC_BASE_URL="https://api.example.com"
  export ANTHROPIC_AUTH_TOKEN="secret123"
When system parses the file
Then extract the same values as standard format
```

#### Scenario: Comments and empty lines
```
Given file contains:
  # This is a comment

  export ANTHROPIC_BASE_URL=https://api.example.com
  # Another comment
  export ANTHROPIC_AUTH_TOKEN=secret123

When system parses the file
Then ignore comments and empty lines, extract only the key-value pairs
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
**Requirement:** The system SHALL read environment files from the config directory with proper error handling.

**Rationale:** Core functionality to load configuration data from disk.

**Implementation Notes:**
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

## Removed Requirements
None.

## Edge Cases

### Multiple Occurrences
```
Given file contains ANTHROPIC_BASE_URL twice
When system parses the file
Then use the last value encountered
```

### Extra Whitespace
```
Given ANTHROPIC_BASE_URL =  https://api.example.com  \n
When system parses the file
Then trim whitespace and extract: "https://api.example.com"
```

### Case Sensitivity
```
Given file contains:
  anthropic_base_url=https://api.example.com
  ANTHROPIC_AUTH_TOKEN=secret123
When system validates the file
Then fail validation because field name is case-sensitive
```

## Success Criteria
1. All existing .env files parse correctly
2. Proper validation of required fields
3. Clear error messages for all failure scenarios
4. Support for standard .env format features
5. Robust handling of edge cases

## References
- .env file format specification: https://github.com/motdotla/dotenv#rules
- dotenvy crate (Rust implementation)
- Current bash implementation uses shell `source` command (line 87 in /cce)
