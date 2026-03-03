use std::fs;
use std::io::Write;
use tempfile::NamedTempFile;
use xsnap::config::test::{discover_test_files, load_test_file};

#[test]
fn test_load_test_file() {
    let json = r#"{
        "tests": [
            {
                "name": "home page",
                "url": "/",
                "threshold": 5
            },
            {
                "name": "about page",
                "url": "/about"
            }
        ]
    }"#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(json.as_bytes()).unwrap();
    file.flush().unwrap();

    let tests = load_test_file(file.path()).unwrap();

    assert_eq!(tests.len(), 2);
    assert_eq!(tests[0].name, "home page");
    assert_eq!(tests[0].url, "/");
    assert_eq!(tests[0].threshold, Some(5));
    assert_eq!(tests[1].name, "about page");
    assert_eq!(tests[1].url, "/about");
    assert!(tests[1].threshold.is_none());
}

#[test]
fn test_discover_test_files() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let tests_dir = tmp_dir.path().join("tests");
    let nested_dir = tests_dir.join("nested");
    fs::create_dir_all(&nested_dir).unwrap();

    // Create matching files
    fs::write(tests_dir.join("home.xsnap.json"), r#"{"tests":[]}"#).unwrap();
    fs::write(nested_dir.join("dashboard.xsnap.json"), r#"{"tests":[]}"#).unwrap();

    // Create non-matching files
    fs::write(tests_dir.join("readme.md"), "# readme").unwrap();
    fs::write(tests_dir.join("config.json"), "{}").unwrap();

    let pattern = "tests/**/*.xsnap.json";
    let paths = discover_test_files(tmp_dir.path(), pattern, &[]).unwrap();

    assert_eq!(paths.len(), 2);

    let filenames: Vec<String> = paths
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
        .collect();

    assert!(filenames.contains(&"home.xsnap.json".to_string()));
    assert!(filenames.contains(&"dashboard.xsnap.json".to_string()));
}
