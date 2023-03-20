use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use serde::Deserialize;

pub type AnyRes<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Deserialize)]
pub struct SuccesCase {
    pub desc: String,
    pub files: Vec<PathBuf>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(rename = "override", default)]
    pub override_env: bool,
    pub expected: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct ErrorCase {
    pub desc: String,
    pub files: Vec<PathBuf>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(rename = "override", default)]
    pub override_env: bool,
    pub error: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum TestCase {
    Success(SuccesCase),
    Error(ErrorCase),
}

pub fn load_test_cases(path: &str) -> AnyRes<Vec<TestCase>> {
    let file = File::open(get_resource_path(path)?)?;
    let reader = BufReader::new(file);
    let mut data: Vec<TestCase> = serde_json::from_reader(reader)?;
    for case in data.iter_mut() {
        match case {
            TestCase::Success(SuccesCase { files, .. })
            | TestCase::Error(ErrorCase { files, .. }) => {
                *files = files
                    .into_iter()
                    .map(|p| get_resource_path(&p).unwrap())
                    .collect();
            }
        }
    }
    Ok(data)
}

pub fn get_resource_path(path: impl AsRef<Path>) -> AnyRes<PathBuf> {
    let project_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    Ok(PathBuf::from(project_dir)
        .join("tests/resources")
        .join(path))
}
