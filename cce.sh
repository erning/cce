#!/bin/bash

# ==============================================================================
# CCE (Claude Code Environment) Manager
# ==============================================================================
#
# OVERVIEW:
# --------
# CCE is a shell script that allows you to manage multiple Claude Code
# environments with different API configurations. It enables easy switching
# between different API providers and authentication tokens without having
# to manually set environment variables each time.
#
# USE CASES:
# ----------
# - Switch between different Claude API providers (e.g., GLM, Kimi, Minimax)
# - Manage multiple API keys for different accounts or projects
# - Quickly change API endpoints for testing different services
# - Maintain separate configurations for development and production
#
# CONFIGURATION:
# --------------
# Environment files are stored in ~/.config/cce/ with the format:
#   <name>.env
#
# Each environment file should contain export statements for:
#   - ANTHROPIC_BASE_URL: The API endpoint URL
#   - ANTHROPIC_AUTH_TOKEN: Your authentication token/key
#
# EXAMPLE ENVIRONMENT FILE (~/.config/cce/glm.env):
#   export ANTHROPIC_BASE_URL="https://open.bigmodel.cn/api/anthropic"
#   export ANTHROPIC_AUTH_TOKEN="your_token_here"
#
# USAGE:
# -----
# 1. List available environments (with interactive fzf selection if available):
#    ./cce
#
# 2. Use a specific environment:
#    ./cce <environment_name> [-- ARGS...]
#
# 3. Use custom command:
#    ./cce -c claude-code <environment_name>
#    ./cce --command echo <environment_name>
#
# 4. Validate environments:
#    ./cce --validate [environment_name]
#
# 5. Examples:
#    ./cce glm -- --help
#    ./cce kimi-k2 -- "Write a Python script"
#    ./cce minimax-m2 -- --version
#
# OPTIONS:
# -------
#   -c, --command <CMD>  Command executable to run [default: claude]
#       --validate       Validate environment files
#       --version        Print version
#   -h, --help           Print help
#
# REQUIREMENTS:
# ------------
# - Claude Code CLI tool must be installed and accessible as 'claude'
# - Environment directory: ~/.config/cce/ or $XDG_CONFIG_HOME/cce/
# - Valid environment files with proper permissions
#
# ERROR HANDLING:
# --------------
# - If no environment name is provided, lists available environments
# - If specified environment file doesn't exist, shows an error message
# - Exits with appropriate error codes for debugging
#
# ==============================================================================

set -e

# Read version from Cargo.toml if available, otherwise fallback
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [[ -f "$SCRIPT_DIR/Cargo.toml" ]]; then
  VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' "$SCRIPT_DIR/Cargo.toml" | head -1)
fi
VERSION="${VERSION:-2.0.8}"

# Parse arguments
COMMAND="claude"
ENV_NAME=""
VALIDATE=false
SHOW_HELP=false
SHOW_VERSION=false
ARGS=()

while [[ $# -gt 0 ]]; do
  case $1 in
    -c|--command)
      if [[ $# -lt 2 ]]; then
        echo "Error: --command requires an argument" >&2
        exit 1
      fi
      COMMAND="$2"
      shift 2
      ;;
    --validate)
      VALIDATE=true
      shift
      ;;
    -h|--help)
      SHOW_HELP=true
      shift
      ;;
    --version)
      SHOW_VERSION=true
      shift
      ;;
    --)
      shift
      ARGS+=("$@")
      break
      ;;
    *)
      if [[ -z "$ENV_NAME" ]]; then
        ENV_NAME="$1"
      else
        ARGS+=("$1")
      fi
      shift
      ;;
  esac
done

# Get config directory (XDG_CONFIG_HOME or ~/.config)
if [[ -n "${XDG_CONFIG_HOME:-}" ]] && [[ -n "${XDG_CONFIG_HOME// /}" ]]; then
  ENV_DIR="$XDG_CONFIG_HOME/cce"
else
  ENV_DIR="$HOME/.config/cce"
fi

# Print version
if [[ "$SHOW_VERSION" == true ]]; then
  echo "cce $VERSION"
  exit 0
fi

# Print help
print_help() {
  echo "Claude Code Environment Manager"
  echo ""
  echo "Usage: cce [OPTIONS] [NAME] [-- ARGS...]"
  echo ""
  echo "Arguments:"
  echo "  [NAME]      Environment name"
  echo "  [ARGS...]   Arguments to pass to command"
  echo ""
  echo "Options:"
  echo "  -c, --command <CMD>  Command executable to run [default: claude]"
  echo "      --validate       Validate environment files"
  echo "      --version        Print version"
  echo "  -h, --help           Print help"
}

if [[ "$SHOW_HELP" == true ]]; then
  print_help
  exit 0
fi

