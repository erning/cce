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
  the Bash 3.2-compatible workaround `${_CCE_ARGS[@]+"${_CCE_ARGS[@]}"}`, which
  expands to nothing when `_CCE_ARGS` is empty without tripping "unbound
  variable".
- `-o pipefail` propagates a non-zero status out of any pipeline, so a
  silent failure mid-pipe (e.g. in the fzf path) is observable.

## Execution pipeline

A run of `cce` proceeds through these stages:

1. **Argument parsing.** A hand-written `while`/`case` loop walks `$@`.
   The first non-option positional becomes `_CCE_ENV_NAME`; **once `_CCE_ENV_NAME`
   is consumed, every remaining token is appended to `_CCE_ARGS` verbatim and
   parsing stops.** That removes the old footgun where `cce glm --help`
   would parse `--help` as a `cce` flag instead of forwarding it. An
   unknown `-X` token before `_CCE_ENV_NAME` is a hard error rather than
   silently being treated as a name. `--command` requires a non-empty
   value at parse time.
2. **Config directory resolution.** Per the XDG spec, `XDG_CONFIG_HOME`
   must be an absolute path; if it is, `_CCE_ENV_DIR=$XDG_CONFIG_HOME/cce`.
   Anything else (unset, empty, relative) falls back to
   `$HOME/.config/cce`, with a warning if a relative `XDG_CONFIG_HOME`
   was rejected. If neither yields a usable path (`HOME` unset and no
   absolute `XDG_CONFIG_HOME`), the script errors out.
3. **Mode dispatch.** In order: version → help → list/picker (when
   `_CCE_ENV_NAME` is empty) → run.
4. **Environment name validation.** Before any file lookup that uses
   `_CCE_ENV_NAME`, the name is checked against the whitelist (see below).
5. **Permission warning.** A best-effort `stat` lookup warns if the env
   *file* is group- or world-writable. The config *directory* is
   intentionally not checked — see "Non-goals" below.
6. **Syntax pre-check.** `bash -n "$_CCE_ENV_FILE"` validates the env file
   parses cleanly, with a `cce`-context error if it does not. Parse
   errors do not trigger the runtime `ERR` trap below (no command runs),
   so we catch them here.
7. **Source.** The env file is sourced into the current shell. An `ERR`
   trap is armed (via `builtin trap`) so that a runtime command failure
   inside the env file prints a context line naming the file. The trap
   is cleared immediately after the source returns, also via `builtin
   trap` — see "Why `builtin` prefixes the post-source call sites" for
   why the prefix matters.
8. **Command sanity check.** `builtin command -v "$_CCE_COMMAND"` runs
   *after* sourcing — the env file is allowed to modify `PATH`. If the
   command is not on `PATH` we `builtin exit 127` with a clear message
   instead of letting `exec` print bash's own error. The `exit` call
   uses the `builtin` prefix for the same reason as the other
   post-source builtin calls.
9. **Exec.** `builtin exec "$_CCE_COMMAND" ${_CCE_ARGS[@]+"${_CCE_ARGS[@]}"}`
   replaces the `cce` process with the target command.

## Environment name validation

Names are matched against `^[A-Za-z0-9_][A-Za-z0-9._-]*$`:

- First char: letter, digit, or underscore — **no leading dot or dash**.
- Subsequent chars: letter, digit, dot, dash, or underscore.
- One or more characters total (empty is rejected by the first-char
  rule).

This whitelist is enforced in two places:

1. `validate_env_name "$_CCE_ENV_NAME"` rejects user-supplied names from the
   command line. An invocation like `cce ../../etc/passwd` errors out
   before any filesystem lookup, so an env file path can never escape
   `_CCE_ENV_DIR`.
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

