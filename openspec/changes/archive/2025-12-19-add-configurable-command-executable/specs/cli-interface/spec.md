## ADDED Requirements

### Requirement: CLI-007 Configurable Command Executable
**Requirement:** The system SHALL accept an optional `--command` (or `-c`) parameter to specify the executable name to run, with a default value of "claude".

**Rationale:** Allows users to use alternative executables (e.g., `claude-code`, custom wrapper scripts) while maintaining backward compatibility.

**Implementation Notes:**
- Add `--command` / `-c` as a global option in the CLI struct
- Default value is "claude" to maintain backward compatibility
- The parameter value is passed to the command executor
- Works with both direct environment execution and fzf selection

#### Scenario: Default executable behavior
- **WHEN** user runs `cce glm` without `--command` parameter
- **THEN** execute the default `claude` command with glm environment

#### Scenario: Custom executable specified
- **WHEN** user runs `cce --command claude-code glm`
- **THEN** execute `claude-code` command with glm environment

#### Scenario: Short flag for custom executable
- **WHEN** user runs `cce -c my-claude glm --help`
- **THEN** execute `my-claude --help` with glm environment

#### Scenario: Custom executable with fzf selection
- **WHEN** user runs `cce --command claude-code` without environment name
- **AND** fzf is available
- **THEN** show fzf selection interface
- **WHEN** user selects an environment
- **THEN** execute `claude-code` with the selected environment
