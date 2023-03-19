#![feature(test)]

use thiserror::Error;

use evaluator::EvaluationError;
use parser::ParseError;

mod env;
mod evaluator;
mod parser;
#[cfg(test)]
mod test_utils;
mod tokenizer;

#[derive(Debug, Error)]
pub enum PotenvError {
    #[error(transparent)]
    ParseError(#[from] ParseError),
    #[error(transparent)]
    EvaluationError(#[from] EvaluationError),
}
