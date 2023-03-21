# potenv

A Rust implementation of the [POSIX-compliant dotenv file format specification](https://github.com/php-xdg/dotenv-spec).

## Usage

Load environment variables from a `.env` file in the current working directory:

```rust
potenv::load(vec![".env"]).expect("Failed to load .env file.");
```

For convenience, an iterator over the loaded variables is returned:

```rust
let vars = potenv::load(vec![".env"]).unwrap();
for (name, value) in vars {
  assert_eq!(value, std::env::var(name).unwrap());
}
```

If you just want to evaluate the dotenv files without loading them into the environment, use the following:

```rust
use potenv::Potenv;

let vars = Potenv::default()
  .evaluate(vec![".env"])
  .unwrap();
```

By default, environment variables take precedence over variables defined in a dotenv file.

When this is not the desired behaviour, you can use the following:

```rust
use potenv::Potenv;

let vars = Potenv::default()
  .override_env(true)
  .load(vec![".env"])
  .unwrap();
```

If you don't want to read from and/or write to the process environment,
you can implement the [env::EnvProvider] trait.

For example, this is how to frobnicate all variables:

```rust
use potenv::{Potenv, env::EnvProvider};

pub struct Frobnicator;

impl EnvProvider for Frobnicator {
  fn var(&self, name: &str) -> Option<String> {
    Some("frobnicated".into())
  }
  fn set_var(&mut self, name: &str, value: &str) {}
}

let vars = Potenv::new(Frobnicator, false)
  .evaluate(vec![".env"])
  .unwrap();
for (name, value) in vars {
  assert_eq!("frobnicated", value);
}
```
