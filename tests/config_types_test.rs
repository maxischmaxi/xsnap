use serde_json::json;
use xsnap::config::types::*;

#[test]
fn test_deserialize_minimal_global_config() {
    let data = json!({
        "baseUrl": "http://localhost:3000"
    });

    let config: GlobalConfig = serde_json::from_value(data).unwrap();

    assert_eq!(config.base_url, "http://localhost:3000");
    assert!(config.browser.is_none());
    assert!(config.full_screen);
    assert_eq!(config.test_pattern, "tests/**/*.xsnap.json");
    assert!(config.ignore_patterns.is_empty());
    assert!(config.default_sizes.is_none());
    assert!(config.functions.is_empty());
    assert_eq!(config.snapshot_directory, "__snapshots__");
    assert_eq!(config.threshold, 0);
    assert_eq!(config.retry, 1);
    assert!(config.parallelism.is_none());
    assert_eq!(config.diff_pixel_color.r, 255);
    assert_eq!(config.diff_pixel_color.g, 0);
    assert_eq!(config.diff_pixel_color.b, 255);
    assert!(config.http_headers.is_empty());
    assert!(config.tests.is_empty());
}

#[test]
fn test_deserialize_full_global_config() {
    let data = json!({
        "baseUrl": "https://example.com",
        "browser": {
            "version": "120.0",
            "args": ["--no-sandbox", "--disable-gpu"],
            "env": { "DISPLAY": ":99" }
        },
        "fullScreen": false,
        "testPattern": "specs/**/*.spec.json",
        "ignorePatterns": ["**/skip-this/**"],
        "defaultSizes": [
            { "name": "desktop", "width": 1920, "height": 1080 },
            { "name": "mobile", "width": 375, "height": 812 }
        ],
        "functions": {
            "login": [
                { "action": "click", "selector": "#login-btn" },
                { "action": "type", "selector": "#username", "text": "admin" }
            ]
        },
        "snapshotDirectory": "my_snapshots",
        "threshold": 5,
        "retry": 3,
        "parallelism": 4,
        "diffPixelColor": { "r": 0, "g": 255, "b": 0 },
        "httpHeaders": { "Authorization": "Bearer token123" },
        "tests": [
            {
                "name": "home page",
                "url": "/"
            }
        ]
    });

    let config: GlobalConfig = serde_json::from_value(data).unwrap();

    assert_eq!(config.base_url, "https://example.com");

    let browser = config.browser.as_ref().unwrap();
    assert_eq!(browser.version.as_deref(), Some("120.0"));
    assert_eq!(browser.args, vec!["--no-sandbox", "--disable-gpu"]);
    assert_eq!(browser.env.get("DISPLAY").unwrap(), ":99");

    assert!(!config.full_screen);
    assert_eq!(config.test_pattern, "specs/**/*.spec.json");
    assert_eq!(config.ignore_patterns, vec!["**/skip-this/**"]);

    let sizes = config.default_sizes.as_ref().unwrap();
    assert_eq!(sizes.len(), 2);
    assert_eq!(sizes[0].name, "desktop");
    assert_eq!(sizes[0].width, 1920);
    assert_eq!(sizes[0].height, 1080);
    assert_eq!(sizes[1].name, "mobile");
    assert_eq!(sizes[1].width, 375);
    assert_eq!(sizes[1].height, 812);

    let login_fn = config.functions.get("login").unwrap();
    assert_eq!(login_fn.len(), 2);

    assert_eq!(config.snapshot_directory, "my_snapshots");
    assert_eq!(config.threshold, 5);
    assert_eq!(config.retry, 3);
    assert_eq!(config.parallelism, Some(4));
    assert_eq!(config.diff_pixel_color.r, 0);
    assert_eq!(config.diff_pixel_color.g, 255);
    assert_eq!(config.diff_pixel_color.b, 0);
    assert_eq!(
        config.http_headers.get("Authorization").unwrap(),
        "Bearer token123"
    );
    assert_eq!(config.tests.len(), 1);
    assert_eq!(config.tests[0].name, "home page");
    assert_eq!(config.tests[0].url, "/");
}

