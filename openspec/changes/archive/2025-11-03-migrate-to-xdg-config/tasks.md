## 1. Implementation

- [ ] 1.1 Modify `EnvironmentManager::new()` to check `$XDG_CONFIG_HOME` first
- [ ] 1.2 Fall back to `$HOME/.config` if `$XDG_CONFIG_HOME` is not set
- [ ] 1.3 Update `config_dir()` to return the new XDG-compliant path
- [ ] 1.4 Remove dependency on `dirs::config_dir()` if no longer needed
- [ ] 1.5 Test the implementation on different platforms (macOS, Linux)

## 2. Spec Updates

- [ ] 2.1 Update `CONF-004` scenarios in `specs/configuration-loading/spec.md`
- [ ] 2.2 Verify all scenario paths reflect `~/.config/cce/` instead of platform-specific paths

## 3. Validation

- [ ] 3.1 Run `openspec validate migrate-to-xdg-config --strict`
- [ ] 3.2 Fix any validation errors or warnings

## 4. Documentation

- [ ] 4.1 Update README or user documentation about config file location
- [ ] 4.2 Add migration instructions for users with existing configs
