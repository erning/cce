# Agent Guide — CCE

CCE is a single Bash script (`cce.sh`) that runs `claude` — or any other
command — with environment variables loaded from a named `.env` file under
`~/.config/cce/`. There is no compiled component, no build step, no
package manager. Edit the script in place.

## Source of truth

- `cce.sh` — the entire implementation. If behavior and docs disagree,
  the script wins; update the docs.
- `README.md` — user-facing reference: install, CLI, configuration,
  examples. Update it in the same change as any script edit that changes
  observable behavior.
- `DESIGN.md` — internals: execution pipeline, name validation,
  discovery, fzf flow, source+exec model. Read it before changing how the
  script runs commands or handles environment files.

## What was removed

This project previously had a Rust reimplementation under `src/`, an
OpenSpec workflow under `openspec/`, and a multi-file `docs/` tree. All
three are gone. Do not propose reintroducing them, do not write Rust, do
not create OpenSpec proposals or specs, and do not recreate `docs/` —
user-facing content lives in `README.md`, internals live in `DESIGN.md`.
If you find a stray reference to `cargo`, `Cargo.toml`, `src/*.rs`,
`openspec/`, `docs/`, or a spec-id like `CLI-001` / `CONF-002`, treat it
as stale and remove it.

## Conventions

- **Language**: Bash 3.2+. The script must run on stock macOS `/bin/bash`
  (3.2.57). That rules out `local -n` namerefs (Bash 4.3+),
  `mapfile`/`readarray` (Bash 4+), associative arrays, and `${var,,}`
  case conversion. Process substitution `< <(...)` and arrays are fine.
  Empty arrays under `set -u` need the workaround
  `${arr[@]+"${arr[@]}"}` — Bash 3.2 errors on bare `"${arr[@]}"` when
  the array is empty.
- **Strict mode**: `set -euo pipefail` is on. New code must keep working
  under all three — guard expected failures with `|| true` or explicit
  `if` blocks, and use `${VAR:-default}` for any variable that may be
  unset.
- **Env name regex**: the only allowed env names are
  `^[A-Za-z0-9_][A-Za-z0-9._-]*$`. The regex lives in the `NAME_RE`
  constant near the top of `cce.sh` and is enforced both in
  `validate_env_name` (CLI input) and in `get_env_names` (discovery).
  Any code path that accepts a user-supplied name must call
  `validate_env_name` before touching the filesystem.
- **No external deps** beyond coreutils that ship with both BSD and
  GNU systems (`printf`, `sort`, `basename`, `sed`, `stat`, `command`).
  `fzf` is optional and detected at runtime. `stat` flags differ
  between BSD and GNU — use the `get_file_mode` helper, which tries
  both forms and validates the output is purely numeric.
- **Unix only**. The script ends with `source` + `exec`; do not add
  Windows fallbacks.
- **Security**: never log the contents of an env file — only its path.
  The script warns if an env file is group/world-writable but does not
  refuse to load it.
- **Source semantics**: load env files with `trap ... ERR` + bare `.`,
  not `if ! . file; then`. The `if` form disables `set -e` inside the
  sourced file and silently masks intermediate failures. There is also
  a `bash -n` syntax pre-check for parse errors, which the `ERR` trap
  cannot catch (no command runs on a parse failure).
- **Version**: hard-coded as `VERSION` near the top of `cce.sh`. Bump
  it in the same commit as any user-visible change.

## Working on the script

```bash
# Run locally without installing
./cce.sh --help
./cce.sh <name> [args...]
```

### Lint baseline

Three checks, all expected to be clean before any commit that touches
`cce.sh`:

```bash
bash -n cce.sh                  # syntax check on macOS-stock /bin/bash
shellcheck -s bash cce.sh       # static analysis (bash dialect)
shfmt -d -i 2 -ci cce.sh        # formatting: 2-space indent, indented case
```

`shfmt -w -i 2 -ci cce.sh` rewrites the file in place if formatting drifts.
There is no CI that runs these — they are a manual baseline.

### Manual verification

There is no automated test suite. Verify changes by running the affected
mode against a real `~/.config/cce/` directory — at minimum: `--help`,
`--version`, the list/picker path, an unknown-flag rejection, and one
`cce <name>` invocation that hits `exec`. For source/exec changes, also
test with an env file that contains a deliberately-failing command and
one with a deliberate parse error to confirm both error paths print a
clear `cce` context line.

## Commit style

Imperative, focused, atomic. Match the existing log
(`git log --oneline`). Touch `cce.sh` and `README.md` (or `DESIGN.md`)
together when behavior or internals change; doc-only and script-only
commits are both fine when scope is genuinely separate.
