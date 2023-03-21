use std::collections::HashMap;

use super::{EnvProvider, ProcessEnvProvider};

#[test]
fn test_process_env() {
    let mut env = ProcessEnvProvider;
    let name = "__TEST_VAR__";
    let value = "foobar";
    env.set_var(name, value);
    let result = env.var(name).unwrap();
    std::env::remove_var(name);
    assert_eq!(value, result);
}

#[test]
fn test_hashmap() {
    let mut env = HashMap::new();
    let name = "FOO";
    let value = "foobar";
    env.set_var(name, value);
    let result = env.var(name).unwrap();
    assert_eq!(value, result);
}
