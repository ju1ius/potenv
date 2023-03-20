#[cfg(test)]
mod tests;

use std::collections::HashMap;

/// Trait for environment variable providers.
pub trait EnvProvider {
    fn get_var(&self, name: &str) -> Option<String>;

    fn set_var(&mut self, name: &str, value: &str);
}

/// An environment variable provider that reads from and writes to
/// the current process environment.
#[derive(Debug, Clone, Copy)]
pub struct ProcessEnvProvider;

impl EnvProvider for ProcessEnvProvider {
    fn get_var(&self, name: &str) -> Option<String> {
        std::env::var_os(name).map(|v| v.to_string_lossy().into())
    }

    fn set_var(&mut self, name: &str, value: &str) {
        std::env::set_var(name, value)
    }
}

/// An environment variable provider that reads from and writes to a HashMap.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct HashMapProvider(HashMap<String, String>);

impl EnvProvider for HashMapProvider {
    fn get_var(&self, name: &str) -> Option<String> {
        self.0.get(name).map(ToOwned::to_owned)
    }

    fn set_var(&mut self, name: &str, value: &str) {
        self.0.insert(name.to_owned(), value.to_owned());
    }
}

impl From<HashMap<String, String>> for HashMapProvider {
    fn from(value: HashMap<String, String>) -> Self {
        Self(value)
    }
}

impl<K, V> FromIterator<(K, V)> for HashMapProvider
where
    K: AsRef<str>,
    V: AsRef<str>,
{
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        Self(HashMap::from_iter(iter.into_iter().map(|(k, v)| {
            (k.as_ref().to_owned(), v.as_ref().to_owned())
        })))
    }
}
