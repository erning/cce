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
cce glm -- --help                      # pass flags through to claude
cce -c claude-code glm -- "hello"      # use a different executable
```

## Usage

```
Usage: cce [OPTIONS] [NAME] [-- ARGS...]
```

### Arguments

| Argument  | Description                                                      |
|-----------|------------------------------------------------------------------|
| `NAME`    | Environment name. Resolves to `<config-dir>/<NAME>.env`.         |
| `ARGS...` | Arguments forwarded to the target command (after `--`).          |

`NAME` is the first non-option positional argument. Anything after a
literal `--` is collected verbatim and passed to the command. Tokens that
appear after `NAME` *without* a preceding `--` are also collected as
`ARGS`, but use `--` whenever an arg starts with `-` so it is not mistaken
for a `cce` flag.

If `NAME` itself starts with `-` it is not treated as an environment name —
`cce` falls back to listing / interactive selection. Always put flags
*before* the environment name.

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
2. **Run a named environment** — `NAME` was provided and does not start
   with `-`. Sources the environment file and `exec`s the command.
3. **List or pick** — no `NAME` was provided (or it started with `-`). If
   `fzf` is available *and* at least one environment exists, an
   interactive picker is shown; otherwise the available environments are
   listed.

### Examples

```bash
# Interactive picker (or list if fzf is not installed)
cce

# Run the default `claude` command with the glm environment
cce glm

# Pass arguments through to claude — note the `--` separator
cce glm -- --model claude-3-opus "explain this codebase"

# Use a custom executable (claude-code) with the kimi-k2 environment
cce --command claude-code kimi-k2 -- --version

# Same, using the short flag
cce -c claude-code kimi-k2
```

### Exit codes

| Code  | Meaning                                                            |
|-------|--------------------------------------------------------------------|
| `0`   | Success: command ran (and the command itself exited 0), or help/version was printed. |
| `1`   | An error originating from `cce` itself: invalid name, missing file, bad arguments. |
| other | Forwarded from the target command after `exec`.                    |

## Configuration

`cce` reads environment configurations from per-environment files in a
single directory. The directory location follows the
[XDG Base Directory specification](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html).

### Config directory

The directory is resolved at startup as follows:

1. If `XDG_CONFIG_HOME` is set **and** non-empty (after stripping spaces),
   use `$XDG_CONFIG_HOME/cce/`.
2. Otherwise use `$HOME/.config/cce/`.

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
| `ANTHROPIC_AUTH_TOKEN` | Yes      | Authentication token for the API.        |
| `ANTHROPIC_BASE_URL`   | No       | API endpoint URL. Provider-default if absent. |

`cce` itself does not interpret these variables — it just sources the
file and `exec`s the target command. The target command (usually
`claude`) is what actually reads them. You are free to export anything
else the command understands.

### Security notes

- Environment files contain API tokens. Restrict their permissions
  (`chmod 600`) and keep the directory out of any backups or sync targets
  that you do not control.
- `cce` never logs the contents of environment files; only their paths
  appear in error messages.
- Environment **names** passed on the command line are validated to
  prevent path traversal (rejected if they contain `/`, `\`, `..`, or are
  empty). Implementation details in [DESIGN.md](DESIGN.md#environment-name-validation).

## How it works

See [DESIGN.md](DESIGN.md) for the execution pipeline, the `source` +
`exec` model, fzf integration, and the rationale for keeping the
implementation as a single Bash script.

## License

MIT.
