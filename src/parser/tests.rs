#![allow(unused_macros)]

use rstest::rstest;

use super::{ast::Assignment, ParseResult, Parser};
use crate::{
    parser::ParseError,
    tokenizer::{
        pos::Position,
        token::{Token, TokenKind},
        TokenizerResult,
    },
};

macro_rules! tok {
    ($k:ident, $v:literal, $l:literal, $c:literal) => {
        Ok(Token::new(
            TokenKind::$k,
            $v.to_string(),
            Position::new($l, $c),
        ))
    };
    ($k:ident, $v:literal) => {
        tok!($k, $v, 0, 0)
    };
}

macro_rules! terr {
    ($e:expr) => {
        Err(ParseError::$e)
    };
}

#[rstest]
#[case::eof_in_assignment_list(
    vec![],
    |r| assert!(matches!(r, Err(ParseError::Eof))),
)]
#[case::unexpected_in_assignment_list(
    vec![tok!(Characters, "foo")],
    |r| assert!(matches!(r, Err(ParseError::Unexpected(_)))),
)]
#[case::eof_in_value(
    vec![tok!(Assign, "foo")],
    |r| assert!(matches!(r, Err(ParseError::Eof))),
)]
#[case::unexpected_in_value(
    vec![
        tok!(Assign, "foo"),
        tok!(ExpansionOperator, "bar")
    ],
    |r| assert!(matches!(r, Err(ParseError::Unexpected(_)))),
)]
#[case::unexpected_in_operator(
    vec![
        tok!(Assign, "foo"),
        tok!(StartExpansion, "bar"),
        tok!(Assign, "baz"),
    ],
    |r| assert!(matches!(r, Err(ParseError::Unexpected(_)))),
)]
#[case::unknown_operator(
    vec![
        tok!(Assign, "foo"),
        tok!(StartExpansion, "bar"),
        tok!(ExpansionOperator, "<lol>"),
        tok!(EndExpansion, ""),
    ],
    |r| assert!(matches!(r, Err(ParseError::UnknownOperator(_)))),
)]
#[case::eof_in_expansion(
    vec![
        tok!(Assign, "foo"),
        tok!(StartExpansion, "bar"),
        tok!(ExpansionOperator, "-"),
    ],
    |r| assert!(matches!(r, Err(ParseError::Eof))),
)]
#[case::unexpected_in_expansion(
    vec![
        tok!(Assign, "foo"),
        tok!(StartExpansion, "bar"),
        tok!(ExpansionOperator, "-"),
        tok!(Assign, "baz"),
    ],
    |r| assert!(matches!(r, Err(ParseError::Unexpected(_)))),
)]
fn parse_errors(
    #[case] input: Vec<TokenizerResult>,
    #[case] assert: impl Fn(ParseResult<Vec<Assignment>>),
) {
    let res = Parser::new(input.into_iter()).parse();
    assert(res);
}
