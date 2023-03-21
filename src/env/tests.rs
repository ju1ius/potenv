use std::collections::HashMap;

use super::{EnvProvider, HashMapProvider, ProcessEnvProvider};

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
    let mut env = HashMapProvider::from(HashMap::new());
    let name = "__TEST_VAR__";
    let value = "foobar";
    env.set_var(name, value);
    let result = env.var(name).unwrap();
    assert_eq!(value, result);
}
