# CCE — Claude Code Environment Manager

A small Bash script that runs `claude` — or any other command — with
environment variables loaded from a named `.env` file under
`~/.config/cce/`. Use it to keep multiple Claude API providers and
accounts side-by-side and switch between them without touching your shell
profile.

## Install

```bash
curl -o cce https://raw.githubusercontent.com/erning/cce/refs/heads/master/cce.sh
chmod +x cce
mv cce ~/.local/bin/   # or anywhere on $PATH
```

## Requirements

- Bash 3.2+ (works on stock macOS `/bin/bash`).
- A Unix-like OS. The script ends with `source` + `exec` and is not
  intended for Windows.
- The target command (default `claude`) on `PATH`.
- Optional: [`fzf`](https://github.com/junegunn/fzf) for interactive
  environment selection when `cce` is invoked with no name.

## Quick start

```bash
# 1. Create the config directory
mkdir -p ~/.config/cce

# 2. Write an environment file (chmod recommended — it contains a token)
cat > ~/.config/cce/glm.env <<'EOF'
export ANTHROPIC_BASE_URL="https://open.bigmodel.cn/api/anthropic"
export ANTHROPIC_AUTH_TOKEN="your_token_here"
EOF
chmod 600 ~/.config/cce/glm.env

# 3. Use it
cce              # interactive picker (fzf) or list
cce glm          # run claude with the glm environment
cce glm --help                  # everything after NAME is forwarded
cce -c claude-code glm "hello"  # use a different executable
```

## Usage

```
Usage: cce [OPTIONS] NAME [ARGS...]
       cce [OPTIONS]                # list / interactive picker
```

### Arguments

| Argument  | Description                                                      |
|-----------|------------------------------------------------------------------|
| `NAME`    | Environment name. Resolves to `<config-dir>/<NAME>.env`.         |
| `ARGS...` | Arguments forwarded verbatim to the target command.              |

`NAME` is the first non-option positional argument. **Once `NAME` has been
consumed, every remaining token is forwarded to the target command — `cce`
performs no further flag parsing.** That means `cce glm --help` runs
`claude --help` against the `glm` environment; you do *not* need a `--`
separator. An optional leading `--` after `NAME` is consumed for
backwards compatibility with older invocations.

`cce` flags must therefore appear **before** `NAME`. Unknown options
before `NAME` (or any `-X` that is not one of the flags below) error out
rather than being silently treated as an environment name.

### Options

| Flag                    | Description                                          |
|-------------------------|------------------------------------------------------|
| `-c`, `--command <CMD>` | Executable to run. Default: `claude`.                |
| `--version`             | Print the script version and exit.                   |
| `-h`, `--help`          | Print help and exit.                                 |

### Modes of operation

`cce` has three mutually exclusive modes, selected from the parsed
arguments:

1. **Help / version** — `--help` or `--version`. Prints and exits 0.
2. **Run a named environment** — `NAME` was provided. Sources the
   environment file and `exec`s the command.
3. **List or pick** — no `NAME` was provided. If `fzf` is available, at
   least one environment exists, **and stdin is a terminal**, an
   interactive picker is shown. Otherwise (no fzf, no environments, or a
   non-interactive stdin such as a pipe / CI run) the available
   environments are listed.

### Examples

```bash
# Interactive picker (or list if fzf is not installed)
cce

# Run the default `claude` command with the glm environment
cce glm

# Pass arguments through to claude — no `--` needed
cce glm --model claude-3-opus "explain this codebase"

# `--` after NAME still works for muscle-memory back-compat
cce glm -- --version

# Use a custom executable (claude-code) with the kimi-k2 environment
cce --command claude-code kimi-k2 --version

# Same, using the short flag
cce -c claude-code kimi-k2
```

### Exit codes

| Code  | Meaning                                                            |
|-------|--------------------------------------------------------------------|
| `0`   | Success: command ran (and the command itself exited 0), or help/version was printed. |
| `1`   | An error originating from `cce` itself: invalid name, missing file, bad arguments, source failure. |
| `127` | The target command (`-c <CMD>`) was not found on `PATH` after the env file was sourced. |
| other | Forwarded from the target command after `exec`.                    |

## Configuration

`cce` reads environment configurations from per-environment files in a
single directory. The directory location follows the
[XDG Base Directory specification](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html).

### Config directory

The directory is resolved at startup as follows:

1. If `XDG_CONFIG_HOME` is set **and** is an absolute path (per the XDG
   spec), use `$XDG_CONFIG_HOME/cce/`. A non-absolute value is rejected
   with a warning and `cce` falls back to step 2.
2. Otherwise use `$HOME/.config/cce/`.

If neither yields a usable directory (`HOME` unset and `XDG_CONFIG_HOME`
not absolute), `cce` errors out at startup.

`cce` does not create the directory for you. If it does not exist, listing
reports it as empty rather than failing.

### File layout

Each environment is a single file inside the config directory:

```
~/.config/cce/
├── glm.env
├── kimi-k2.env
└── minimax-m2.env
```

Rules:

- The file extension **must** be `.env`. Files with any other extension
  are ignored by discovery.
- The environment name is the filename with the `.env` suffix stripped,
  so `kimi-k2.env` is the environment `kimi-k2`.
- Names must match `^[A-Za-z0-9_][A-Za-z0-9._-]*$` — they must start
  with a letter, digit, or underscore, and may contain only letters,
  digits, dots, hyphens, and underscores. Files with names outside this
  set are skipped during discovery with a warning, and `cce <bad-name>`
  on the command line errors out.
- Discovery is non-recursive — only the top level of the config directory
  is scanned.
- Listing is sorted alphabetically by name.

### File format

Environment files are sourced by Bash at runtime, so they may use the
full Bash syntax:

```bash
# ~/.config/cce/glm.env
export ANTHROPIC_BASE_URL="https://open.bigmodel.cn/api/anthropic"
export ANTHROPIC_AUTH_TOKEN="your_token_here"
```

Use `export KEY=VALUE` for anything you want the target command to see —
plain `KEY=VALUE` assignments are visible inside the script while it is
sourcing the file, but are not inherited by the `exec`'d child process.

You can use anything Bash accepts: variable expansion, command
substitution, conditionals, etc. For example, reading a token from a
file:

```bash
export ANTHROPIC_BASE_URL="https://api.example.com"
export ANTHROPIC_AUTH_TOKEN="$(cat ~/.secrets/anthropic-token)"
```

Comments start with `#` and are ignored by Bash.

### Variables consumed by Claude

| Variable               | Required | Purpose                                  |
|------------------------|----------|------------------------------------------|
| `ANTHROPIC_API_KEY`    | Yes\*    | Sent by `claude` as the `x-api-key` header. Use this for Anthropic first-party API keys. |
| `ANTHROPIC_AUTH_TOKEN` | Yes\*    | Sent by `claude` as `Authorization: Bearer <value>`. Use this for OAuth tokens and for custom providers that speak the Anthropic schema with bearer auth. |
| `ANTHROPIC_BASE_URL`   | No       | API endpoint URL. Provider default if absent. |

\* At least one auth variable must be set; which one depends on your
provider. Anthropic's first-party API takes `ANTHROPIC_API_KEY`; OAuth
flows and most alternative Claude-compatible providers take
`ANTHROPIC_AUTH_TOKEN`. The two variables are **not aliases** — they
populate different HTTP headers, so pick the one your provider expects.

`cce` itself does not interpret these variables — it just sources the
file and `exec`s the target command. The target command (usually
`claude`) is what actually reads them. You are free to export anything
else the command understands.

### Security notes

- Environment files contain API tokens. Restrict their permissions
  (`chmod 600`) and keep the directory out of any backups or sync targets
  that you do not control. `cce` prints a best-effort warning at runtime
  if the env file is group- or world-writable.
- `cce` never logs the contents of environment files; only their paths
  appear in error messages.
- Environment **names** passed on the command line are validated against
  the same whitelist as discovery (`^[A-Za-z0-9_][A-Za-z0-9._-]*$`),
  which rejects empty names, leading dots, leading dashes, path
  separators, `..`, whitespace, and control characters. Implementation
  details in [DESIGN.md](DESIGN.md#environment-name-validation).
- **Environment files are not sandboxed.** An env file is ordinary Bash,
  sourced into the running shell with full language access — command
  substitution, file I/O, arbitrary function definitions, `PATH`
  changes, the lot. `cce` uses `readonly` internal variables and
  `builtin` prefixes on the post-source control-flow calls
  (`exec`/`command`/`trap`/`exit`) to block the most common accidental
  footguns, but these are **footgun guards, not a security boundary**.
  Treat an env file the same way you would treat a `.bashrc` snippet:
  only source files you wrote or would be willing to run by hand.
  Full rationale and non-goals in
  [DESIGN.md](DESIGN.md#non-goals).

## How it works

See [DESIGN.md](DESIGN.md) for the execution pipeline, the `source` +
`exec` model, fzf integration, and the rationale for keeping the
implementation as a single Bash script.

## License

MIT.
