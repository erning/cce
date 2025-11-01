## MODIFIED Requirements
### Requirement: CLI-001 List Environments Subcommand
**Requirement:** When executed without an environment name argument, the system SHALL attempt to use fzf for interactive environment selection. If fzf is not available or the user cancels the selection, it SHALL list all available environments.

**Rationale:** Enhance user experience by allowing interactive selection while maintaining backward compatibility.

**Implementation Notes:**
- Detect fzf availability using std::process::Command
- Display fzf interface for environment selection
- Accept keyboard navigation (arrows, j/k, Ctrl+N/Ctrl+P)
- Accept Enter to confirm selection
- Accept ESC or q to cancel and fall back to list view
- Display environments in alphabetical order in fzf
- If fzf unavailable or selection cancelled, show traditional list view
- Include helpful usage information

#### Scenario: Interactive selection with fzf available
```
Given fzf is installed and environments "glm" and "kimi" exist
When user runs "cce" without arguments
Then display fzf interface for environment selection
When user navigates to "glm" and presses Enter
Then load glm environment and execute claude
```

#### Scenario: Interactive selection with fzf unavailable
```
Given fzf is not installed and environments "glm" and "kimi" exist
When user runs "cce" without arguments
Then display:
  Usage: cce <name> [claude-code arguments...]
    glm
    kimi
And exit with code 1
```

#### Scenario: User cancels interactive selection
```
Given fzf is installed and environments "glm" and "kimi" exist
When user runs "cce" without arguments
Then display fzf interface for environment selection
When user presses ESC or q
Then display:
  Usage: cce <name> [claude-code arguments...]
    glm
    kimi
And exit with code 1
```

#### Scenario: No environments available with fzf
```
Given fzf is installed but no environments exist in ~/.config/cce/
When user runs "cce" without arguments
Then display:
  Usage: cce <name> [claude-code arguments...]
    (no environment found)
And exit with code 1
```
