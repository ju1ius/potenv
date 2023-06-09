#[cfg(test)]
mod tests;

use std::collections::HashMap;

/// Trait for environment variable providers.
pub trait EnvProvider {
    fn var(&self, name: &str) -> Option<String>;

    fn set_var(&mut self, name: &str, value: &str);
}

/// An environment variable provider that reads from and writes to
/// the current process environment.
#[derive(Debug, Clone, Copy)]
pub struct ProcessEnvProvider;

impl EnvProvider for ProcessEnvProvider {
    fn var(&self, name: &str) -> Option<String> {
        std::env::var_os(name).map(|v| v.to_string_lossy().into())
    }

    fn set_var(&mut self, name: &str, value: &str) {
        std::env::set_var(name, value)
    }
}

impl EnvProvider for HashMap<String, String> {
    fn var(&self, name: &str) -> Option<String> {
        self.get(name).map(ToOwned::to_owned)
    }

    fn set_var(&mut self, name: &str, value: &str) {
        self.insert(name.to_owned(), value.to_owned());
    }
}
