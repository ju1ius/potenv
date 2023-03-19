use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use serde::de::DeserializeOwned;
use walkdir::WalkDir;

pub type AnyRes<T> = Result<T, Box<dyn std::error::Error>>;

pub fn load_spec_file<T: DeserializeOwned>(path: &PathBuf) -> AnyRes<Vec<T>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let data: Vec<T> = serde_json::from_reader(reader)?;
    Ok(data)
}

pub fn collect_spec_files(dir: impl AsRef<Path>) -> Vec<PathBuf> {
    let project_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("Could not determine project directory.");
    let root = PathBuf::from(project_dir)
        .join("dotenv-spec/tests")
        .join(dir);
    let mut paths: Vec<_> = WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_name().to_string_lossy().ends_with(".json"))
        .map(|e| e.path().to_path_buf())
        .collect();
    paths.sort();
    paths
}
