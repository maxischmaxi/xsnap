use xsnap::config::types::*;
use xsnap::config::validate::validate_config;

fn minimal_global_config() -> GlobalConfig {
    serde_json::from_value(serde_json::json!({
        "baseUrl": "http://localhost:3000"
    }))
    .unwrap()
}

fn test_config(name: &str, url: &str) -> TestConfig {
    serde_json::from_value(serde_json::json!({
        "name": name,
        "url": url
    }))
    .unwrap()
}

#[test]
fn test_validate_duplicate_names() {
    let global = minimal_global_config();
    let tests = vec![
        test_config("home page", "/"),
        test_config("home page", "/home"),
    ];

    let result = validate_config(&global, &tests);

    assert!(result.is_err());
    let err = result.unwrap_err();
    let msg = format!("{}", err);
    assert!(
        msg.contains("Duplicate test name") && msg.contains("home page"),
        "Got: {}",
        msg
    );
}

#[test]
fn test_validate_undefined_function() {
    let global = minimal_global_config();

    let test_with_function: TestConfig = serde_json::from_value(serde_json::json!({
        "name": "dashboard",
        "url": "/dashboard",
        "actions": [
            { "action": "function", "name": "nonexistent_fn" }
        ]
    }))
    .unwrap();

    let tests = vec![test_with_function];

    let result = validate_config(&global, &tests);

    assert!(result.is_err());
    let err = result.unwrap_err();
    let msg = format!("{}", err);
    assert!(
        msg.contains("Undefined function") && msg.contains("nonexistent_fn"),
        "Got: {}",
        msg
    );
}

#[test]
fn test_validate_valid_config() {
    let mut global = minimal_global_config();
    global.functions.insert(
        "login".to_string(),
        vec![
            serde_json::from_value(serde_json::json!({
                "action": "click",
                "selector": "#login-btn"
            }))
            .unwrap(),
        ],
    );

    let test_with_function: TestConfig = serde_json::from_value(serde_json::json!({
        "name": "dashboard",
        "url": "/dashboard",
        "actions": [
            { "action": "function", "name": "login" },
            { "action": "wait", "timeout": 1000 }
        ]
    }))
    .unwrap();

    let tests = vec![
        test_config("home page", "/"),
        test_with_function,
    ];

    let result = validate_config(&global, &tests);
    assert!(result.is_ok());
}
