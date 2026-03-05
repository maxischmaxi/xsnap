use std::io::Write;
use tempfile::NamedTempFile;
use xsnap::config::global::load_global_config;

#[test]
fn test_load_jsonc_with_comments() {
    let jsonc = r#"{
        // This is a line comment
        "baseUrl": "http://localhost:3000",
        /* This is a block comment */
        "threshold": 5,
        "baseDirectory": "my_snapshots/__base_images__",
        "diffDirectory": "my_snapshots/__diff__",
        "updatedDirectory": "my_snapshots/__updated__"
    }"#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(jsonc.as_bytes()).unwrap();
    file.flush().unwrap();

    let config = load_global_config(file.path()).unwrap();

    assert_eq!(config.base_url, "http://localhost:3000");
    assert_eq!(config.threshold, 5);
    assert_eq!(config.base_directory, "my_snapshots/__base_images__");
    assert_eq!(config.diff_directory, "my_snapshots/__diff__");
    assert_eq!(config.updated_directory, "my_snapshots/__updated__");
    // Defaults should still be applied
    assert!(config.full_screen);
    assert_eq!(config.retry, 1);
}

#[test]
fn test_load_config_not_found() {
    let path = std::path::Path::new("/tmp/nonexistent_xsnap_config_12345.json");
    let result = load_global_config(path);

    assert!(result.is_err());
    let err = result.unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("Config not found"), "Got: {}", msg);
}

#[test]
fn test_load_invalid_json() {
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(b"{ invalid json }").unwrap();
    file.flush().unwrap();

    let result = load_global_config(file.path());

    assert!(result.is_err());
    let err = result.unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("Invalid config"), "Got: {}", msg);
}
