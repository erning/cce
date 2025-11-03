## Why

Currently, the application uses `dirs::config_dir()` which returns platform-specific directories. On macOS, this defaults to `~/Library/Application Support/cce`, which is the macOS-specific convention. However, this deviates from the XDG Base Directory specification, which provides a standardized cross-platform approach for configuration files.

Using `$XDG_CONFIG_HOME/.config/cce` aligns with modern Linux/Unix conventions and provides better cross-platform consistency. The `XDG_CONFIG_HOME` environment variable, if set, should take precedence; otherwise, it should fall back to `$HOME/.config/cce`.

## What Changes

- Modify `EnvironmentManager::new()` to use `$XDG_CONFIG_HOME/.config/cce` instead of the platform-specific `config_dir()`
- Check for `$XDG_CONFIG_HOME` environment variable and use it if set
- Fall back to `$HOME/.config/cce` if `$XDG_CONFIG_HOME` is not set or empty
- Update configuration loading spec to reflect the new directory path
- **BREAKING**: Existing configuration files in `~/Library/Application Support/cce` (macOS) or other platform-specific locations will not be automatically found

## Impact

- Affected specs: `configuration-loading` (CONF-004 scenarios will need updating)
- Affected code: `src/manager.rs` (EnvironmentManager implementation)
- Migration required: Users will need to move existing `.env` files from old locations to new location
- Platform behavior: macOS users will see different default config path (moves from Application Support to .config)

## Migration Plan

1. Add logic to detect legacy config directories and provide helpful migration messages
2. Consider auto-migration of config files from old location to new location on first run
3. Update documentation to reflect new config directory location
