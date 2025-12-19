# Change: Add Argument Passthrough with Environment Selection

## Why
Currently, when no environment name is provided, cce lists environments or shows fzf selection, but any arguments after `--` are lost. Users should be able to run `cce -- --help` to first select an environment (via fzf or list), then pass `--help` to the executed command.

## What Changes
- When `cce -- <args>` is invoked, behave the same as `cce` (list environments or fzf selection)
- After environment selection, pass the arguments after `--` to the command
- Example: `cce -- --help` → select environment → execute `claude --help` with selected environment

## Impact
- Affected specs: `cli-interface`
- Affected code: `src/main.rs`
