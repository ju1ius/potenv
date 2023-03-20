use std::collections::HashMap;
mod utils;

use potenv::{Potenv, PotenvError};
use utils::*;

#[test]
fn test_load() -> AnyRes<()> {
    for case in load_test_cases("evaluate.json")? {
        match case {
            TestCase::Success(t) => assert_success(t)?,
            TestCase::Error(t) => assert_error(t)?,
        };
    }
    Ok(())
}

struct Setup(Vec<String>);

impl Setup {
    pub fn new(env: &HashMap<String, String>, expected: &HashMap<String, String>) -> Self {
        populate_env(env);
        Self(
            env.keys()
                .chain(expected.keys())
                .map(Clone::clone)
                .collect(),
        )
    }
}

impl Drop for Setup {
    fn drop(&mut self) {
        cleanup_env(self.0.iter());
    }
}

impl From<&SuccesCase> for Setup {
    fn from(case: &SuccesCase) -> Self {
        Self::new(&case.env, &case.expected)
    }
}

impl From<&ErrorCase> for Setup {
    fn from(case: &ErrorCase) -> Self {
        Self::new(&case.env, &HashMap::new())
    }
}

fn assert_success(case: SuccesCase) -> AnyRes<()> {
    let _setup = Setup::from(&case);
    let mut potenv = Potenv::default().override_env(case.override_env);
    potenv.load(case.files)?;
    for (k, v) in case.expected {
        let var = std::env::var(k)?;
        assert_eq!(v, var);
    }
    Ok(())
}

fn assert_error(case: ErrorCase) -> AnyRes<()> {
    let _setup = Setup::from(&case);
    let mut potenv = Potenv::default().override_env(case.override_env);
    let result = potenv.load(case.files);
    match case.error.as_str() {
        "ParseError" => assert!(matches!(result, Err(PotenvError::ParseError(_)))),
        "EvaluationError" => assert!(matches!(result, Err(PotenvError::EvaluationError(_)))),
        _ => assert!(matches!(result, Err(_))),
    }
    Ok(())
}

fn populate_env(vars: &HashMap<String, String>) {
    for (k, v) in vars {
        std::env::set_var(k, v);
    }
}

fn cleanup_env(vars: impl IntoIterator<Item = impl AsRef<str>>) {
    for key in vars {
        std::env::remove_var(key.as_ref());
    }
}