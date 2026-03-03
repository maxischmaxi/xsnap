use std::path::{Path, PathBuf};

use crate::config::types::{TestConfig, TestFile};
use crate::error::XsnapError;

pub fn load_test_file(path: &Path) -> Result<Vec<TestConfig>, XsnapError> {
    let content = std::fs::read_to_string(path).map_err(|_| XsnapError::ConfigNotFound {
        path: path.display().to_string(),
    })?;

    let test_file: TestFile =
        serde_json::from_str(&content).map_err(|e| XsnapError::ConfigInvalid {
            message: format!("{}: {}", path.display(), e),
        })?;

    Ok(test_file.tests)
}

pub fn discover_test_files(
    base_dir: &Path,
    pattern: &str,
    ignore_patterns: &[String],
) -> Result<Vec<PathBuf>, XsnapError> {
    let full_pattern = base_dir.join(pattern).display().to_string();
    let paths: Vec<PathBuf> = glob::glob(&full_pattern)
        .map_err(|e| XsnapError::ConfigInvalid {
            message: format!("Invalid test pattern '{}': {}", pattern, e),
        })?
        .filter_map(|entry| entry.ok())
        .filter(|path| {
            let path_str = path.display().to_string();
            !ignore_patterns
                .iter()
                .any(|pattern| path_str.contains(pattern))
        })
        .collect();

    Ok(paths)
}
