use std::sync::Arc;

use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::page::Page;
use futures::StreamExt;
use tokio::sync::Semaphore;

use crate::config::types::BrowserConfig as XsnapBrowserConfig;
use crate::error::XsnapError;

/// A pool that manages a single browser instance with semaphore-based
/// parallelism control for page creation.
pub struct BrowserPool {
    browser: Arc<Browser>,
    semaphore: Arc<Semaphore>,
    _handler: tokio::task::JoinHandle<()>,
}

impl BrowserPool {
    /// Creates a new `BrowserPool` by launching a Chromium instance.
    ///
    /// - `chrome_path`: Path to the Chrome/Chromium executable.
    /// - `parallelism`: Maximum number of concurrent pages.
    /// - `global_browser_config`: Optional extra browser configuration (args, env).
    pub async fn new(
        chrome_path: &std::path::Path,
        parallelism: usize,
        global_browser_config: Option<&XsnapBrowserConfig>,
    ) -> Result<Self, XsnapError> {
        let mut builder = BrowserConfig::builder()
            .chrome_executable(chrome_path)
            .new_headless_mode()
            .no_sandbox()
            .arg("--disable-gpu")
            .arg("--no-first-run")
            .arg("--no-default-browser-check");

        if let Some(config) = global_browser_config {
            for arg in &config.args {
                builder = builder.arg(arg);
            }
            for (key, val) in &config.env {
                builder = builder.env(key, val);
            }
        }

        let browser_config = builder.build().map_err(|e| XsnapError::BrowserLaunchFailed {
            message: format!("Invalid browser config: {}", e),
        })?;

        let (browser, mut handler) =
            Browser::launch(browser_config)
                .await
                .map_err(|e| XsnapError::BrowserLaunchFailed {
                    message: format!("Failed to launch browser: {}", e),
                })?;

        let handle = tokio::spawn(async move {
            while let Some(_event) = handler.next().await {}
        });

        Ok(Self {
            browser: Arc::new(browser),
            semaphore: Arc::new(Semaphore::new(parallelism)),
            _handler: handle,
        })
    }

    /// Acquires a new browser page from the pool.
    ///
    /// This will wait until a semaphore permit is available (i.e., the number
    /// of concurrent pages is below the parallelism limit). Returns the page
    /// and the owned permit; the permit should be held for the duration of
    /// page usage and dropped when done.
    pub async fn acquire(&self) -> Result<(Page, tokio::sync::OwnedSemaphorePermit), XsnapError> {
        let permit = self
            .semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| XsnapError::BrowserLaunchFailed {
                message: "Pool semaphore closed".into(),
            })?;

        let page = self
            .browser
            .new_page("about:blank")
            .await
            .map_err(|e| XsnapError::CdpError {
                message: format!("Failed to create page: {}", e),
            })?;

        Ok((page, permit))
    }
}
