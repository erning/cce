use cce::config::Environment;
use std::path::PathBuf;

#[test]
fn test_load_basic_env() {
    let path = PathBuf::from("tests/fixtures/basic.env");
    let env = Environment::from_file(path, "basic".to_string()).unwrap();

    assert_eq!(
        env.env_vars.get("ANTHROPIC_BASE_URL").unwrap(),
        "https://api.anthropic.com"
    );
    assert_eq!(
        env.env_vars.get("ANTHROPIC_AUTH_TOKEN").unwrap(),
        "sk_test_basic_token"
    );
}

#[test]
fn test_load_env_with_variable_reference() {
    let path = PathBuf::from("tests/fixtures/with_vars.env");
    let env = Environment::from_file(path, "with_vars".to_string()).unwrap();

    assert_eq!(
        env.env_vars.get("ANTHROPIC_BASE_URL").unwrap(),
        "https://api.anthropic.com/v1"
    );
    assert_eq!(
        env.env_vars.get("ANTHROPIC_AUTH_TOKEN").unwrap(),
        "sk_test_var_token"
    );
}

#[test]
fn test_load_env_with_command_substitution() {
    let path = PathBuf::from("tests/fixtures/with_command.env");
    let env =
        Environment::from_file(path, "with_command".to_string()).unwrap();

    assert_eq!(
        env.env_vars.get("ANTHROPIC_BASE_URL").unwrap(),
        "https://api.anthropic.com"
    );
    assert_eq!(
        env.env_vars.get("ANTHROPIC_AUTH_TOKEN").unwrap(),
        "sk_test_from_command"
    );
}

#[test]
fn test_load_complex_shell_logic() {
    let path = PathBuf::from("tests/fixtures/complex.env");
    let env = Environment::from_file(path, "complex".to_string()).unwrap();

    // Test that variable reference + command substitution works
    assert_eq!(
        env.env_vars.get("ANTHROPIC_BASE_URL").unwrap(),
        "https://api.example.com/v1"
    );
    assert_eq!(
        env.env_vars.get("ANTHROPIC_AUTH_TOKEN").unwrap(),
        "sk_prod_token_123"
    );
}
