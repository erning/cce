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

The trade-off is that `cce` is Unix-only. Porting it to Windows would
mean parsing env files manually and spawning a subprocess instead of
`source` + `exec` — a meaningful rewrite, not a flag.

## Strict mode

The script runs under `set -euo pipefail`:

- `-e` exits on any uncaught command failure.
- `-u` treats unset variables as errors. Most call sites use
  `${VAR:-default}` to be explicit; the array passthrough at `exec` uses
  the Bash 3.2-compatible workaround `${ARGS[@]+"${ARGS[@]}"}`, which
  expands to nothing when `ARGS` is empty without tripping "unbound
  variable".
- `-o pipefail` propagates a non-zero status out of any pipeline, so a
  silent failure mid-pipe (e.g. in the fzf path) is observable.

## Execution pipeline

A run of `cce` proceeds through these stages:

1. **Argument parsing.** A hand-written `while`/`case` loop walks `$@`.
   The first non-option positional becomes `ENV_NAME`; **once `ENV_NAME`
   is consumed, every remaining token is appended to `ARGS` verbatim and
   parsing stops.** That removes the old footgun where `cce glm --help`
   would parse `--help` as a `cce` flag instead of forwarding it. An
   unknown `-X` token before `ENV_NAME` is a hard error rather than
   silently being treated as a name. `--command` requires a non-empty
   value at parse time.
2. **Config directory resolution.** Per the XDG spec, `XDG_CONFIG_HOME`
   must be an absolute path; if it is, `ENV_DIR=$XDG_CONFIG_HOME/cce`.
   Anything else (unset, empty, relative) falls back to
   `$HOME/.config/cce`, with a warning if a relative `XDG_CONFIG_HOME`
   was rejected. If neither yields a usable path (`HOME` unset and no
   absolute `XDG_CONFIG_HOME`), the script errors out.
3. **Mode dispatch.** In order: version → help → list/picker (when
   `ENV_NAME` is empty) → run.
4. **Environment name validation.** Before any file lookup that uses
   `ENV_NAME`, the name is checked against the whitelist (see below).
5. **Permission warning.** A best-effort `stat` lookup warns if the env
   file is group- or world-writable.
6. **Syntax pre-check.** `bash -n "$ENV_FILE"` validates the env file
   parses cleanly, with a `cce`-context error if it does not. Parse
   errors do not trigger the runtime `ERR` trap below (no command runs),
   so we catch them here.
7. **Source.** The env file is sourced into the current shell. An `ERR`
   trap is armed so that a runtime command failure inside the env file
   prints a context line naming the file. The trap is cleared
   immediately after the source returns.
8. **Command sanity check.** `command -v "$COMMAND"` runs *after*
   sourcing — the env file is allowed to modify `PATH`. If the command
   is not on `PATH` we exit 127 with a clear message instead of letting
   `exec` print bash's own error.
9. **Exec.** `exec "$COMMAND" ${ARGS[@]+"${ARGS[@]}"}` replaces the
   `cce` process with the target command.

## Environment name validation

Names are matched against `^[A-Za-z0-9_][A-Za-z0-9._-]*$`:

- First char: letter, digit, or underscore — **no leading dot or dash**.
- Subsequent chars: letter, digit, dot, dash, or underscore.
- One or more characters total (empty is rejected by the first-char
  rule).

This whitelist is enforced in two places:

1. `validate_env_name "$ENV_NAME"` rejects user-supplied names from the
   command line. An invocation like `cce ../../etc/passwd` errors out
   before any filesystem lookup, so an env file path can never escape
   `ENV_DIR`.
2. `get_env_names` filters discovered files by the same regex and warns
   for each skipped file. The picker / list never offers something the
   runner would refuse.

The whitelist is intentionally narrower than "no path traversal": it
also rejects whitespace, control characters, and leading dashes (which
would otherwise be confused with CLI options). It matches the set of
characters that survive `cce <name>` round-trips cleanly across shells,
filesystems, and fzf, which is the property we actually want.

## Environment discovery

`get_env_names` prints sorted, valid environment names to stdout (one
per line). Callers slurp the output via `while IFS= read -r`:

- If `ENV_DIR` does not exist, the function prints nothing (not an
  error).
- Each `*.env` file's basename is checked against `NAME_RE`. Matches go
  into a local array; non-matches print a `Warning: skipping invalid env
  file name: '...'` to stderr.
- Names are sorted via `printf '%s\n' "${names[@]}" | sort`, so listing
  and the fzf picker are always alphabetical.
- The list/picker callers pass the already-collected array into
  `list_environments` as positional arguments rather than letting it
  re-run `get_env_names`. This matters because re-running would print
  the "skipping invalid name" warnings twice on the empty-name path.

Discovery is non-recursive, does not match dotfiles (Bash glob default),
and does not follow symlinks specially.

The function deliberately avoids Bash 4+ features (`local -n` namerefs,
`mapfile`/`readarray`) so the script runs unmodified on the stock macOS
`/bin/bash`, which is still 3.2.

## Interactive selection

When `ENV_NAME` is empty, `cce` checks `command -v fzf`. If `fzf` is
available *and* at least one environment exists, it pipes the sorted
names into `fzf` and inspects the exit code explicitly:

```bash
set +e
selected=$(printf '%s\n' "${env_names[@]}" | fzf)
fzf_status=$?
set -e

case $fzf_status in
  0)     ENV_NAME="$selected" ;;
  1|130) list_environments "${env_names[@]}"; exit 0 ;;  # no-match or SIGINT
  *)     echo "Error: fzf exited with status $fzf_status" >&2; exit 1 ;;
esac
```

The previous version treated *every* fzf failure as a user cancellation,
which silently masked terminal-not-a-TTY, broken-pipe, and crash cases.
The explicit case lets unexpected statuses surface as an error rather
than dropping the user into a confusing "no environments selected" path.

If a name is selected, control falls through to the normal run path —
the `--command` and any trailing `ARGS` collected during parsing are
reused verbatim.

## Source and exec

The run path is deliberately small but defensive:

```bash
trap 'echo "Error: failed while loading environment file: $ENV_FILE" >&2' ERR
. "$ENV_FILE"
trap - ERR
```

### Why `trap ERR` and not `if ! . file`

A natural-looking pattern is:

```bash
if ! . "$ENV_FILE"; then
  echo "Error: ..." >&2
  exit 1
fi
```

**Do not do this.** Bash treats commands inside an `if` test as part of
a tested expression, and **`set -e` is disabled for them** — including
inside the file being sourced. A `false` (or any failing command) in
the env file would *not* abort sourcing; the script would happily
continue with a half-loaded environment. The `ERR` trap pattern keeps
`set -e` active inside the sourced file, so partial loads always abort.

There is one failure mode the `ERR` trap does not catch: a Bash *parse*
error inside the env file, because no command actually runs. We catch
those upfront with a `bash -n "$ENV_FILE"` syntax pre-check that prints
the bad-syntax error indented under a clear `cce` context line.

### Consequences of source + exec

Because the env file is sourced into the running script, anything it
sets becomes part of `cce`'s environment, and `exec` then hands that
environment over to the target command. Three things worth knowing:

- The target command sees the *exported* variables from the env file.
  Plain (non-`export`) assignments are still visible to the script but
  are not inherited by `exec`'d children, so production env files should
  use `export`.
- `cce` is no longer in the process tree once `exec` succeeds — there
  is no parent process consuming memory, and signals go straight to the
  target command.
- The exit status is the target command's. The only exit code `cce`
  produces on its own paths is 1 for cce-level errors and 127 if the
  configured `--command` is not on `PATH` after sourcing.
