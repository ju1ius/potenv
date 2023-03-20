extern crate test;

use std::path::PathBuf;

use test::Bencher;

use super::{parse, Parser};
use crate::{
    parser::ParseError,
    tokenizer::{token::Token, TokenizerResult},
};

#[bench]
fn bench_parse(b: &mut Bencher) {
    let p = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("benches/resources/big.env");
    let input = std::fs::read_to_string(&p).unwrap();
    // let name = p.to_string_lossy().to_string();
    b.iter(|| parse(&input, None).unwrap())
}

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
