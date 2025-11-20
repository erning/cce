use cce::config::Environment;
use std::path::PathBuf;

#[test]
fn test_load_basic_env() {
    let path = PathBuf::from("tests/fixtures/basic.env");
    let env = Environment::from_file(path, "basic".to_string()).unwrap();

    assert_eq!(env.base_url, "https://api.anthropic.com");
    assert_eq!(env.auth_token, "sk_test_basic_token");
}

#[test]
fn test_load_env_with_variable_reference() {
    let path = PathBuf::from("tests/fixtures/with_vars.env");
    let env = Environment::from_file(path, "with_vars".to_string()).unwrap();

    assert_eq!(env.base_url, "https://api.anthropic.com/v1");
    assert_eq!(env.auth_token, "sk_test_var_token");
}
