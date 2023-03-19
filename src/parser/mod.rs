pub mod ast;
#[cfg(test)]
mod tests;

use std::iter::Peekable;

use thiserror::Error;

use crate::tokenizer::{
    err::SyntaxError,
    token::{Token, TokenKind},
    Tokenizer, TokenizerResult,
};

use self::ast::*;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ParseError {
    #[error("Unexpected end of input")]
    Eof,
    #[error("Unexpected token {0:?}")]
    Unexpected(Token),
    #[error("Unknown expansion operator '{0}'")]
    UnknownOperator(String),
    #[error(transparent)]
    Syntax(#[from] SyntaxError),
}

pub type ParseResult<T> = Result<T, ParseError>;

pub fn parse(input: &str, filename: Option<&str>) -> ParseResult<Vec<Assignment>> {
    let filename = filename.map(ToString::to_string);
    let tokenizer = Tokenizer::new(input.chars(), filename);
    Parser::new(tokenizer).parse()
}

macro_rules! match_kind {
    ($($kind:ident)|+) => {
        $( Some(Ok(Token {kind: TokenKind::$kind, ..})) )|+
    };
}

pub struct Parser<I>
where
    I: Iterator<Item = TokenizerResult>,
{
    tokens: Peekable<I>,
}

impl<I> Parser<I>
where
    I: Iterator<Item = TokenizerResult>,
{
    pub fn new(tokenizer: I) -> Self {
        Self {
            tokens: tokenizer.peekable(),
        }
    }

    pub fn parse(&mut self) -> ParseResult<Vec<Assignment>> {
        let mut nodes = Vec::with_capacity(16);
        loop {
            match self.tokens.peek() {
                None => return Err(ParseError::Eof),
                Some(Err(_)) => return self.take_err(),
                match_kind!(Eof) => return Ok(nodes),
                match_kind!(Assign) => nodes.push(self.parse_assignment()?),
                Some(Ok(_)) => {
                    return Err(ParseError::Unexpected(self.take_cur()?));
                }
            };
        }
    }

    fn parse_assignment(&mut self) -> ParseResult<Assignment> {
        let name = self.take_cur()?.value;
        let value = self.parse_assignment_value()?;
        Ok(Assignment::new(name, value))
    }

    fn parse_assignment_value(&mut self) -> ParseResult<Vec<Expression>> {
        let mut nodes = Vec::new();
        loop {
            match self.tokens.peek() {
                None => return Err(ParseError::Eof),
                Some(Err(_)) => return self.take_err(),
                match_kind!(Eof | Assign) => return Ok(nodes),
                match_kind!(Characters) => {
                    nodes.push(Expression::Characters(self.take_cur()?.value));
                }
                match_kind!(SimpleExpansion) => {
                    nodes.push(Expression::Expansion(Expansion::new(
                        self.take_cur()?.value,
                        Operator::IfUnset,
                        vec![],
                    )));
                }
                match_kind!(StartExpansion) => {
                    let name = self.take_cur()?.value;
                    let operator = self.parse_operator()?;
                    let rhs = self.parse_expansion_value()?;
                    nodes.push(Expression::Expansion(Expansion::new(name, operator, rhs)));
                }
                Some(Ok(_)) => {
                    return Err(ParseError::Unexpected(self.take_cur()?));
                }
            };
        }
    }

    fn parse_expansion_value(&mut self) -> ParseResult<Vec<Expression>> {
        let mut nodes = Vec::new();
        loop {
            match self.tokens.peek() {
                None => return Err(ParseError::Eof),
                Some(Err(_)) => return self.take_err(),
                match_kind!(EndExpansion) => {
                    self.tokens.next();
                    return Ok(nodes);
                }
                match_kind!(Characters) => {
                    nodes.push(Expression::Characters(self.take_cur()?.value));
                }
                match_kind!(SimpleExpansion) => {
                    nodes.push(Expression::Expansion(Expansion::new(
                        self.take_cur()?.value,
                        Operator::IfUnset,
                        vec![],
                    )));
                }
                match_kind!(StartExpansion) => {
                    let name = self.take_cur()?.value;
                    let operator = self.parse_operator()?;
                    let rhs = self.parse_expansion_value()?;
                    nodes.push(Expression::Expansion(Expansion::new(name, operator, rhs)));
                }
                Some(Ok(_)) => {
                    return Err(ParseError::Unexpected(self.take_cur()?));
                }
            };
        }
    }

    fn parse_operator(&mut self) -> ParseResult<Operator> {
        let token = self.expect(TokenKind::ExpansionOperator)?;
        match token.value.as_str() {
            "-" => Ok(Operator::IfUnset),
            ":-" => Ok(Operator::IfUnsetOrNull),
            "=" => Ok(Operator::AssignIfUnset),
            ":=" => Ok(Operator::AssignIfUnsetOrNull),
            "+" => Ok(Operator::IfSet),
            ":+" => Ok(Operator::IfSetAndNotNull),
            "?" => Ok(Operator::ErrorIfUnset),
            ":?" => Ok(Operator::ErrorIfUnsetOrNull),
            op => Err(ParseError::UnknownOperator(op.to_owned())),
        }
    }

    fn expect(&mut self, kind: TokenKind) -> ParseResult<Token> {
        match self.tokens.next() {
            None => Err(ParseError::Eof),
            Some(Ok(token)) if token.kind == kind => Ok(token),
            Some(Ok(token)) => Err(ParseError::Unexpected(token)),
            Some(Err(e)) => Err(ParseError::Syntax(e)),
        }
    }

    fn take_cur(&mut self) -> ParseResult<Token> {
        Ok(self.tokens.next().unwrap()?)
    }

    fn take_err<T>(&mut self) -> ParseResult<T> {
        Err(ParseError::Syntax(self.tokens.next().unwrap().unwrap_err()))
    }
}