#[test]
fn test_deserialize_test_config() {
    let data = json!([
        {
            "name": "dashboard",
            "url": "/dashboard",
            "threshold": 10,
            "retry": 2,
            "only": true,
            "skip": false,
            "expectedResponseCode": 200,
            "sizes": [
                { "name": "tablet", "width": 768, "height": 1024 }
            ],
            "actions": [
                { "action": "wait", "timeout": 1000 },
                { "action": "click", "selector": ".menu" }
            ],
            "ignore": [
                { "x1": 0, "y1": 0, "x2": 100, "y2": 50 },
                { "selector": ".dynamic-ad" }
            ],
            "httpHeaders": { "X-Custom": "value" }
        },
        {
            "name": "about",
            "url": "/about"
        }
    ]);

    let tests: Vec<TestConfig> = serde_json::from_value(data).unwrap();

    assert_eq!(tests.len(), 2);

    let t = &tests[0];
    assert_eq!(t.name, "dashboard");
    assert_eq!(t.url, "/dashboard");
    assert_eq!(t.threshold, Some(10));
    assert_eq!(t.retry, Some(2));
    assert!(t.only);
    assert!(!t.skip);
    assert_eq!(t.expected_response_code, Some(200));

    let sizes = t.sizes.as_ref().unwrap();
    assert_eq!(sizes.len(), 1);
    assert_eq!(sizes[0].name, "tablet");

    let actions = t.actions.as_ref().unwrap();
    assert_eq!(actions.len(), 2);

    let ignores = t.ignore.as_ref().unwrap();
    assert_eq!(ignores.len(), 2);

    assert_eq!(
        t.http_headers.as_ref().unwrap().get("X-Custom").unwrap(),
        "value"
    );

    // Second test has all defaults
    let t2 = &tests[1];
    assert_eq!(t2.name, "about");
    assert_eq!(t2.url, "/about");
    assert!(t2.threshold.is_none());
    assert!(t2.retry.is_none());
    assert!(!t2.only);
    assert!(!t2.skip);
    assert!(t2.expected_response_code.is_none());
    assert!(t2.sizes.is_none());
    assert!(t2.browser.is_none());
    assert!(t2.actions.is_none());
    assert!(t2.ignore.is_none());
    assert!(t2.http_headers.is_none());
}

#[test]
fn test_deserialize_size() {
    let data = json!({
        "name": "laptop",
        "width": 1440,
        "height": 900
    });

    let size: Size = serde_json::from_value(data).unwrap();

    assert_eq!(size.name, "laptop");
    assert_eq!(size.width, 1440);
    assert_eq!(size.height, 900);
}

#[test]
fn test_action_size_restriction() {
    // The "@" field should deserialize into size_restriction
    let data = json!({
        "action": "click",
        "selector": "#button",
        "@": ["desktop", "tablet"]
    });

    let action: Action = serde_json::from_value(data).unwrap();

    match action {
        Action::Click {
            selector,
            size_restriction,
        } => {
            assert_eq!(selector, "#button");
            let restriction = size_restriction.unwrap();
            assert_eq!(restriction, vec!["desktop", "tablet"]);
        }
        _ => panic!("Expected Click variant"),
    }

    // Without @ field, size_restriction should be None
    let data_no_restriction = json!({
        "action": "wait",
        "timeout": 500
    });

    let action2: Action = serde_json::from_value(data_no_restriction).unwrap();

    match action2 {
        Action::Wait {
            timeout,
            size_restriction,
        } => {
            assert_eq!(timeout, 500);
            assert!(size_restriction.is_none());
        }
        _ => panic!("Expected Wait variant"),
    }
}

