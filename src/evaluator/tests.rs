use std::collections::HashMap;

use serde::Deserialize;
use thiserror::Error;

use super::{EvaluationError, Evaluator, Scope};
use crate::{
    parser::{parse, ParseError},
    test_utils::{collect_spec_files, load_spec_file, AnyRes},
};

macro_rules! scope {
    ($($k:literal: $v:literal),+) => {{
       let mut hm = HashMap::new();
       $(hm.insert($k.to_string(), $v.to_string()))+;
       hm
    }};
}

#[derive(Debug, Error)]
enum EvalError {
    #[error(transparent)]
    Parse(#[from] ParseError),
    #[error(transparent)]
    Eval(#[from] EvaluationError),
}

fn eval(input: &str, env: Scope, override_env: bool) -> Result<Scope, EvalError> {
    let mut eval = Evaluator::new(&env, override_env);
    let ast = parse(input, Some("<test>".into()))?;
    eval.evaluate(ast)?;
    Ok(eval.into_scope())
}

#[test]
fn test_bug() -> AnyRes<()> {
    assert_spec_err(ErrorCase {
        desc: "test bug".into(),
        input: "a=${a+${b?}}".into(),
        env: scope!["a": ""],
        override_env: true,
        error: "UndefinedVariable".into(),
    })?;
    Ok(())
}

/// Specification tests

#[derive(Debug, Default, Deserialize)]
struct SuccessCase {
    desc: String,
    input: String,
    #[serde(default)]
    env: HashMap<String, String>,
    #[serde(rename = "override", default)]
    override_env: bool,
    expected: HashMap<String, String>,
}

#[derive(Debug, Default, Deserialize)]
struct ErrorCase {
    desc: String,
    input: String,
    #[serde(default)]
    env: HashMap<String, String>,
    #[serde(rename = "override", default)]
    override_env: bool,
    error: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TestCase {
    Success(SuccessCase),
    Error(ErrorCase),
}

impl std::fmt::Display for TestCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success(SuccessCase { desc, .. }) | Self::Error(ErrorCase { desc, .. }) => {
                f.write_str(desc)
            }
        }
    }
}

#[test]
fn test_spec() -> AnyRes<()> {
    for file in collect_spec_files("evaluation") {
        println!("File: {}", file.to_str().unwrap());
        for (i, case) in load_spec_file::<TestCase>(&file)?.into_iter().enumerate() {
            let message = format!("{:?} > {}: {}", file.file_name().unwrap(), i, case);
            println!("{}", message);
            match case {
                TestCase::Success(t) => assert_spec_expected(t)?,
                TestCase::Error(t) => assert_spec_err(t)?,
            };
        }
    }
    Ok(())
}

fn assert_spec_expected(case: SuccessCase) -> Result<(), EvalError> {
    let result = eval(&case.input, case.env, case.override_env)?;
    assert_eq!(case.expected, result);
    println!("Ok");
    Ok(())
}

fn assert_spec_err(case: ErrorCase) -> Result<(), EvalError> {
    let err = eval(&case.input, case.env, case.override_env).unwrap_err();
    match case.error.as_str() {
        "ParseError" => assert!(matches!(err, EvalError::Parse(_))),
        "UndefinedVariable" => assert!(matches!(err, EvalError::Eval(_))),
        _ => (),
    }
    println!("Ok");
    Ok(())
}
