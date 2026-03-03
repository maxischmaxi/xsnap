use xsnap::browser::download::{cache_dir, get_download_url, resolve_chromium_version};

#[test]
fn test_resolve_auto_version() {
    let result = resolve_chromium_version("auto").unwrap();
    assert_eq!(result, "latest");
}

#[test]
fn test_resolve_specific_version() {
    let result = resolve_chromium_version("120.0.6099.109").unwrap();
    assert_eq!(result, "120.0.6099.109");
}

#[test]
fn test_get_download_url_linux() {
    let url = get_download_url("120.0.6099.109", "linux");
    assert_eq!(
        url,
        "https://storage.googleapis.com/chrome-for-testing-public/120.0.6099.109/linux64/chrome-linux64.zip"
    );
}

#[test]
fn test_get_download_url_macos() {
    let url = get_download_url("120.0.6099.109", "macos");
    assert_eq!(
        url,
        "https://storage.googleapis.com/chrome-for-testing-public/120.0.6099.109/mac-x64/chrome-mac-x64.zip"
    );
}

#[test]
fn test_get_download_url_macos_arm() {
    let url = get_download_url("120.0.6099.109", "macos-arm");
    assert_eq!(
        url,
        "https://storage.googleapis.com/chrome-for-testing-public/120.0.6099.109/mac-arm64/chrome-mac-arm64.zip"
    );
}

#[test]
fn test_get_download_url_windows() {
    let url = get_download_url("120.0.6099.109", "windows");
    assert_eq!(
        url,
        "https://storage.googleapis.com/chrome-for-testing-public/120.0.6099.109/win64/chrome-win64.zip"
    );
}

#[test]
fn test_get_download_url_unknown_platform() {
    let url = get_download_url("120.0.6099.109", "freebsd");
    // Unknown platforms default to linux64.
    assert!(url.contains("linux64"));
}

#[test]
fn test_cache_dir() {
    let dir = cache_dir();
    let dir_str = dir.to_string_lossy();
    assert!(dir_str.contains("xsnap"));
    assert!(dir_str.contains("chromium"));
}
