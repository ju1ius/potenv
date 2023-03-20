use rstest::rstest;
use serde::Deserialize;

use super::*;
use crate::test_utils::{collect_spec_files, load_spec_file, AnyRes};

macro_rules! tok {
    ($k:ident, $v:literal, $l:literal, $c:literal) => {
        Token::new(TokenKind::$k, $v.to_string(), Position::new($l, $c))
    };
}

fn tokenize(input: &str) -> Result<Vec<Token>, SyntaxError> {
    Tokenizer::new(input.chars(), Some("<test>".into()))
        .into_iter()
        .collect()
}

fn assert_tokens(input: &str, expected: Vec<Token>) -> Result<(), SyntaxError> {
    let tokens = tokenize(input).unwrap();
    assert_eq!(expected, tokens);
    Ok(())
}

#[test]
fn tokenize_empty() -> Result<(), SyntaxError> {
    let expected = vec![tok!(Eof, "", 1, 1)];
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
        tok!(Eof, "", 5, 1),
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
        tok!(Eof, "", 1, 14),
    ];
    assert_tokens(input, expected)
}

#[rstest]
#[case("&", 1, 1)]
#[case("abc", 1, 4)]
#[case("a=b&c", 1, 4)]
#[case("a='0\n\x00'", 2, 1)]
#[case("a='1\n2\n3", 1, 3)]
#[case("a=\"1\n2\n3", 1, 3)]
#[case("a=${a-1\n2\n3", 1, 4)]
#[case("a=\"1\n2\n${3}\"", 3, 3)]
#[case("#x\na=`pwd`", 2, 3)]
#[case("#x\na=$(pwd)", 2, 4)]
fn test_error_position(#[case] input: &str, #[case] line: usize, #[case] col: usize) {
    let res = tokenize(input).unwrap_err();
    let expected = format!("on line {line}, column {col}");
    assert!(
        res.to_string().contains(&expected),
        "expected position ({line}, {col}) but got {res:?}"
    );
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
    for path in collect_spec_files("tokenization").into_iter() {
        let tests = load_spec_file::<TestCase>(&path)?;
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
    // println!("Running {}", desc);
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
    // println!("Running {}", desc);
    let result: Vec<_> = tokenize(input)?.into_iter().map(token_to_json).collect();
    assert_eq!(
        expected, result,
        "{}\nexpected: {:?}\nactual: {:?}",
        desc, expected, result
    );
    Ok(())
}

fn token_to_json(token: Token) -> TestToken {
    return TestToken {
        kind: match token.kind {
            TokenKind::Eof => "EOF".into(),
            k => format!("{:?}", k),
        },
        value: token.value,
    };
}
