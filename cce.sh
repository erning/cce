#!/usr/bin/env bash
#
# cce — Claude Code Environment Manager
# https://github.com/erning/cce
#
# Run `cce --help` for usage. See README.md and DESIGN.md for details.

set -euo pipefail

VERSION="2.1.1"

# Names allowed for environment files (without the .env suffix).
# First char: letter, digit, or underscore. Subsequent chars may also
# contain `.` and `-`. Rejects empty, leading dot/dash, whitespace,
# control chars, path separators, and `..`.
readonly NAME_RE='^[A-Za-z0-9_][A-Za-z0-9._-]*$'

# Defaults
COMMAND="claude"
ENV_NAME=""
SHOW_HELP=false
SHOW_VERSION=false
ARGS=()

# -----------------------------------------------------------------------------
# Argument parsing
# -----------------------------------------------------------------------------
# Rule: the first positional argument is the environment name. Once it has
# been consumed, every remaining token is forwarded verbatim to the target
# command — there is no further flag parsing. An optional leading `--` after
# NAME is consumed for back-compat with older invocations.

while [[ $# -gt 0 ]]; do
  case $1 in
    -c | --command)
      if [[ $# -lt 2 ]]; then
        echo "Error: --command requires an argument" >&2
        exit 1
      fi
      if [[ -z "$2" ]]; then
        echo "Error: --command requires a non-empty argument" >&2
        exit 1
      fi
      COMMAND="$2"
      shift 2
      ;;
    -h | --help)
      SHOW_HELP=true
      shift
      ;;
    --version)
      SHOW_VERSION=true
      shift
      ;;
    --)
      # `--` before NAME has no useful meaning here. Reject it explicitly
      # rather than silently dropping into list mode.
      echo "Error: '--' must come after the environment name" >&2
      exit 1
      ;;
    -*)
      echo "Error: unknown option '$1'" >&2
      echo "Run 'cce --help' for usage." >&2
      exit 1
      ;;
    *)
      ENV_NAME="$1"
      shift
      # Optional `--` separator after NAME, kept for back-compat.
      if [[ ${1:-} == "--" ]]; then
        shift
      fi
      ARGS+=("$@")
      break
      ;;
  esac
done

# -----------------------------------------------------------------------------
# Config directory
# -----------------------------------------------------------------------------
# XDG spec: XDG_CONFIG_HOME must be an absolute path. Anything else falls
# back to $HOME/.config.

