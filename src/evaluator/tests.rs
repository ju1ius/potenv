use std::collections::HashMap;

use serde::Deserialize;

use crate::{
    env::HashMapProvider,
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

fn eval(input: &str, env: Scope, override_env: bool) -> AnyRes<Scope> {
    let provider = HashMapProvider::from(env);
    let mut eval = Evaluator::new(&provider, override_env);
    let ast = parse(input, Some("<test>"))?;
    eval.evaluate(ast)?;
    Ok(eval.scope())
}

#[test]
fn test_bug() -> AnyRes<()> {
    let desc = "test bug";
    let input = "a=${a+${b?}}";
    let env = scope!["a": ""];
    // let expected = scope!["a": ""];
    let error = "ParseError".to_string();
    // assert_spec_expected(desc, input, env, false, expected)?;
    assert_spec_err(desc, input, env, true, error)?;
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
        #[serde(default)]
        env: HashMap<String, String>,
        #[serde(rename = "override", default)]
        override_env: bool,
        expected: HashMap<String, String>,
    },
    Error {
        desc: String,
        input: String,
        #[serde(default)]
        env: HashMap<String, String>,
        #[serde(rename = "override", default)]
        override_env: bool,
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
    env: HashMap<String, String>,
    override_env: bool,
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
    env: HashMap<String, String>,
    override_env: bool,
    _error: String,
) -> AnyRes<()> {
    println!("Running: {}", desc);
    let result = eval(input, env, override_env);
    assert!(result.is_err());
    println!("Ok");
    Ok(())
}
