use std::path::{Path, PathBuf};

use crate::config::types::TestConfig;
use crate::error::XsnapError;

pub fn load_test_file(path: &Path) -> Result<Vec<TestConfig>, XsnapError> {
    let content = std::fs::read_to_string(path).map_err(|_| XsnapError::ConfigNotFound {
        path: path.display().to_string(),
    })?;

    let tests: Vec<TestConfig> = serde_json::from_str(&content).map_err(|e| {
        XsnapError::ConfigInvalid {
            message: format!("{}: {}", path.display(), e),
        }
    })?;

    Ok(tests)
}

pub fn discover_test_files(
    base_dir: &Path,
    pattern: &str,
    _ignore_patterns: &[String],
) -> Result<Vec<PathBuf>, XsnapError> {
    let full_pattern = base_dir.join(pattern).display().to_string();
    let paths: Vec<PathBuf> = glob::glob(&full_pattern)
        .map_err(|e| XsnapError::ConfigInvalid {
            message: format!("Invalid test pattern '{}': {}", pattern, e),
        })?
        .filter_map(|entry| entry.ok())
        .collect();

    Ok(paths)
}
