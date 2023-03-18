#![feature(test)]

use thiserror::Error;

use evaluator::EvaluationError;
use parser::ParseError;

mod evaluator;
mod parser;
mod tokenizer;

#[derive(Debug, Error)]
pub enum PotenvError {
    #[error(transparent)]
    ParseError(#[from] ParseError),
    #[error(transparent)]
    EvaluationError(#[from] EvaluationError),
}
