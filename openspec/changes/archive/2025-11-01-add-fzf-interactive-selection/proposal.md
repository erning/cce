# Add fzf Interactive Environment Selection

## Why
Currently, when users run `cce` without arguments, it only lists available environments. Users must manually type the environment name to use it. This requires memorizing environment names and is less efficient for workflows with many environments.

## What Changes
- Add interactive environment selection using fzf when no environment name is provided
- Detect if fzf is available; fall back to listing mode if not
- Allow users to navigate and select from a list using keyboard shortcuts
- Maintain backward compatibility with direct environment name specification

## Impact
- Affected specs: cli-interface specification (modify existing requirements)
- Affected code: src/main.rs (modify list_environments function)
- Dependencies: Add fzf detection and fallback mechanism
- User experience: Significantly improved for users with many environments

## Technical Approach
1. Detect fzf availability via which command
2. Pipe environment list to fzf for interactive selection
3. Parse fzf output to get selected environment name
4. Fall back to listing mode if fzf not available or selection cancelled
5. Continue with existing run_environment logic for selected environment