#[test]
fn test_all_action_types() {
    // Wait
    let wait_json = json!({ "action": "wait", "timeout": 2000 });
    let wait: Action = serde_json::from_value(wait_json).unwrap();
    match wait {
        Action::Wait { timeout, .. } => assert_eq!(timeout, 2000),
        _ => panic!("Expected Wait"),
    }

    // Click
    let click_json = json!({ "action": "click", "selector": "button.submit" });
    let click: Action = serde_json::from_value(click_json).unwrap();
    match click {
        Action::Click { selector, .. } => assert_eq!(selector, "button.submit"),
        _ => panic!("Expected Click"),
    }

    // Type (note: camelCase of "Type" is "type")
    let type_json = json!({
        "action": "type",
        "selector": "input#email",
        "text": "test@example.com"
    });
    let type_action: Action = serde_json::from_value(type_json).unwrap();
    match type_action {
        Action::Type { selector, text, .. } => {
            assert_eq!(selector, "input#email");
            assert_eq!(text, "test@example.com");
        }
        _ => panic!("Expected Type"),
    }

    // Scroll with all fields
    let scroll_json = json!({
        "action": "scroll",
        "selector": ".content",
        "pxAmount": -200
    });
    let scroll: Action = serde_json::from_value(scroll_json).unwrap();
    match scroll {
        Action::Scroll {
            selector,
            px_amount,
            ..
        } => {
            assert_eq!(selector.as_deref(), Some(".content"));
            assert_eq!(px_amount, Some(-200));
        }
        _ => panic!("Expected Scroll"),
    }

    // Scroll minimal (no optional fields)
    let scroll_minimal = json!({ "action": "scroll" });
    let scroll2: Action = serde_json::from_value(scroll_minimal).unwrap();
    match scroll2 {
        Action::Scroll {
            selector,
            px_amount,
            ..
        } => {
            assert!(selector.is_none());
            assert!(px_amount.is_none());
        }
        _ => panic!("Expected Scroll"),
    }

    // ForcePseudoState
    let pseudo_json = json!({
        "action": "forcePseudoState",
        "selector": "a.nav-link",
        "hover": true,
        "active": false,
        "focus": true,
        "visited": false
    });
    let pseudo: Action = serde_json::from_value(pseudo_json).unwrap();
    match pseudo {
        Action::ForcePseudoState {
            selector,
            hover,
            active,
            focus,
            visited,
            ..
        } => {
            assert_eq!(selector, "a.nav-link");
            assert!(hover);
            assert!(!active);
            assert!(focus);
            assert!(!visited);
        }
        _ => panic!("Expected ForcePseudoState"),
    }

    // ForcePseudoState with defaults (booleans default to false)
    let pseudo_minimal = json!({
        "action": "forcePseudoState",
        "selector": "div"
    });
    let pseudo2: Action = serde_json::from_value(pseudo_minimal).unwrap();
    match pseudo2 {
        Action::ForcePseudoState {
            hover,
            active,
            focus,
            visited,
            ..
        } => {
            assert!(!hover);
            assert!(!active);
            assert!(!focus);
            assert!(!visited);
        }
        _ => panic!("Expected ForcePseudoState"),
    }

    // Function
    let func_json = json!({ "action": "function", "name": "login" });
    let func: Action = serde_json::from_value(func_json).unwrap();
    match func {
        Action::Function { name, .. } => assert_eq!(name, "login"),
        _ => panic!("Expected Function"),
    }
}

#[test]
fn test_all_ignore_types() {
    // Coordinates
    let coords_json = json!({
        "x1": 10,
        "y1": 20,
        "x2": 300,
        "y2": 400
    });
    let coords: IgnoreRegion = serde_json::from_value(coords_json).unwrap();
    match coords {
        IgnoreRegion::Coordinates {
            x1,
            y1,
            x2,
            y2,
            size_restriction,
        } => {
            assert_eq!(x1, 10);
            assert_eq!(y1, 20);
            assert_eq!(x2, 300);
            assert_eq!(y2, 400);
            assert!(size_restriction.is_none());
        }
        _ => panic!("Expected Coordinates"),
    }

    // Coordinates with size restriction
    let coords_with_restriction = json!({
        "x1": 0,
        "y1": 0,
        "x2": 50,
        "y2": 50,
        "@": ["mobile"]
    });
    let coords2: IgnoreRegion = serde_json::from_value(coords_with_restriction).unwrap();
    match coords2 {
        IgnoreRegion::Coordinates {
            size_restriction, ..
        } => {
            assert_eq!(size_restriction.unwrap(), vec!["mobile"]);
        }
        _ => panic!("Expected Coordinates"),
    }

    // Selector
    let selector_json = json!({
        "selector": ".ad-banner"
    });
    let sel: IgnoreRegion = serde_json::from_value(selector_json).unwrap();
    match sel {
        IgnoreRegion::Selector {
            selector,
            size_restriction,
        } => {
            assert_eq!(selector, ".ad-banner");
            assert!(size_restriction.is_none());
        }
        _ => panic!("Expected Selector"),
    }

    // SelectorAll
    let selector_all_json = json!({
        "selectorAll": "img.lazy"
    });
    let sel_all: IgnoreRegion = serde_json::from_value(selector_all_json).unwrap();
    match sel_all {
        IgnoreRegion::SelectorAll {
            selector_all,
            size_restriction,
        } => {
            assert_eq!(selector_all, "img.lazy");
            assert!(size_restriction.is_none());
        }
        _ => panic!("Expected SelectorAll"),
    }
}
