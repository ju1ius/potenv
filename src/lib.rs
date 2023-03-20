#![feature(test)]

use std::path::Path;

use env::{EnvProvider, ProcessEnvProvider};
use evaluator::{EvaluationError, Evaluator, Scope};
use parser::{parse, ParseError};
use thiserror::Error;

pub mod env;
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
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

type PotenvResult<T> = Result<T, PotenvError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Potenv<T>
where
    T: EnvProvider,
{
    env: T,
    override_env: bool,
}

impl Default for Potenv<ProcessEnvProvider> {
    fn default() -> Self {
        Self::new(ProcessEnvProvider, false)
    }
}

impl<T> Potenv<T>
where
    T: EnvProvider,
{
    pub fn new(env: T, override_env: bool) -> Self {
        Self { env, override_env }
    }

    pub fn override_env(mut self, override_env: bool) -> Self {
        self.override_env = override_env;
        self
    }

    pub fn load<I, P>(&mut self, files: I) -> PotenvResult<Scope>
    where
        P: AsRef<Path>,
        I: IntoIterator<Item = P>,
    {
        let scope = self.evaluate(files)?;
        for (name, value) in scope.iter() {
            if self.override_env || self.env.get_var(name).is_none() {
                self.env.set_var(name, value);
            }
        }
        Ok(scope)
    }

    pub fn evaluate<I, P>(&self, files: I) -> PotenvResult<Scope>
    where
        P: AsRef<Path>,
        I: IntoIterator<Item = P>,
    {
        let mut eval = Evaluator::new(&self.env, self.override_env);
        for file in files {
            let path = file.as_ref();
            let input = std::fs::read_to_string(path)?;
            let ast = parse(&input, Some(path.to_path_buf()))?;
            eval.evaluate(ast)?;
        }
        Ok(eval.into_env())
    }
}
