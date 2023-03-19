use std::collections::HashMap;

use thiserror::Error;

use crate::{
    env::EnvProvider,
    parser::ast::{Assignment, Expansion, Expression, Operator},
};

#[cfg(test)]
mod tests;

type Scope = HashMap<String, String>;

#[derive(Debug, Error)]
pub enum EvaluationError {
    #[error("Undefined variable ${0} {1}")]
    UndefinedVariable(String, String),
    #[error("Missing required value for variable ${0} {1}")]
    EmptyValue(String, String),
}

type EvaluationResult<T> = Result<T, EvaluationError>;

#[derive(Debug)]
struct Evaluator<T>
where
    T: EnvProvider,
{
    env: T,
    scope: Scope,
    override_env: bool,
}

impl<T> Evaluator<T>
where
    T: EnvProvider,
{
    pub fn new(env: T, override_env: bool) -> Self {
        Self {
            env,
            override_env,
            scope: HashMap::new(),
        }
    }

    pub fn evaluate(&mut self, ast: Vec<Assignment>) -> EvaluationResult<()> {
        for node in ast {
            self.evaluate_assignment(node)?;
        }
        Ok(())
    }

    pub fn into_env(self) -> HashMap<String, String> {
        self.scope
    }

    pub fn scope(&self) -> Scope {
        self.scope.clone()
    }

    fn evaluate_assignment(&mut self, node: Assignment) -> EvaluationResult<()> {
        let name = node.name;
        let value = if self.override_env {
            self.evaluate_expression(node.value)?
        } else {
            if let Some(v) = self.env.get_var(&name) {
                v
            } else {
                self.evaluate_expression(node.value)?
            }
        };
        self.scope.insert(name, value);
        Ok(())
    }

    fn evaluate_expression(&mut self, nodes: Vec<Expression>) -> EvaluationResult<String> {
        let mut result = String::with_capacity(64);
        for node in nodes {
            let value = match node {
                Expression::Characters(chars) => chars,
                Expression::Expansion(expr) => self.evaluate_expansion(expr)?,
            };
            result.push_str(&value);
        }
        Ok(result)
    }

    fn evaluate_expansion(&mut self, expr: Expansion) -> EvaluationResult<String> {
        let value = self.resolve(&expr.name);
        let result = match expr.operator {
            Operator::IfUnset => match value {
                None => self.evaluate_expression(expr.rhs)?,
                _ => value.unwrap(),
            },
            Operator::IfUnsetOrNull => match value.as_deref() {
                None | Some("") => self.evaluate_expression(expr.rhs)?,
                _ => value.unwrap(),
            },
            Operator::IfSet => match value {
                None => "".to_owned(),
                _ => self.evaluate_expression(expr.rhs)?,
            },
            Operator::IfSetAndNotNull => match value.as_deref() {
                None | Some("") => "".to_owned(),
                _ => self.evaluate_expression(expr.rhs)?,
            },
            Operator::AssignIfUnset => match value {
                None => self.assign_op(expr.name, expr.rhs)?,
                _ => value.unwrap(),
            },
            Operator::AssignIfUnsetOrNull => match value.as_deref() {
                None | Some("") => self.assign_op(expr.name, expr.rhs)?,
                _ => value.unwrap(),
            },
            Operator::ErrorIfUnset => match value {
                None => self.error_op(expr.name, expr.rhs, false)?,
                _ => value.unwrap(),
            },
            Operator::ErrorIfUnsetOrNull => match value.as_deref() {
                None => self.error_op(expr.name, expr.rhs, false)?,
                Some("") => self.error_op(expr.name, expr.rhs, true)?,
                _ => value.unwrap(),
            },
        };
        Ok(result)
    }

    fn resolve(&self, name: &str) -> Option<String> {
        if self.override_env {
            self.scope
                .get(name)
                .map(ToOwned::to_owned)
                .or_else(|| self.env.get_var(name))
        } else {
            self.env
                .get_var(name)
                .or_else(|| self.scope.get(name).map(ToOwned::to_owned))
        }
    }

    fn assign_op(&mut self, name: String, expr: Vec<Expression>) -> EvaluationResult<String> {
        let value = self.evaluate_expression(expr)?;
        self.scope.insert(name, value.clone());
        Ok(value)
    }

    fn error_op(
        &mut self,
        name: String,
        expr: Vec<Expression>,
        require_value: bool,
    ) -> EvaluationResult<String> {
        let message = self.evaluate_expression(expr)?;
        if require_value {
            Err(EvaluationError::EmptyValue(name, message))
        } else {
            Err(EvaluationError::UndefinedVariable(name, message))
        }
    }
}
