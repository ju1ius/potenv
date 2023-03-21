#![feature(test)]
#![doc = include_str!("../README.md")]

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

/// Loads environment variables from the specified files,
/// and exports them into the current process's environment.
pub fn load<I>(files: I) -> PotenvResult<impl Iterator<Item = (String, String)>>
where
    I: IntoIterator,
    I::Item: AsRef<Path>,
{
    Potenv::default().load(files)
}

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

    /// Sets whether variables in dotenv files should override those from the environment provider.
    /// Defaults to false.
    pub fn override_env(mut self, override_env: bool) -> Self {
        self.override_env = override_env;
        self
    }

    /// Loads environment variables from the specified files,
    /// and exports them to the current process's environment.
    pub fn load<I>(&mut self, files: I) -> PotenvResult<impl Iterator<Item = (String, String)>>
    where
        I: IntoIterator,
        I::Item: AsRef<Path>,
    {
        let scope = self.eval(files)?;
        for (name, value) in scope.iter() {
            if self.override_env || self.env.var(name).is_none() {
                self.env.set_var(name, value);
            }
        }
        Ok(scope.into_iter())
    }

    /// Loads environment variables from the specified files
    /// without exporting them to the current process's environment.
    pub fn evaluate<I>(&self, files: I) -> PotenvResult<impl Iterator<Item = (String, String)>>
    where
        I: IntoIterator,
        I::Item: AsRef<Path>,
    {
        Ok(self.eval(files)?.into_iter())
    }

    fn eval<I>(&self, files: I) -> PotenvResult<Scope>
    where
        I: IntoIterator,
        I::Item: AsRef<Path>,
    {
        let mut eval = Evaluator::new(&self.env, self.override_env);
        for file in files {
            let path = file.as_ref();
            let input = std::fs::read_to_string(path)?;
            let ast = parse(&input, Some(path.to_path_buf()))?;
            eval.evaluate(ast)?;
        }
        Ok(eval.into_scope())
    }
}
