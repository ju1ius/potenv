use std::collections::HashMap;

use super::{EnvProvider, HashMapProvider, ProcessEnvProvider};

#[test]
fn test_process_env() {
    let mut env = ProcessEnvProvider;
    let name = "__TEST_VAR__";
    let value = "foobar";
    env.set_var(name, value);
    let result = env.get_var(name).unwrap();
    std::env::remove_var(name);
    assert_eq!(value, result);
}

#[test]
fn test_hashmap() {
    let mut env = HashMapProvider::from(HashMap::new());
    let name = "__TEST_VAR__";
    let value = "foobar";
    env.set_var(name, value);
    let result = env.get_var(name).unwrap();
    assert_eq!(value, result);
}

#[test]
fn test_hashmap_provider_from_iter() {
    let to_str = |(k, v): (&str, &str)| (k.to_string(), v.to_string());
    let input = [("a", "1"), ("b", "2")];
    let result: HashMapProvider = input.into_iter().collect();
    let expected = HashMapProvider::from(HashMap::from_iter(input.into_iter().map(to_str)));
    assert_eq!(expected, result);
}
