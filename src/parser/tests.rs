#![allow(unused_macros)]

use super::Parser;
use crate::{
    parser::ParseError,
    tokenizer::{token::Token, TokenizerResult},
};

fn tokenize(tokens: Vec<Token>) -> impl Iterator<Item = TokenizerResult> {
    tokens.into_iter().map(|t| Ok(t))
}

macro_rules! tok {
    ($k:ident, $v:literal, $l:literal, $c:literal) => {
        Ok(Token::new(
            TokenKind::$k,
            $v.to_string(),
            Position::new($l, $c),
        ))
    };
}

macro_rules! terr {
    ($e:expr) => {
        Err(ParseError::$e)
    };
}

#[test]
fn eof_error_with_empty_iterator() {
    let res = Parser::new(tokenize(vec![])).parse();
    let err = res.expect_err("Sould error.");
    assert_eq!(ParseError::Eof, err);
}
