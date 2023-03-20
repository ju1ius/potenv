use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use potenv::{env::HashMapProvider, Potenv, PotenvError};
use utils::{get_resource_path, load_test_cases, AnyRes, ErrorCase, SuccesCase, TestCase};

mod utils;

type Scope = HashMap<String, String>;

fn eval(file: impl AsRef<Path>, env: Scope, override_env: bool) -> Result<Scope, PotenvError> {
    let env = HashMapProvider::from(env);
    let potenv = Potenv::new(env, override_env);
    let scope = potenv.evaluate(vec![PathBuf::from(file.as_ref())])?;
    Ok(scope)
}

#[test]
fn test_file_not_found() -> AnyRes<()> {
    let file = get_resource_path("nope.txt")?;
    let result = eval(&file, Default::default(), false);
    assert!(matches!(result, Err(PotenvError::Io(_))));
    Ok(())
}

#[test]
fn test_evaluate() -> AnyRes<()> {
    for case in load_test_cases("evaluate.json")? {
        match case {
            TestCase::Success(t) => assert_success(t)?,
            TestCase::Error(t) => assert_error(t)?,
        };
    }
    Ok(())
}

fn assert_success(case: SuccesCase) -> AnyRes<()> {
    let env = HashMapProvider::from(case.env);
    let potenv = Potenv::new(env, case.override_env);
    let scope = potenv.evaluate(case.files)?;
    assert_eq!(case.expected, scope);
    Ok(())
}

fn assert_error(case: ErrorCase) -> AnyRes<()> {
    let env = HashMapProvider::from(case.env);
    let potenv = Potenv::new(env, case.override_env);
    let result = potenv.evaluate(case.files);
    match case.error.as_str() {
        "ParseError" => assert!(matches!(result, Err(PotenvError::ParseError(_)))),
        "EvaluationError" => assert!(matches!(result, Err(PotenvError::EvaluationError(_)))),
        _ => assert!(matches!(result, Err(_))),
    }
    Ok(())
}
