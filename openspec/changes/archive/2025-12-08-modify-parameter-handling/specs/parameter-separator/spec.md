# Parameter Separator Support Specification

## ADDED Requirements

### Requirement: Explicit '--' Separator Recognition
The CCE tool SHALL explicitly recognize and handle the `--` separator to distinguish between CCE options and arguments to be passed to claudecode.

#### Scenario: Basic '--' separator usage
Given a user runs `cce glm -- --help`
Then CCE should:
- Parse `--help` as an argument to pass to claudecode
- Not attempt to interpret `--help` as a CCE option
- Execute `claude --help` within the glm environment

#### Scenario: Multiple arguments after '--'
Given a user runs `cce kimi -- --version --model claude-3`
Then CCE should:
- Parse `--version --model claude-3` as arguments for claudecode
- Pass all arguments after `--` to the claude command
- Execute `claude --version --model claude-3` within the kimi environment

### Requirement: CCE Version Display
The CCE tool SHALL display its own version when `--version` is used before any `--` separator.

#### Scenario: CCE version without '--'
Given a user runs `cce --version`
Then CCE should:
- Display the CCE version (e.g., "cce 2.0.5")
- Exit without executing claudecode

#### Scenario: CCE version with environment specified
Given a user runs `cce glm --version`
Then CCE should:
- Display the CCE version (e.g., "cce 2.0.5")
- Exit without executing claudecode
- Not load the glm environment

### Requirement: Claude Version Display via '--'
Arguments after `--` SHALL be passed to claudecode, including `--version`.

#### Scenario: Claude version via '--'
Given a user runs `cce -- --version`
Then CCE should:
- Pass `--version` to claudecode
- Execute `claude --version` (within the default/no environment)
- Display claude's version information

#### Scenario: Environment-specific claude version
Given a user runs `cce minimax -- --version`
Then CCE should:
- Load the minimax environment
- Execute `claude --version` within that environment
- Display claude's version information

## MODIFIED Requirements

### Requirement: Backward Compatibility
The implementation SHALL maintain backward compatibility with the current argument handling behavior when `--` is not used.

#### Scenario: Arguments without '--' separator
Given a user runs `cce glm --help`
Then CCE should:
- Continue to work as before (backward compatibility)
- Pass `--help` to claudecode
- Execute `claude --help` within the glm environment

#### Scenario: Mixed arguments without '--'
Given a user runs `cce kimi --model claude-3 --verbose`
Then CCE should:
- Pass all arguments to claudecode
- Execute `claude --model claude-3 --verbose` within the kimi environment
- Maintain existing behavior

## REMOVED Requirements

- The current behavior of collecting all trailing arguments without explicit `--` separator is deprecated in favor of explicit separator support
- Documentation should be updated to recommend using `--` for clarity
- Guide users toward explicit separator usage while maintaining backward compatibility