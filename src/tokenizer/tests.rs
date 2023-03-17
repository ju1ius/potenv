use serde::Deserialize;
use serde_json;
use std::{error::Error, fs::File, io::BufReader, path::PathBuf};

use super::*;

type AnyRes<T> = Result<T, Box<dyn Error>>;

macro_rules! tok {
    ($k:ident, $v:literal, $l:literal, $c:literal) => {
        Token::new(TokenKind::$k, $v.to_string(), Position::new($l, $c))
    };
}

fn tokenize(input: &str) -> Result<Vec<Token>, SyntaxError> {
    Tokenizer::new(input, None).into_iter().collect()
}

fn assert_tokens(input: &str, expected: Vec<Token>) -> Result<(), SyntaxError> {
    let tokens = tokenize(input).unwrap();
    assert_eq!(expected, tokens);
    Ok(())
}

#[test]
fn tokenize_empty() -> Result<(), SyntaxError> {
    let expected = vec![tok!(EOF, "", 1, 1)];
    assert_tokens("", expected)
}

#[test]
fn tokenize_comments() -> Result<(), SyntaxError> {
    let input = r##"
# a comment
a=42
# another comment
"##;
    let expected = vec![
        tok!(Assign, "a", 3, 1),
        tok!(Characters, "42", 3, 3),
        tok!(EOF, "", 5, 1),
    ];
    assert_tokens(input, expected)
}

#[test]
fn simple_raw_values() -> Result<(), SyntaxError> {
    let input = "A=a B=1\tC=yes";
    let expected = vec![
        tok!(Assign, "A", 1, 1),
        tok!(Characters, "a", 1, 3),
        tok!(Assign, "B", 1, 5),
        tok!(Characters, "1", 1, 7),
        tok!(Assign, "C", 1, 9),
        tok!(Characters, "yes", 1, 11),
        tok!(EOF, "", 1, 14),
    ];
    assert_tokens(input, expected)
}

#[test]
fn shell_param_zero() {
    let input = "A=${0}";
    let res = tokenize(input);
    assert!(res.is_err(), "Expected error but got: {:?}", res);
}

/// Specification tests

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct TestToken {
    pub kind: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TestCase {
    Success {
        desc: String,
        input: String,
        expected: Vec<TestToken>,
    },
    Error {
        desc: String,
        input: String,
        error: String,
    },
}

impl ToString for TestCase {
    fn to_string(&self) -> String {
        match self {
            Self::Success { desc, .. } | Self::Error { desc, .. } => desc.clone(),
        }
    }
}

#[test]
fn test_spec() -> AnyRes<()> {
    for path in collect_spec_files()?.into_iter() {
        let tests = load_spec_file(&path)?;
        for (i, case) in tests.into_iter().enumerate() {
            let message = format!(
                "{:?} > {}: {}",
                path.file_name().unwrap(),
                i,
                case.to_string()
            );
            match case {
                TestCase::Success {
                    input, expected, ..
                } => {
                    assert_spec_expected(&input, expected, &message)?;
                }
                TestCase::Error { input, error, .. } => {
                    assert_spec_err(&input, &error, &message);
                }
            }
        }
    }
    Ok(())
}

fn assert_spec_err(input: &str, error: &str, desc: &str) {
    println!("Running {}", desc);
    let result = tokenize(input);
    assert!(
        result.is_err(),
        "{}\nexpected: {}\nactual: {:?}",
        desc,
        error,
        result
    );
}

fn assert_spec_expected(input: &str, expected: Vec<TestToken>, desc: &str) -> AnyRes<()> {
    println!("Running {}", desc);
    let result: Vec<_> = tokenize(input)?.into_iter().map(token_to_json).collect();
    assert_eq!(
        expected, result,
        "{}\nexpected: {:?}\nactual: {:?}",
        desc, expected, result
    );
    Ok(())
}

fn collect_spec_files() -> AnyRes<Vec<PathBuf>> {
    let root =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?).join("dotenv-spec/tests/tokenization");
    let mut paths: Vec<_> = std::fs::read_dir(root)?
        .flat_map(|r| r.map(|e| e.path()))
        .filter(|p| p.is_file())
        .collect();
    paths.sort();
    Ok(paths)
}

fn load_spec_file(path: &PathBuf) -> AnyRes<Vec<TestCase>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let data: Vec<TestCase> = serde_json::from_reader(reader)?;
    Ok(data)
}

fn token_to_json(token: Token) -> TestToken {
    let kind = format!("{:?}", token.kind);
    return TestToken {
        kind,
        value: token.value,
    };
}