# Collect sorted environment names into an array
# Usage: get_env_names result_array
get_env_names() {
  local -n _result=$1
  _result=()
  if [[ -d "$ENV_DIR" ]]; then
    local files=()
    for f in "$ENV_DIR"/*.env; do
      if [[ -f "$f" ]]; then
        files+=("$(basename "$f" .env)")
      fi
    done
    if [[ ${#files[@]} -gt 0 ]]; then
      while IFS= read -r name; do
        _result+=("$name")
      done < <(printf '%s\n' "${files[@]}" | sort)
    fi
  fi
}

# Validate environment name (reject path traversal)
validate_env_name() {
  local name="$1"
  if [[ -z "$name" ]] || [[ "$name" == */* ]] || [[ "$name" == *\\* ]] || [[ "$name" == *..* ]]; then
    echo "Error: Invalid environment name '$name'" >&2
    exit 1
  fi
}

# List available environments
list_environments() {
  echo "Usage: cce [OPTIONS] [NAME] [-- ARGS...]"

  local env_names=()
  get_env_names env_names

  if [[ ${#env_names[@]} -eq 0 ]]; then
    echo "No environments found."
    echo "Create environment files in: $ENV_DIR"
    if [[ ! -d "$ENV_DIR" ]]; then
      echo "Directory does not exist yet."
    fi
  else
    for name in "${env_names[@]}"; do
      echo "  $name"
    done
  fi
}

# Validate environment file
# Returns 0 if valid, 1 if invalid
validate_environment() {
  local env_name="$1"
  local env_file="$ENV_DIR/${env_name}.env"

  if [[ ! -f "$env_file" ]]; then
    echo "Validating $env_name... ERROR: File not found"
    return 1
  fi

  printf "Validating %s... " "$env_name"

  # Check for required variables
  local missing_required=()
  local missing_optional=()

  if ! grep -q "^export ANTHROPIC_AUTH_TOKEN=" "$env_file" && \
     ! grep -q "^ANTHROPIC_AUTH_TOKEN=" "$env_file"; then
    missing_required+=("ANTHROPIC_AUTH_TOKEN")
  fi

  if ! grep -q "^export ANTHROPIC_BASE_URL=" "$env_file" && \
     ! grep -q "^ANTHROPIC_BASE_URL=" "$env_file"; then
    missing_optional+=("ANTHROPIC_BASE_URL")
  fi

  if [[ ${#missing_required[@]} -eq 0 ]]; then
    echo "✓ OK"
    if [[ ${#missing_optional[@]} -gt 0 ]]; then
      echo "  Note: Missing optional vars: ${missing_optional[*]}"
    fi
    return 0
  else
    echo "✗ INVALID"
    echo "  Missing required: ${missing_required[*]}"
    return 1
  fi
}

# Run validate mode
if [[ "$VALIDATE" == true ]]; then
  all_valid=true
  if [[ -n "$ENV_NAME" ]]; then
    validate_env_name "$ENV_NAME"
    validate_environment "$ENV_NAME" || all_valid=false
  else
    local_env_names=()
    get_env_names local_env_names

    if [[ ${#local_env_names[@]} -eq 0 ]]; then
      echo "No environments found to validate."
    else
      for env_name in "${local_env_names[@]}"; do
        validate_environment "$env_name" || all_valid=false
      done
      echo ""
      if [[ "$all_valid" == true ]]; then
        echo "All environments are valid."
      else
        echo "Some environments have issues."
      fi
    fi
  fi
  if [[ "$all_valid" == false ]]; then
    exit 1
  fi
  exit 0
fi

# Check if first arg starts with '-', treat as showing list
if [[ "$ENV_NAME" == -* ]]; then
  list_environments
  exit 0
fi

# No environment name provided
if [[ -z "$ENV_NAME" ]]; then
  env_names=()
  get_env_names env_names

  # Try interactive fzf selection if available
  if [[ ${#env_names[@]} -gt 0 ]] && command -v fzf &> /dev/null; then
    selected=$(printf "%s\n" "${env_names[@]}" | fzf) || {
      # User cancelled (ESC/q), show list
      list_environments
      exit 0
    }

    if [[ -n "$selected" ]]; then
      ENV_NAME="$selected"
    else
      list_environments
      exit 0
    fi
  else
    list_environments
    exit 0
  fi
fi

# Validate environment name
validate_env_name "$ENV_NAME"

# Run environment
ENV_FILE="$ENV_DIR/${ENV_NAME}.env"

if [[ ! -f "$ENV_FILE" ]]; then
  echo "Error: Environment file not found: $ENV_FILE" >&2
  exit 1
fi

# Source the environment file and execute the command
. "$ENV_FILE"
exec "$COMMAND" "${ARGS[@]}"
