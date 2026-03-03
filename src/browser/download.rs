use std::path::{Path, PathBuf};

use crate::error::XsnapError;

/// URL for Chrome for Testing latest version metadata.
#[allow(dead_code)]
const CHROME_LATEST_URL: &str = "https://googlechromelabs.github.io/chrome-for-testing/last-known-good-versions-with-downloads.json";

/// Returns the platform-specific cache directory for storing Chromium downloads.
///
/// On Linux: `~/.cache/xsnap/chromium`
/// On macOS: `~/Library/Caches/xsnap/chromium`
/// On Windows: `%LOCALAPPDATA%/xsnap/chromium`
pub fn cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from(".cache"))
        .join("xsnap")
        .join("chromium")
}

/// Resolves a chromium version string.
///
/// If the version is "auto", it resolves to "latest".
/// Otherwise, the version string is passed through as-is.
pub fn resolve_chromium_version(version: &str) -> Result<String, XsnapError> {
    if version == "auto" {
        Ok("latest".into())
    } else {
        Ok(version.to_string())
    }
}

/// Constructs the download URL for Chrome for Testing based on version and platform.
pub fn get_download_url(version: &str, platform: &str) -> String {
    let platform_key = match platform {
        "linux" => "linux64",
        "macos" | "darwin" => "mac-x64",
        "macos-arm" => "mac-arm64",
        "windows" => "win64",
        _ => "linux64",
    };
    format!(
        "https://storage.googleapis.com/chrome-for-testing-public/{}/{}/chrome-{}.zip",
        version, platform_key, platform_key
    )
}

/// Ensures a Chromium binary is available, downloading it if necessary.
///
/// Returns the path to the chrome binary.
pub async fn ensure_chromium(version: &str) -> Result<PathBuf, XsnapError> {
    let resolved = resolve_chromium_version(version)?;
    let cache = cache_dir().join(&resolved);

    if cache.exists() {
        return find_chrome_binary(&cache);
    }

    let platform = current_platform();
    let url = get_download_url(&resolved, &platform);
    download_and_extract(&url, &cache).await?;
    find_chrome_binary(&cache)
}

/// Detects the current platform string for download URL construction.
fn current_platform() -> String {
    if cfg!(target_os = "linux") {
        if cfg!(target_arch = "aarch64") {
            "linux-arm".into()
        } else {
            "linux".into()
        }
    } else if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            "macos-arm".into()
        } else {
            "macos".into()
        }
    } else if cfg!(target_os = "windows") {
        "windows".into()
    } else {
        "linux".into()
    }
}

/// Searches recursively for the chrome binary inside a directory.
fn find_chrome_binary(dir: &Path) -> Result<PathBuf, XsnapError> {
    let binary_name = if cfg!(target_os = "windows") {
        "chrome.exe"
    } else {
        "chrome"
    };
    find_file_recursive(dir, binary_name).ok_or_else(|| XsnapError::BrowserLaunchFailed {
        message: format!("Chrome binary not found in {}", dir.display()),
    })
}

/// Recursively searches a directory for a file with the given name.
fn find_file_recursive(dir: &std::path::Path, name: &str) -> Option<PathBuf> {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(found) = find_file_recursive(&path, name) {
                    return Some(found);
                }
            } else if entry.file_name().to_string_lossy() == name {
                return Some(path);
            }
        }
    }
    None
}

/// Downloads a zip archive from the given URL and extracts it to the target directory.
async fn download_and_extract(url: &str, target: &PathBuf) -> Result<(), XsnapError> {
    std::fs::create_dir_all(target).map_err(|e| XsnapError::BrowserDownloadFailed {
        message: format!("Failed to create cache dir: {}", e),
    })?;

    let response = reqwest::get(url)
        .await
        .map_err(|e| XsnapError::BrowserDownloadFailed {
            message: format!("Download failed: {}", e),
        })?;

    if !response.status().is_success() {
        return Err(XsnapError::BrowserDownloadFailed {
            message: format!("HTTP {}: {}", response.status(), url),
        });
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| XsnapError::BrowserDownloadFailed {
            message: format!("Failed to read response: {}", e),
        })?;

    let cursor = std::io::Cursor::new(bytes);
    let mut archive =
        zip::ZipArchive::new(cursor).map_err(|e| XsnapError::BrowserDownloadFailed {
            message: format!("Failed to open zip: {}", e),
        })?;
    archive
        .extract(target)
        .map_err(|e| XsnapError::BrowserDownloadFailed {
            message: format!("Failed to extract: {}", e),
        })?;

    // Make the chrome binary executable on unix systems.
    #[cfg(unix)]
    {
        if let Ok(binary) = find_chrome_binary(&target.to_path_buf()) {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o755));
        }
    }

    Ok(())
}
