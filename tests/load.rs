use std::{collections::HashMap, path::PathBuf};
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
        Self::populate_env(env);
        Self(
            env.keys()
                .chain(expected.keys())
                .map(Clone::clone)
                .collect(),
        )
    }

    fn populate_env(vars: &HashMap<String, String>) {
        for (k, v) in vars {
            std::env::set_var(k, v);
        }
    }
}

impl Drop for Setup {
    fn drop(&mut self) {
        for key in self.0.iter() {
            std::env::remove_var(key);
        }
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
    load(case.files, case.override_env)?;
    for (k, v) in case.expected {
        let var = std::env::var(k)?;
        assert_eq!(v, var);
    }
    Ok(())
}

fn assert_error(case: ErrorCase) -> AnyRes<()> {
    let _setup = Setup::from(&case);
    let result = load(case.files, case.override_env);
    match case.error.as_str() {
        "ParseError" => assert!(matches!(result, Err(PotenvError::ParseError(_)))),
        "EvaluationError" => assert!(matches!(result, Err(PotenvError::EvaluationError(_)))),
        _ => assert!(matches!(result, Err(_))),
    }
    Ok(())
}

fn load(files: Vec<PathBuf>, override_env: bool) -> Result<HashMap<String, String>, PotenvError> {
    Ok(if override_env {
        Potenv::default()
            .override_env(override_env)
            .load(files)?
            .collect()
    } else {
        potenv::load(files)?.collect()
    })
}
