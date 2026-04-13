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
- **Strict mode**: `set -e` is on. New code must keep working under it —
  guard expected failures with `|| true` or explicit `if` blocks.
- **No external deps** beyond the coreutils already assumed (`grep`,
  `sort`, `printf`, `sed`, `basename`, `command`). `fzf` is optional and
  detected at runtime.
- **Unix only**. The script ends with `source` + `exec`; do not add
  Windows fallbacks.
- **Security**: environment names are validated against `/`, `\`, `..`,
  and empty before any filesystem lookup. Keep that check in front of
  every new code path that uses a user-supplied name. Never log the
  contents of an env file — only its path.
- **Version**: hard-coded as `VERSION` near the top of `cce.sh`. Bump it
  in the same commit as any user-visible change.

## Working on the script

```bash
# Run locally without installing
./cce.sh --help
./cce.sh <name> -- <args...>

# Lint (if installed)
shellcheck cce.sh
```

There is no automated test suite. Verify changes by running the affected
mode against a real `~/.config/cce/` directory — at minimum: `--help`,
`--version`, the list/picker path, and one `cce <name>` invocation that
hits `exec`.

## Commit style

Imperative, focused, atomic. Match the existing log
(`git log --oneline`). Touch `cce.sh` and `README.md` (or `DESIGN.md`)
together when behavior or internals change; doc-only and script-only
commits are both fine when scope is genuinely separate.
