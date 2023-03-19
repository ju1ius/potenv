use std::collections::HashMap;

use serde::Deserialize;

use crate::{
    parser::parse,
    test_utils::{collect_spec_files, load_spec_file, AnyRes},
};

use super::{Evaluator, Scope};

macro_rules! scope {
    ($($k:literal: $v:literal),+) => {{
       let mut hm = HashMap::new();
       $(hm.insert($k.to_string(), $v.to_string()))+;
       hm
    }};
}

fn eval(input: &str, env: Option<Scope>, override_env: Option<bool>) -> AnyRes<Scope> {
    let mut eval = Evaluator::new(env.unwrap_or_default(), override_env.unwrap_or(false));
    let ast = parse(input, Some("<test>"))?;
    eval.evaluate(ast)?;
    Ok(eval.scope())
}

#[test]
fn test_bug() -> AnyRes<()> {
    let desc = "test bug";
    let input = "a=${a:?foo}";
    let env = scope!["a": ""];
    // let expected = scope!["a": ""];
    let error = "ParseError".to_string();
    // assert_spec_expected(desc, input, Some(env), Some(false), expected)?;
    assert_spec_err(desc, input, Some(env), Some(true), error)?;
    println!("Ok bug!");
    Ok(())
}

/// Specification tests

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TestCase {
    Success {
        desc: String,
        input: String,
        env: Option<HashMap<String, String>>,
        #[serde(rename = "override")]
        override_env: Option<bool>,
        expected: HashMap<String, String>,
    },
    Error {
        desc: String,
        input: String,
        env: Option<HashMap<String, String>>,
        #[serde(rename = "override")]
        override_env: Option<bool>,
        error: String,
    },
}

impl std::fmt::Display for TestCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success { desc, .. } | Self::Error { desc, .. } => f.write_str(desc),
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
                TestCase::Success {
                    input,
                    env,
                    override_env,
                    expected,
                    ..
                } => {
                    assert_spec_expected(&message, &input, env, override_env, expected)?;
                }
                TestCase::Error {
                    input,
                    env,
                    override_env,
                    error,
                    ..
                } => {
                    assert_spec_err(&message, &input, env, override_env, error)?;
                }
            }
        }
    }
    Ok(())
}

fn assert_spec_expected(
    desc: &str,
    input: &str,
    env: Option<HashMap<String, String>>,
    override_env: Option<bool>,
    expected: HashMap<String, String>,
) -> AnyRes<()> {
    let result = eval(input, env, override_env)?;
    assert_eq!(expected, result);
    println!("Ok");
    Ok(())
}

fn assert_spec_err(
    desc: &str,
    input: &str,
    env: Option<HashMap<String, String>>,
    override_env: Option<bool>,
    _error: String,
) -> AnyRes<()> {
    println!("Running: {}", desc);
    let result = eval(input, env, override_env);
    assert!(result.is_err());
    println!("Ok");
    Ok(())
}
