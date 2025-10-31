# CCE (Claude Code Environment) Manager

CCE is a shell script that allows you to manage multiple Claude Code environments with different API configurations. It enables easy switching between different API providers and authentication tokens without having to manually set environment variables each time.

## Features

- **Multi-Provider Support**: Switch between different Claude API providers (e.g., GLM, Kimi, Minimax)
- **Environment Management**: Manage multiple API keys for different accounts or projects
- **Quick Configuration**: Quickly change API endpoints for testing different services
- **Environment Separation**: Maintain separate configurations for development and production

## Installation

1. Download the `cce` script to your preferred location (e.g., `/usr/local/bin/` or `~/bin/`):

   ```bash
   curl -o cce https://raw.githubusercontent.com/erning/cce/refs/heads/master/cce
   chmod +x cce
   ```

2. Ensure the script is in your PATH or reference it with the full path.

## Setup

Create environment configuration files in `~/.config/cce/`. Each file should be named `<name>.env` and contain the following exports:

### Example Environment File (`~/.config/cce/glm.env`)

```bash
export ANTHROPIC_BASE_URL="https://open.bigmodel.cn/api/anthropic"
export ANTHROPIC_AUTH_TOKEN="your_token_here"
```

### Configuration Parameters

- **ANTHROPIC_BASE_URL**: The API endpoint URL
- **ANTHROPIC_AUTH_TOKEN**: Your authentication token/key

## Usage

### List Available Environments

```bash
./cce
```

This will display all available environment configurations in `~/.config/cce/`.

### Use a Specific Environment

```bash
./cce <environment_name> [claude-code arguments...]
```

### Examples

```bash
# List all available environments
./cce

# Use GLM environment with --help
./cce glm --help

# Use Kimi environment with a prompt
./cce kimi-k2 "Write a Python script"

# Use Minimax environment with --version
./cce minimax-m2 --version
```

## Requirements

- **Claude Code CLI tool** must be installed and accessible as `claude`
- **Environment directory**: `~/.config/cce/`
- **Valid environment files** with proper permissions

## How It Works

1. CCE looks for environment files in `~/.config/cce/`
2. When you specify an environment name, it sources the corresponding `.env` file
3. It then passes all remaining arguments to the `claude` command
4. The Claude Code CLI uses the environment variables from the sourced file

## Directory Structure

```
~/.config/cce/
├── glm.env       # GLM provider configuration
├── kimi-k2.env   # Kimi K2 configuration
└── minimax-m2.env # Minimax M2 configuration
```

## Use Cases

- **Switch API Providers**: Easily switch between different Claude API providers without modifying system-wide environment variables
- **Multiple Accounts**: Manage API keys for different accounts or projects
- **Testing & Development**: Quickly change API endpoints for testing different services
- **Environment Isolation**: Maintain separate configurations for development and production environments

## Error Handling

- If no environment name is provided, CCE lists available environments
- If the specified environment file doesn't exist, an error message is displayed
- Exits with appropriate error codes for debugging

## Troubleshooting

### "Directory ~/.config/cce does not exist"

Create the directory:
```bash
mkdir -p ~/.config/cce
```

### "Environment file not found"

Ensure that:
1. The environment file exists in `~/.config/cce/`
2. The file has a `.env` extension
3. The file contains valid `export` statements for the required variables

### "claude: command not found"

Ensure that the Claude Code CLI tool is installed and accessible in your PATH as `claude`.

## License

The MIT License
