# Proposal: Modify Parameter Handling to Support '--' Separator

## Summary

Modify the CCE parameter handling to properly support the `--` separator, enabling users to distinguish between CCE flags and arguments to be passed to claudecode. This change will allow:
- `cce <model> --version` to display CCE's version
- `cce <model> -- --version` to display claudecode's version
- Clear separation between CCE options and claudecode arguments

## Why

Currently, CCE collects all trailing arguments in a single vector without explicit handling of the `--` separator. While the current implementation works due to `clap`'s `trailing_var_arg` feature, it doesn't provide the semantic distinction that `--` traditionally represents in Unix tools.

## What Changes

1. **Unix Convention**: The `--` separator is a standard Unix convention indicating "end of options, beginning of positional arguments"
2. **Clarity**: Users expect `--` to clearly separate tool options from pass-through arguments
3. **Compatibility**: Aligns with user expectations from other CLI tools
4. **Future-proofing**: Enables potential addition of CCE-specific flags without ambiguity

## Scope

This proposal covers:
- Modifying argument parsing to explicitly recognize `--` separator
- Updating both Rust and shell implementations
- Ensuring backward compatibility
- Adding comprehensive tests for the new behavior

## Out of Scope

- Adding new CCE-specific command-line options (beyond --version)
- Modifying environment file format or structure
- Changing the core execution logic beyond argument handling