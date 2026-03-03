use thiserror::Error;

#[derive(Debug, Error)]
pub enum XsnapError {
    #[error("Config not found: {path}")]
    ConfigNotFound { path: String },

    #[error("Config invalid: {message}")]
    ConfigInvalid { message: String },

    #[error("Duplicate test name: {name}")]
    DuplicateTestName { name: String },

    #[error("Undefined function: {name}")]
    UndefinedFunction { name: String },

    #[error("Browser download failed: {message}")]
    BrowserDownloadFailed { message: String },

    #[error("Browser launch failed: {message}")]
    BrowserLaunchFailed { message: String },

    #[error("CDP error: {message}")]
    CdpError { message: String },

    #[error("Navigation failed for {url}: {message}")]
    NavigationFailed { url: String, message: String },

    #[error("Screenshot failed: {message}")]
    ScreenshotFailed { message: String },

    #[error("Diff failed: {message}")]
    DiffFailed { message: String },
}

impl XsnapError {
    pub fn exit_code(&self) -> i32 {
        match self {
            XsnapError::ConfigNotFound { .. } => 1,
            XsnapError::ConfigInvalid { .. } => 1,
            XsnapError::DuplicateTestName { .. } => 1,
            XsnapError::UndefinedFunction { .. } => 1,
            XsnapError::BrowserDownloadFailed { .. } => 2,
            XsnapError::BrowserLaunchFailed { .. } => 2,
            XsnapError::CdpError { .. } => 2,
            XsnapError::NavigationFailed { .. } => 3,
            XsnapError::ScreenshotFailed { .. } => 3,
            XsnapError::DiffFailed { .. } => 4,
        }
    }
}
