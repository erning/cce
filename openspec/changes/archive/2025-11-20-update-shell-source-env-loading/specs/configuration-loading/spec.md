## MODIFIED Requirements

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