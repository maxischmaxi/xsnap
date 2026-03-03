use std::path::Path;

use json_comments::StripComments;

use crate::config::types::GlobalConfig;
use crate::error::XsnapError;

pub fn load_global_config(path: &Path) -> Result<GlobalConfig, XsnapError> {
    let content = std::fs::read(path).map_err(|e| XsnapError::ConfigNotFound {
        path: format!("{}: {}", path.display(), e),
    })?;

    let stripped = StripComments::new(content.as_slice());
    let config: GlobalConfig =
        serde_json::from_reader(stripped).map_err(|e| XsnapError::ConfigInvalid {
            message: format!("{}: {}", path.display(), e),
        })?;

    Ok(config)
}