- If `_CCE_ENV_DIR` does not exist, the function prints nothing (not an
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

When `_CCE_ENV_NAME` is empty, `cce` checks three conditions before
showing fzf: `fzf` is on `PATH`, at least one valid environment exists,
**and stdin is a terminal** (`[[ -t 0 ]]`). If any of those is false,
the list path is taken instead:

```bash
if [[ ${#env_names[@]} -gt 0 ]] \
    && command -v fzf >/dev/null 2>&1 \
    && [[ -t 0 ]]; then
  set +e
  selected=$(printf '%s\n' "${env_names[@]}" | fzf)
  fzf_status=$?
  set -e

  case $fzf_status in
    0)     _CCE_ENV_NAME="$selected" ;;
    1|130) list_environments "${env_names[@]}"; exit 0 ;;  # no-match or SIGINT
    *)     echo "Error: fzf exited with status $fzf_status" >&2; exit 1 ;;
  esac
else
  list_environments "${env_names[@]}"
  exit 0
fi
```

The TTY guard matters for piped or CI invocations (`echo "" | cce`,
`cce </dev/null`, cron, git hooks). Without it, `fzf` would either
hang on `/dev/tty` or exit noisily on systems with no controlling
terminal — neither is a good default for a script that is usefully
embeddable in pipelines. With the guard, those invocations cleanly
fall through to the plain listing.

The previous version also treated *every* fzf failure as a user
cancellation, which silently masked terminal-not-a-TTY, broken-pipe,
and crash cases. The explicit `case` lets unexpected statuses surface
as an error rather than dropping the user into a confusing "no
environments selected" path.

### fzf exit code 1 vs. 130

Both `1` (no match selected) and `130` (SIGINT — Esc / Ctrl-C) fall
through to the same `list_environments` call. Differentiating them
(e.g. silent exit on 130, list on 1) was considered and rejected:
seeing the list after cancelling is harmless and occasionally useful
(it reminds you what's available), while the extra branch adds code
for a purely subjective win. If this ever becomes annoying, it is a
one-line change; there is no structural obstacle to splitting the
cases later.

If a name is selected, control falls through to the normal run path —
the `--command` and any trailing `_CCE_ARGS` collected during parsing are
reused verbatim.

## Source and exec

The run path is deliberately small but defensive:

```bash
builtin trap 'echo "Error: failed while loading environment file: $_CCE_ENV_FILE" >&2; builtin exit 1' ERR
. "$_CCE_ENV_FILE"
builtin trap - ERR
set -euo pipefail
```

### Why `trap ERR` and not `if ! . file`

A natural-looking pattern is:

```bash
if ! . "$_CCE_ENV_FILE"; then
  echo "Error: ..." >&2
  exit 1
fi
```

**Do not do this.** Bash treats commands inside an `if` test as part of
a tested expression, and **`set -e` is disabled for them** — including
inside the file being sourced. A `false` (or any failing command) in
the env file would *not* abort sourcing; the script would happily
continue with a half-loaded environment. The `ERR` trap pattern keeps
error reporting wired up regardless of the tested-context rules.

### Why the ERR trap actively exits

The trap body ends with `builtin exit 1` rather than relying on
`set -e` to abort for it. Both are documented ways to fail a shell
script, but they differ in one crucial detail: `set -e` can be
toggled off from inside the sourced file. An env file that writes
`set +e` — intentionally or by mistake — leaves cce's shell with
error-on-failure disabled, so even though the ERR trap still fires
on `false`, nothing actually aborts the script. The reviewer's
reproduction was:

```bash
# probe.env
set +e
false
export X=after
```

Without the active `exit`, cce would print the "failed while loading"
context line, then calmly continue past the source, past the
`command -v` check, and `exec` the target command with `X=after` in
its environment. That is the opposite of a safe partial-load.

With `builtin exit 1` in the trap body, the `false` still triggers
the trap; the trap prints its context and exits 1; cce aborts before
exec. The `builtin` prefix on `exit` matters for the same reason it
matters on `exec`/`trap`/`command` — see "Why `builtin` prefixes the
post-source call sites" below.

### Restoring strict mode after source

The env file runs inside cce's own shell, so any `set +e` / `set +u`
/ `set +o pipefail` it performs persists in cce's state after the
source returns. To keep the post-source path (command lookup and
exec) running under the same guarantees as the top of the script,
cce re-enables all three immediately after source:

```bash
. "$_CCE_ENV_FILE"
builtin trap - ERR
set -euo pipefail
```

This is cheap and idempotent in the success path, and it prevents a
`set +u` in the env file from silently turning subsequent unset-var
references into empty strings inside cce.

Note that re-enabling strict mode does **not** affect the target
command. Shell options are shell-internal state; `exec` replaces the
process and the target starts with whatever options its own shell (if
any) chooses. The restore only hardens the handful of lines between
the source return and the final `exec`.

### Failure modes the ERR trap does NOT catch

A handful of bash failures exit the sourced file before any command
actually runs, which means the `ERR` trap never fires and users see
bash's raw error with no cce context:

- **Parse errors.** E.g. an unclosed `$(`. Bash prints
  `env_file: line N: syntax error ...` and never runs a command.
  We catch this class upfront with a `bash -n "$_CCE_ENV_FILE"`
  syntax pre-check so the error can be indented under a clear `cce`
  context line.
- **Parameter-expansion errors.** E.g. `echo ${MISSING?missing var}`,
  or Bash 4-only expansions like `${FOO,,}` hitting Bash 3.2. Bash
  aborts the sourced file with its own error (`env_file: line N:
  MISSING: missing var`) and no ERR trap runs. Unlike parse errors,
  we cannot pre-check these: expansion happens at runtime and depends
  on the surrounding state. `cce` still exits non-zero (sourcing
  returned non-zero, and `set -e` is active in the caller), it just
  does so without the friendly "failed while loading environment
  file" prefix.

If cce is ever expected to wrap expansion errors with its own
context, the only workable approach is to source inside a subshell
and parse its output — significantly more invasive than the current
design warrants for a diagnostic-message improvement.

### Env files can modify traps and shell options during source

Because the env file is ordinary bash, it can install its own ERR
trap or disarm cce's:

```bash
trap - ERR              # removes cce's trap entirely
trap '...' ERR          # replaces it
trap '...; return 0' ERR  # fires, returns early from source with status 0
```

cce's defence is partial and best-effort:

1. The post-source `builtin trap - ERR` guarantees no carried-over
   trap survives into the command-lookup / exec path — even if the
   env file installed one, it is removed before cce uses bash again.
2. `set -euo pipefail` is re-enabled post-source, so any `set +e` /
   `set +u` / `set +o pipefail` from inside the source cannot leak
   into cce's own code.
3. However, cce cannot prevent the env file from owning shell state
   *during* the source itself. A deliberately written
   `trap '…; return 0' ERR` will cause the source to return 0 on
   failure, and cce will dutifully proceed to exec the target — the
   env file has explicitly opted out of the abort contract, and
   that is the env file's prerogative.

See "Non-goals" at the bottom of this file for the full statement of
the trust boundary.

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
- A bare `return` inside the env file **ends the source early** (that
  is standard Bash behavior for sourced files — `return` is only valid
  in a sourced context or a function body). Any assignments after the
  `return` are skipped, the `ERR` trap does not fire, and cce proceeds
  to the `command -v` + `exec` steps with whatever the env file had
  set up to that point. This is occasionally useful (e.g. gate an env
  file on a hostname check: `[[ $(hostname) == prod-* ]] || return 0`)
  but it is also a foot-gun if you use `return` accidentally. If you
  want the target command to abort instead of run, use `exit 1`, not
  `return`.

### Why internal variables are prefixed `_CCE_`

Every variable that `cce` relies on across the `source` boundary
(`_CCE_COMMAND`, `_CCE_ENV_NAME`, `_CCE_ENV_DIR`, `_CCE_ENV_FILE`,
`_CCE_ARGS`) carries the `_CCE_` prefix and is marked `readonly`
immediately before the env file is sourced. Two reasons, in order:

1. **Collision avoidance.** Without the prefix, a user env file could
   legitimately want to set `ENV_NAME=production` or `COMMAND=...` for
   their own reasons — those are common names. The prefix keeps cce's
   internals out of the namespace a user is likely to touch.
2. **Defense in depth.** Even with unique names, a typo or a deliberate
   assignment to `_CCE_COMMAND` inside an env file would silently
   change what `exec` runs after sourcing returns. `readonly` turns
   that into a bash-level error: the sourced file aborts with a
   `readonly variable` message instead of proceeding with a hijacked
   `_CCE_COMMAND`. Note that bash's readonly-assignment error exits via
   a different path than a failed command, so the `ERR` trap's context
   line is not guaranteed to fire on this specific failure — bash's
   own error message is already unambiguous.

### Why `builtin` prefixes the post-source call sites

Readonly protects cce's *variables* but not its *builtin calls*. An env
file is ordinary bash, so it can just as easily define a shell function
with the same name as a builtin:

```bash
# evil.env
export ANTHROPIC_AUTH_TOKEN=fake
exec() { echo "hijacked $*"; }
```

That function definition survives the `source` call. Without a guard,
`cce.sh`'s final `exec "$_CCE_COMMAND" …` resolves to the env file's
`exec` function — the target command never replaces the cce process,
and the user sees `hijacked /bin/echo …` instead of their command
actually running. The same attack works against `command -v` (the
post-source PATH check), `trap - ERR` (the loading-trap cleanup), and
`exit 127` (the command-not-found branch, which would otherwise let
control fall through into the final exec and produce a jumbled
double-error).

The fix is the `builtin` prefix. `builtin NAME` tells bash to look
`NAME` up in the builtin table and skip function lookup entirely, so
the actual `exec` / `command` / `trap` / `exit` runs regardless of
what the env file defined:

```bash
builtin trap 'echo "Error: ..." >&2' ERR
. "$_CCE_ENV_FILE"
builtin trap - ERR

if ! builtin command -v "$_CCE_COMMAND" >/dev/null 2>&1; then
  echo "Error: command not found: $_CCE_COMMAND" >&2
  builtin exit 127
fi

builtin exec "$_CCE_COMMAND" ${_CCE_ARGS[@]+"${_CCE_ARGS[@]}"}
```

This is not a full sandbox — it is a footgun guard. An env file that
defines a function literally named `builtin` would defeat the prefix,
and at that point the env file is actively adversarial rather than
accidentally shadowing a common name. That case is intentionally out
of scope: the env file is user-supplied bash code, so the trust
boundary is "don't run env files from untrusted sources", not "cce
sandboxes arbitrary bash".

## Non-goals

A few things cce deliberately does **not** do, so future contributors
don't re-add them thinking the omission was an oversight.

- **Config directory permission checks.** `cce` warns only on a
  group/world-writable *env file*, not on the directory that holds it.
  The trust model is already "the env file is user-supplied bash code":
  if an attacker can write files inside `~/.config/cce/`, they can also
  ship one with arbitrary content and the per-file check would catch
  that at load time. Warning on the directory as well would be noisy
  on shared XDG setups without meaningfully raising the security bar,
  so the check stays file-only.
- **Refusing to load on permission warnings.** The warning is
  advisory. `cce` still sources a world-writable env file and still
  execs the target command. This matches the "warn, don't block"
  convention of `ssh` on `~/.ssh/config` mode bits.
- **fzf file-owner checks, symlink following rules, parent-chain
  audits, etc.** All of the above are beyond the intended footprint
  of a ~300-line shell script whose sole job is to splice a `.env`
  file in front of another command.
- **Full sandboxing of env files.** As noted in the `builtin`
  subsection above, an env file is arbitrary bash. The `builtin`
  prefix, `readonly` internal variables, and the syntax pre-check
  together block the most common accidental footguns, but they are
  not a security boundary against a deliberately hostile env file.
  Review files you source, the same way you review `.envrc` or
  `.bashrc` snippets you paste from the internet.
- **Preventing in-source trap / shell-option subversion.** During the
  `source` call itself the env file owns shell state: it can install
  its own ERR trap, disarm cce's, toggle `set -e`, redirect file
  descriptors, and so on. cce mitigates the spill-over by actively
  exiting from the ERR trap body, by disarming the trap again
  post-source, and by restoring `set -euo pipefail` once the source
  returns — but it cannot police what runs during the source itself.
  An env file that explicitly installs `trap '…; return 0' ERR` to
  swallow failures will be respected, and cce will proceed to exec
  the target as if the source succeeded. Same trust boundary as
  above: if you source it, you trust it.
- **Wrapping bash expansion / substitution errors with cce context.**
  Failures like `${MISSING?err}` or a Bash 4-only expansion hitting
  Bash 3.2 abort the sourced file before any command runs, so the
  runtime ERR trap cannot fire. Bash's own error surfaces without a
  cce prefix. Catching these would require sourcing inside a
  subshell and parsing its output — significantly more invasive than
  the diagnostic improvement warrants. cce still exits non-zero on
  these failures; only the context line is missing.
