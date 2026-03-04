use thiserror::Error;

#[derive(Debug, Error)]
pub enum XsnapError {
    #[error("Config not found: {path}")]
    ConfigNotFound { path: String },

    #[error("Invalid config: {message}")]
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

    #[error("Server not reachable after {attempts} attempts: {url}")]
    ServerNotReady { url: String, attempts: u32 },
}

impl XsnapError {
    pub fn exit_code(&self) -> i32 {
        match self {
            XsnapError::ConfigNotFound { .. } => 2,
            XsnapError::ConfigInvalid { .. } => 2,
            XsnapError::DuplicateTestName { .. } => 2,
            XsnapError::UndefinedFunction { .. } => 2,
            XsnapError::BrowserDownloadFailed { .. } => 3,
            XsnapError::BrowserLaunchFailed { .. } => 3,
            XsnapError::CdpError { .. } => 4,
            XsnapError::NavigationFailed { .. } => 4,
            XsnapError::ScreenshotFailed { .. } => 4,
            XsnapError::DiffFailed { .. } => 4,
            XsnapError::ServerNotReady { .. } => 5,
        }
    }
}
