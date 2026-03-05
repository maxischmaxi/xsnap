use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::page::Page;
use futures::StreamExt;
use tokio::sync::{RwLock, Semaphore};

use crate::config::types::BrowserConfig as XsnapBrowserConfig;
use crate::error::XsnapError;

/// Monotonic counter to ensure unique data directories even within the same millisecond.
static INSTANCE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Internal state for a running browser instance and its event handler.
struct BrowserInstance {
    browser: Browser,
    handler: tokio::task::JoinHandle<()>,
    data_dir: PathBuf,
}

/// A pool that manages a single browser instance with semaphore-based
/// parallelism control for page creation.
pub struct BrowserPool {
    instance: RwLock<BrowserInstance>,
    semaphore: Arc<Semaphore>,
}

impl BrowserPool {
    /// Creates a new `BrowserPool` by launching a Chromium instance.
    ///
    /// - `chrome_path`: Path to the Chrome/Chromium executable.
    /// - `semaphore`: Shared semaphore controlling total parallelism across all pools.
    /// - `browser_config`: Optional extra browser configuration (args, env).
    pub async fn new(
        chrome_path: &Path,
        semaphore: Arc<Semaphore>,
        browser_config: Option<&XsnapBrowserConfig>,
    ) -> Result<Self, XsnapError> {
        let instance = Self::launch_browser(chrome_path, browser_config).await?;

        Ok(Self {
            instance: RwLock::new(instance),
            semaphore,
        })
    }

    /// Launch a fresh browser instance with a unique user-data-dir.
    async fn launch_browser(
        chrome_path: &Path,
        browser_config: Option<&XsnapBrowserConfig>,
    ) -> Result<BrowserInstance, XsnapError> {
        // Clean up stale default data dir from older versions / previous crashes.
        let default_lock = Path::new("/tmp/chromiumoxide-runner/SingletonLock");
        if default_lock.exists() {
            let _ = std::fs::remove_file(default_lock);
        }

        // Create a unique data directory per instance to avoid SingletonLock conflicts.
        let counter = INSTANCE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let data_dir = std::env::temp_dir().join(format!(
            "xsnap-browser-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
            counter,
        ));
        std::fs::create_dir_all(&data_dir).map_err(|e| XsnapError::BrowserLaunchFailed {
            message: format!("Failed to create browser data dir: {}", e),
        })?;

        let mut builder = BrowserConfig::builder()
            .chrome_executable(chrome_path)
            .user_data_dir(&data_dir)
            .new_headless_mode()
            .no_sandbox()
            .arg("--disable-gpu")
            .arg("--no-first-run")
            .arg("--no-default-browser-check");

        if let Some(config) = browser_config {
            for arg in &config.args {
                builder = builder.arg(arg.as_str());
            }
            for (key, val) in &config.env {
                builder = builder.env(key, val);
            }
        }

        let config = builder
            .build()
            .map_err(|e| XsnapError::BrowserLaunchFailed {
                message: format!("Invalid browser config: {}", e),
            })?;

        let (browser, mut handler) =
            Browser::launch(config)
                .await
                .map_err(|e| XsnapError::BrowserLaunchFailed {
                    message: format!("Failed to launch browser: {}", e),
                })?;

        let handle = tokio::spawn(async move { while let Some(_event) = handler.next().await {} });

        Ok(BrowserInstance {
            browser,
            handler: handle,
            data_dir,
        })
    }

    /// Gracefully shuts down the browser pool and cleans up its data directory.
    pub async fn shutdown(self) {
        let instance = self.instance.into_inner();
        let data_dir = instance.data_dir.clone();
        drop(instance.browser);
        let _ = instance.handler.await;
        let _ = std::fs::remove_dir_all(&data_dir);
    }

    /// Acquires a new browser page from the pool.
    ///
    /// This will wait until a semaphore permit is available, then creates a
    /// new page on the current browser instance.
    pub async fn acquire(&self) -> Result<(Page, tokio::sync::OwnedSemaphorePermit), XsnapError> {
        let permit = self.semaphore.clone().acquire_owned().await.map_err(|_| {
            XsnapError::BrowserLaunchFailed {
                message: "Pool semaphore closed".into(),
            }
        })?;

        let instance = self.instance.read().await;
        let page = instance
            .browser
            .new_page("about:blank")
            .await
            .map_err(|e| XsnapError::CdpError {
                message: format!("Failed to create page: {}", e),
            })?;

        Ok((page, permit))
    }
}