xdg=${XDG_CONFIG_HOME:-}
if [[ -n "$xdg" && "$xdg" == /* ]]; then
  ENV_DIR="$xdg/cce"
else
  if [[ -n "$xdg" && "$xdg" != /* ]]; then
    echo "Warning: XDG_CONFIG_HOME is not an absolute path; ignoring" >&2
  fi
  if [[ -z "${HOME:-}" ]]; then
    echo "Error: HOME is not set and XDG_CONFIG_HOME is not a usable absolute path" >&2
    exit 1
  fi
  ENV_DIR="$HOME/.config/cce"
fi

# -----------------------------------------------------------------------------
# Help / version
# -----------------------------------------------------------------------------

if [[ "$SHOW_VERSION" == true ]]; then
  echo "cce $VERSION"
  exit 0
fi

print_help() {
  cat <<'EOF'
Claude Code Environment Manager

Usage: cce [OPTIONS] NAME [ARGS...]
       cce [OPTIONS]                # list / interactive picker

Loads <NAME>.env from the config directory and runs the target command
with those environment variables. All ARGS after NAME are forwarded to
the command unchanged.

Arguments:
  NAME      Environment name (must match [A-Za-z0-9_][A-Za-z0-9._-]*).
  ARGS...   Arguments forwarded to the command.

Options:
  -c, --command <CMD>  Command executable to run [default: claude].
      --version        Print version.
  -h, --help           Print help.

If NAME is omitted, lists available environments (interactive selection
via fzf if installed).

Config directory:
  $XDG_CONFIG_HOME/cce/    (if XDG_CONFIG_HOME is set and absolute)
  $HOME/.config/cce/       (otherwise)
EOF
}

if [[ "$SHOW_HELP" == true ]]; then
  print_help
  exit 0
fi

# -----------------------------------------------------------------------------
# Environment discovery
# -----------------------------------------------------------------------------

# Print sorted, valid environment names to stdout (one per line). Files
# whose basename does not match $NAME_RE are skipped with a warning so the
# picker never offers something the runner would reject.
get_env_names() {
  if [[ ! -d "$ENV_DIR" ]]; then
    return 0
  fi
  local f base names=()
  for f in "$ENV_DIR"/*.env; do
    [[ -f "$f" ]] || continue
    base=$(basename "$f" .env)
    if [[ "$base" =~ $NAME_RE ]]; then
      names+=("$base")
    else
      echo "Warning: skipping invalid env file name: '$base.env'" >&2
    fi
  done
  if [[ ${#names[@]} -gt 0 ]]; then
    printf '%s\n' "${names[@]}" | sort
  fi
}

# Reject names that would not survive validation. Called for every
# user-supplied name before it touches the filesystem.
validate_env_name() {
  local name="$1"
  if [[ -z "$name" || ! "$name" =~ $NAME_RE ]]; then
    echo "Error: invalid environment name '$name'" >&2
    echo "Names must start with a letter, digit, or underscore and contain only [A-Za-z0-9._-]." >&2
    exit 1
  fi
}

# Print the usage hint and the available environment list. Takes the
# already-discovered env names as positional arguments to avoid running
# get_env_names twice (which would duplicate any "skipping invalid name"
# warnings).
list_environments() {
  echo "Usage: cce [OPTIONS] NAME [ARGS...]"
  if [[ $# -eq 0 ]]; then
    echo "No environments found."
    echo "Create environment files in: $ENV_DIR"
    if [[ ! -d "$ENV_DIR" ]]; then
      echo "Directory does not exist yet."
    fi
    return
  fi
  local n
  for n in "$@"; do
    echo "  $n"
  done
}

# -----------------------------------------------------------------------------
# Mode dispatch — list / picker / run
# -----------------------------------------------------------------------------

if [[ -z "$ENV_NAME" ]]; then
  env_names=()
  while IFS= read -r name; do
    env_names+=("$name")
  done < <(get_env_names)

  if [[ ${#env_names[@]} -gt 0 ]] && command -v fzf >/dev/null 2>&1; then
    set +e
    selected=$(printf '%s\n' "${env_names[@]}" | fzf)
    fzf_status=$?
    set -e

    case $fzf_status in
      0)
        ENV_NAME="$selected"
        ;;
      1 | 130)
        # 1 = no match selected; 130 = SIGINT (Ctrl-C / Esc).
        list_environments ${env_names[@]+"${env_names[@]}"}
        exit 0
        ;;
      *)
        echo "Error: fzf exited with status $fzf_status" >&2
        exit 1
        ;;
    esac
  else
    list_environments ${env_names[@]+"${env_names[@]}"}
    exit 0
  fi
fi

# -----------------------------------------------------------------------------
# Run an environment
# -----------------------------------------------------------------------------

validate_env_name "$ENV_NAME"

ENV_FILE="$ENV_DIR/${ENV_NAME}.env"

if [[ ! -f "$ENV_FILE" ]]; then
  echo "Error: environment file not found: $ENV_FILE" >&2
  exit 1
fi

# Best-effort permission warning. GNU and BSD stat use different flags;
# try both and only accept a purely-numeric result so a wrong-syntax stat
# (e.g. GNU stat invoked with BSD-style `-f`, which prints filesystem
# info instead of mode) cannot poison the arithmetic below.
get_file_mode() {
  local out
  out=$(stat -c '%a' "$1" 2>/dev/null || true)
  if [[ "$out" =~ ^[0-9]+$ ]]; then
    printf '%s\n' "$out"
    return
  fi
  out=$(stat -f '%Lp' "$1" 2>/dev/null || true)
  if [[ "$out" =~ ^[0-9]+$ ]]; then
    printf '%s\n' "$out"
    return
  fi
}

file_mode=$(get_file_mode "$ENV_FILE")
if [[ -n "$file_mode" ]] && ((8#$file_mode & 0022)); then
  echo "Warning: $ENV_FILE is writable by group or other (mode $file_mode)" >&2
  echo "Consider: chmod 600 $ENV_FILE" >&2
fi

# Pre-validate the env file's bash syntax. Parse errors don't trigger the
# ERR trap below (no command runs), so we catch them here for a clean error
# that names the env file rather than letting bash's own parser message
# appear with no `cce` context.
if ! bash -n "$ENV_FILE" 2>/dev/null; then
  echo "Error: syntax error in environment file: $ENV_FILE" >&2
  bash -n "$ENV_FILE" 2>&1 | sed 's/^/  /' >&2 || true
  exit 1
fi

# Source the environment file. An ERR trap adds context if anything inside
# the file fails at runtime — `if ! . file; then` would put the source in
# a tested context, which disables `set -e` inside the sourced file and
# would mask real errors.
trap 'echo "Error: failed while loading environment file: $ENV_FILE" >&2' ERR
# shellcheck disable=SC1090  # env file path is intentionally dynamic
. "$ENV_FILE"
trap - ERR

# After sourcing, the env file may have changed PATH; verify $COMMAND now.
if ! command -v "$COMMAND" >/dev/null 2>&1; then
  echo "Error: command not found: $COMMAND" >&2
  echo "Make sure '$COMMAND' is installed and on PATH." >&2
  exit 127
fi

# `${ARGS[@]+"${ARGS[@]}"}` is the Bash 3.2 + `set -u` workaround for
# expanding a possibly-empty array without tripping "unbound variable".
exec "$COMMAND" ${ARGS[@]+"${ARGS[@]}"}
