# Design

This document describes how `cce.sh` actually runs. It is intended for
people maintaining or porting the script. End-user behavior lives in
[README.md](README.md).

## Why a Bash script

The job is small: load a `.env` file, then run another command with those
variables. Bash already has both halves built in — `source` reads a file
into the current shell with full shell semantics (variable expansion,
command substitution, conditionals, sourcing other files), and `exec`
replaces the current process so there is no parent overhead and no
intermediate process to manage signals or exit codes.

A compiled implementation would have to either reimplement that shell
semantics (large surface area, easy to get subtly wrong) or shell out and
parse, which is what `cce` already does — without the build step.

The trade-off is that `cce` is Unix-only. Porting it to Windows would mean
parsing env files manually and spawning a subprocess instead of `source` +
`exec` — a meaningful rewrite, not a flag.

## Execution pipeline

A run of `cce` proceeds through these stages:

1. **Argument parsing.** A hand-written `while`/`case` loop walks `$@` and
   collects: `COMMAND` (default `claude`), `ENV_NAME`, `SHOW_HELP`,
   `SHOW_VERSION`, and `ARGS`. The first positional argument becomes
   `ENV_NAME`; subsequent positionals — and everything after a literal
   `--` — go into `ARGS`.
2. **Config directory resolution.** `XDG_CONFIG_HOME/cce` if set and
   non-empty, else `$HOME/.config/cce`. Stored in `ENV_DIR`.
3. **Mode dispatch.** In order: version → help → list/picker (when
   `ENV_NAME` is empty or starts with `-`) → run. The first matching mode
   handles the request and exits.
4. **Environment name validation.** Before any file lookup that uses
   `ENV_NAME`, the name is checked for path traversal. See below.
5. **Source and exec.** The chosen `.env` file is sourced into the current
   shell, then the target command is `exec`'d with `ARGS`.

## Environment name validation

`validate_env_name` rejects an environment name if any of the following
are true:

- it is empty
- it contains `/` (forward slash)
- it contains `\` (backslash)
- it contains `..` (parent-directory token)

This is what prevents an invocation like `cce ../../etc/passwd` from
escaping the config directory.

The check is intentionally a substring test rather than a normalised path
comparison. It is the simplest rule that defends the threat model — names
are user-controlled but config files are not — without trying to interpret
filesystem semantics.

## Environment discovery

`get_env_names` prints sorted environment names to stdout, one per line,
which the callers slurp into a local array via `while IFS= read -r`:

- If `ENV_DIR` does not exist, the function prints nothing (not an error).
- Each filename is reduced to its basename without the `.env` suffix.
- The names are piped through `sort`, so listing and the fzf picker are
  always in alphabetical order.

Discovery is non-recursive and does not follow symlinks specially —
whatever `*.env` matches in `ENV_DIR` is what `cce` sees.

The function deliberately avoids Bash 4+ features (`local -n` namerefs,
`mapfile`/`readarray`) so the script runs unmodified on the stock macOS
`/bin/bash`, which is still 3.2.

## Interactive selection

When `ENV_NAME` is empty, `cce` checks `command -v fzf`. If `fzf` is
available *and* at least one environment exists, it pipes the sorted names
into `fzf` and reads a single line back:

```bash
selected=$(printf "%s\n" "${env_names[@]}" | fzf) || { list_environments; exit 0; }
```

Cancelling fzf (Esc, Ctrl-C, or empty selection) makes the pipeline fail,
which `set -e` would normally abort on; the `|| { ...; exit 0; }` clause
catches that and falls back to the plain list view.

If a name is selected, control falls through to the normal "run" path —
the `--command` and trailing `ARGS` collected during parsing are reused
verbatim.

## Source and exec

The run path is deliberately small:

```bash
ENV_FILE="$ENV_DIR/${ENV_NAME}.env"
# ... existence check ...
. "$ENV_FILE"
exec "$COMMAND" "${ARGS[@]}"
```

Because the env file is sourced into the running script, anything it sets
becomes part of `cce`'s environment, and `exec` then hands that
environment over to the target command. Three consequences worth knowing:

- The target command sees the *exported* variables from the env file.
  Plain (non-`export`) assignments are still visible to the script but are
  not inherited by `exec`'d children, so production env files should use
  `export`.
- `cce` is no longer in the process tree once `exec` succeeds — there is
  no parent process consuming memory, and signals go straight to the
  target command.
- The exit status is the target command's. There is no wrapping, no
  remapping, no special handling of 127.
