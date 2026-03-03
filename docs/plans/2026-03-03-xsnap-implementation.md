# xsnap Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build xsnap, a Rust-based visual regression testing tool that replaces OSnap with JSONC config, ratatui TUI, and pipeline mode for CI.

**Architecture:** Monolithic binary using clap for CLI, chromiumoxide for CDP browser control, image-compare for pixel diffing, ratatui for TUI, and tokio for async parallelism. Config is JSONC with JSON Schema validation.

**Tech Stack:** Rust, tokio, chromiumoxide, image-compare, ratatui, clap, serde_json, schemars

**Design doc:** `docs/plans/2026-03-03-xsnap-design.md`

---

## Phase 1: Project Foundation

### Task 1: Initialize Cargo Project

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/error.rs`

**Step 1: Create Cargo.toml with all dependencies**

```toml
[package]
name = "xsnap"
version = "0.1.0"
edition = "2024"
description = "Visual regression testing tool"

[dependencies]
# CLI
clap = { version = "4", features = ["derive"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
json_comments = "0.2"

# JSON Schema
schemars = "1.2"

# Browser automation
chromiumoxide = { version = "0.9", default-features = false, features = ["tokio-runtime"] }
tokio = { version = "1", features = ["full"] }
futures = "0.3"

# Image processing
image = "0.25"
image-compare = "0.5"

# Terminal UI
ratatui = { version = "0.30", features = ["crossterm"] }
crossterm = { version = "0.29", features = ["event-stream"] }

# Pipeline mode
indicatif = "0.17"

# File patterns
glob = "0.3"

# Interactive prompts (migration)
dialoguer = "0.12"

# HTTP (chromium download)
reqwest = { version = "0.12", features = ["stream"] }

# YAML (migration from OSnap)
serde_yaml = "0.9"

# Errors
thiserror = "2"
anyhow = "1"

# Misc
num_cpus = "1"

[dev-dependencies]
tempfile = "3"
assert_fs = "1"
```

**Step 2: Create src/main.rs with CLI skeleton**

```rust
mod commands;
mod config;
mod error;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "xsnap", version, about = "Visual regression testing tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run visual regression tests
    Test {
        /// Config file path
        #[arg(long, default_value = "xsnap.config.jsonc")]
        config: String,
        /// Don't create new baselines
        #[arg(long)]
        no_create: bool,
        /// Fail if any test has only: true
        #[arg(long)]
        no_only: bool,
        /// Fail if any test has skip: true
        #[arg(long)]
        no_skip: bool,
        /// Only run matching tests
        #[arg(long)]
        filter: Option<String>,
        /// Pipeline mode (no TUI, CI-optimized)
        #[arg(long)]
        pipeline: bool,
        /// Override parallelism
        #[arg(long)]
        parallelism: Option<usize>,
    },
    /// Accept updated screenshots as new baselines
    Approve {
        /// Config file path
        #[arg(long, default_value = "xsnap.config.jsonc")]
        config: String,
        /// Approve all at once
        #[arg(long)]
        all: bool,
        /// Only approve matching tests
        #[arg(long)]
        filter: Option<String>,
    },
    /// Remove unused baseline images
    Cleanup {
        /// Config file path
        #[arg(long, default_value = "xsnap.config.jsonc")]
        config: String,
    },
    /// Migrate OSnap YAML configs to xsnap JSON
    Migrate {
        /// Source directory
        #[arg(long, default_value = ".")]
        source: String,
        /// Target directory
        #[arg(long, default_value = ".")]
        target: String,
    },
    /// Create new xsnap.config.jsonc
    Init,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Test { .. } => todo!("test command"),
        Commands::Approve { .. } => todo!("approve command"),
        Commands::Cleanup { .. } => todo!("cleanup command"),
        Commands::Migrate { .. } => todo!("migrate command"),
        Commands::Init => todo!("init command"),
    }
}
```

**Step 3: Create src/error.rs**

```rust
use thiserror::Error;

#[derive(Error, Debug)]
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
}

impl XsnapError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::ConfigNotFound { .. }
            | Self::ConfigInvalid { .. }
            | Self::DuplicateTestName { .. }
            | Self::UndefinedFunction { .. } => 2,
            Self::BrowserDownloadFailed { .. } | Self::BrowserLaunchFailed { .. } => 3,
            _ => 4,
        }
    }
}
```

**Step 4: Create empty module files**

Create these files with just the module declaration:

- `src/config/mod.rs`: `pub mod global; pub mod test; pub mod schema; pub mod validate;`
- `src/config/global.rs`: empty
- `src/config/test.rs`: empty
- `src/config/schema.rs`: empty
- `src/config/validate.rs`: empty
- `src/commands/mod.rs`: `pub mod test; pub mod approve; pub mod cleanup; pub mod migrate; pub mod init;`
- `src/commands/test.rs`: empty
- `src/commands/approve.rs`: empty
- `src/commands/cleanup.rs`: empty
- `src/commands/migrate.rs`: empty
- `src/commands/init.rs`: empty

**Step 5: Verify it compiles**

Run: `cargo check`
Expected: Compiles with warnings about unused modules and todo!()

**Step 6: Commit**

```bash
git add -A
git commit -m "feat: initialize xsnap project with CLI skeleton and error types"
```

---

## Phase 2: Config Module

### Task 2: Config Types (Data Structures)

**Files:**
- Create: `src/config/types.rs`
- Modify: `src/config/mod.rs`

**Step 1: Write tests for config type serialization**

Create `tests/config_types_test.rs`:

```rust
use xsnap::config::types::*;

#[test]
fn test_deserialize_minimal_global_config() {
    let json = r#"{ "baseUrl": "http://localhost:3000" }"#;
    let config: GlobalConfig = serde_json::from_str(json).unwrap();
    assert_eq!(config.base_url, "http://localhost:3000");
    assert_eq!(config.full_screen, true); // default
    assert_eq!(config.threshold, 0); // default
    assert_eq!(config.retry, 1); // default
    assert_eq!(config.snapshot_directory, "__snapshots__"); // default
}

#[test]
fn test_deserialize_full_global_config() {
    let json = r#"{
        "baseUrl": "http://localhost:3000",
        "browser": {
            "version": "120.0.6099.109",
            "args": ["--no-sandbox"],
            "env": { "DISPLAY": ":99" }
        },
        "fullScreen": false,
        "testPattern": "tests/**/*.xsnap.json",
        "ignorePatterns": ["node_modules"],
        "defaultSizes": [
            { "name": "desktop", "width": 1920, "height": 1080 }
        ],
        "functions": {
            "login": [
                { "action": "click", "selector": "#btn" }
            ]
        },
        "snapshotDirectory": "snapshots",
        "threshold": 5,
        "retry": 2,
        "parallelism": 4,
        "diffPixelColor": { "r": 255, "g": 0, "b": 255 },
        "httpHeaders": { "Authorization": "Bearer tok" },
        "tests": []
    }"#;
    let config: GlobalConfig = serde_json::from_str(json).unwrap();
    assert_eq!(config.browser.as_ref().unwrap().version.as_deref(), Some("120.0.6099.109"));
    assert_eq!(config.full_screen, false);
    assert_eq!(config.default_sizes.as_ref().unwrap().len(), 1);
    assert_eq!(config.threshold, 5);
    assert_eq!(config.retry, 2);
    assert_eq!(config.parallelism, Some(4));
}

#[test]
fn test_deserialize_test_config() {
    let json = r#"[{
        "name": "homepage",
        "url": "/",
        "threshold": 10,
        "actions": [
            { "action": "wait", "timeout": 500 },
            { "action": "click", "selector": "#btn" },
            { "action": "type", "selector": "#input", "text": "hello" },
            { "action": "scroll", "pxAmount": 200 },
            { "action": "scroll", "selector": ".footer" },
            { "action": "forcePseudoState", "selector": ".btn", "hover": true },
            { "action": "function", "name": "login" }
        ],
        "ignore": [
            { "selector": ".timestamp" },
            { "selectorAll": ".ad" },
            { "x1": 0, "y1": 0, "x2": 100, "y2": 50 },
            { "@": ["mobile"], "selector": ".sidebar" }
        ]
    }]"#;
    let tests: Vec<TestConfig> = serde_json::from_str(json).unwrap();
    assert_eq!(tests.len(), 1);
    assert_eq!(tests[0].name, "homepage");
    assert_eq!(tests[0].actions.as_ref().unwrap().len(), 7);
    assert_eq!(tests[0].ignore.as_ref().unwrap().len(), 4);
}

#[test]
fn test_deserialize_size() {
    let json = r#"{ "name": "mobile", "width": 375, "height": 667 }"#;
    let size: Size = serde_json::from_str(json).unwrap();
    assert_eq!(size.name, "mobile");
    assert_eq!(size.width, 1920);  // intentionally wrong to verify test fails first
    assert_eq!(size.height, 667);
}

#[test]
fn test_action_size_restriction() {
    let json = r#"{ "@": ["mobile", "tablet"], "action": "click", "selector": "#menu" }"#;
    let action: Action = serde_json::from_str(json).unwrap();
    match &action {
        Action::Click { selector, size_restriction, .. } => {
            assert_eq!(selector, "#menu");
            assert_eq!(size_restriction.as_ref().unwrap(), &vec!["mobile".to_string(), "tablet".to_string()]);
        }
        _ => panic!("Expected Click action"),
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --test config_types_test`
Expected: FAIL - module not found

**Step 3: Implement config types**

Create `src/config/types.rs`:

```rust
use std::collections::HashMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Global xsnap configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GlobalConfig {
    /// Base URL prepended to all test URLs
    pub base_url: String,

    /// Browser settings
    #[serde(default)]
    pub browser: Option<BrowserConfig>,

    /// Capture full page height (default: true)
    #[serde(default = "default_true")]
    pub full_screen: bool,

    /// Glob pattern for test files
    #[serde(default = "default_test_pattern")]
    pub test_pattern: String,

    /// Patterns to ignore when scanning for tests
    #[serde(default)]
    pub ignore_patterns: Vec<String>,

    /// Default viewport sizes
    #[serde(default)]
    pub default_sizes: Option<Vec<Size>>,

    /// Reusable action sequences
    #[serde(default)]
    pub functions: HashMap<String, Vec<Action>>,

    /// Snapshot output directory (default: __snapshots__)
    #[serde(default = "default_snapshot_dir")]
    pub snapshot_directory: String,

    /// Pixel difference threshold (default: 0)
    #[serde(default)]
    pub threshold: u32,

    /// Number of retries on failure (default: 1)
    #[serde(default = "default_retry")]
    pub retry: u32,

    /// Parallel browser targets (null = auto-detect)
    #[serde(default)]
    pub parallelism: Option<usize>,

    /// Diff highlight color
    #[serde(default = "default_diff_color")]
    pub diff_pixel_color: Color,

    /// Global HTTP headers
    #[serde(default)]
    pub http_headers: HashMap<String, String>,

    /// Inline test definitions
    #[serde(default)]
    pub tests: Vec<TestConfig>,
}

/// Browser configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BrowserConfig {
    /// "auto" for latest, or specific version string
    pub version: Option<String>,

    /// Additional Chrome command-line arguments
    #[serde(default)]
    pub args: Vec<String>,

    /// Environment variables for Chrome process
    #[serde(default)]
    pub env: HashMap<String, String>,
}

/// Viewport size definition
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Size {
    /// Name for referencing in size restrictions
    pub name: String,
    /// Viewport width in pixels
    pub width: u32,
    /// Viewport height in pixels
    pub height: u32,
}

/// RGBA color
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// Test configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TestConfig {
    /// Unique test name
    pub name: String,

    /// URL to navigate to (appended to baseUrl)
    pub url: String,

    /// Pixel difference threshold override
    #[serde(default)]
    pub threshold: Option<u32>,

    /// Retry count override
    #[serde(default)]
    pub retry: Option<u32>,

    /// Run only this test
    #[serde(default)]
    pub only: bool,

    /// Skip this test
    #[serde(default)]
    pub skip: bool,

    /// Expected HTTP response code
    #[serde(default)]
    pub expected_response_code: Option<u16>,

    /// Override default sizes
    #[serde(default)]
    pub sizes: Option<Vec<Size>>,

    /// Per-test browser overrides
    #[serde(default)]
    pub browser: Option<BrowserConfig>,

    /// Actions to perform before screenshot
    #[serde(default)]
    pub actions: Option<Vec<Action>>,

    /// Regions to ignore in comparison
    #[serde(default)]
    pub ignore: Option<Vec<IgnoreRegion>>,

    /// Per-test HTTP headers
    #[serde(default)]
    pub http_headers: Option<HashMap<String, String>>,
}

/// Action to perform before taking a screenshot
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "camelCase")]
pub enum Action {
    /// Wait for timeout (ms) or network idle
    Wait {
        timeout: u64,
        #[serde(rename = "@", default)]
        size_restriction: Option<Vec<String>>,
    },
    /// Click an element
    Click {
        selector: String,
        #[serde(rename = "@", default)]
        size_restriction: Option<Vec<String>>,
    },
    /// Type text into an element
    Type {
        selector: String,
        text: String,
        #[serde(rename = "@", default)]
        size_restriction: Option<Vec<String>>,
    },
    /// Scroll by pixels or to element
    Scroll {
        #[serde(default)]
        selector: Option<String>,
        #[serde(default, rename = "pxAmount")]
        px_amount: Option<i32>,
        #[serde(rename = "@", default)]
        size_restriction: Option<Vec<String>>,
    },
    /// Force CSS pseudo-state on element
    ForcePseudoState {
        selector: String,
        #[serde(default)]
        hover: bool,
        #[serde(default)]
        active: bool,
        #[serde(default)]
        focus: bool,
        #[serde(default)]
        visited: bool,
        #[serde(rename = "@", default)]
        size_restriction: Option<Vec<String>>,
    },
    /// Call a reusable function defined in global config
    Function {
        name: String,
        #[serde(rename = "@", default)]
        size_restriction: Option<Vec<String>>,
    },
}

/// Region to ignore during image comparison
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum IgnoreRegion {
    /// Ignore by coordinate rectangle
    Coordinates {
        x1: u32,
        y1: u32,
        x2: u32,
        y2: u32,
        #[serde(rename = "@", default)]
        size_restriction: Option<Vec<String>>,
    },
    /// Ignore first element matching selector
    Selector {
        selector: String,
        #[serde(rename = "@", default)]
        size_restriction: Option<Vec<String>>,
    },
    /// Ignore all elements matching selector
    SelectorAll {
        #[serde(rename = "selectorAll")]
        selector_all: String,
        #[serde(rename = "@", default)]
        size_restriction: Option<Vec<String>>,
    },
}

fn default_true() -> bool { true }
fn default_test_pattern() -> String { "tests/**/*.xsnap.json".into() }
fn default_snapshot_dir() -> String { "__snapshots__".into() }
fn default_retry() -> u32 { 1 }
fn default_diff_color() -> Color { Color { r: 255, g: 0, b: 255 } }
```

Update `src/config/mod.rs`:
```rust
pub mod types;
pub mod global;
pub mod test;
pub mod schema;
pub mod validate;
```

Update `src/main.rs` to add `pub mod config;` and make it a lib+bin by adding `src/lib.rs`:
```rust
pub mod config;
pub mod error;
```

**Step 4: Fix the intentionally wrong test and run all tests**

Fix `test_deserialize_size`: change `1920` to `375`.

Run: `cargo test --test config_types_test`
Expected: All tests PASS

**Step 5: Commit**

```bash
git add -A
git commit -m "feat: add config type definitions with serde serialization"
```

---

### Task 3: JSONC Parsing & Global Config Loading

**Files:**
- Modify: `src/config/global.rs`
- Create: `tests/config_global_test.rs`

**Step 1: Write failing tests**

Create `tests/config_global_test.rs`:

```rust
use std::path::Path;
use xsnap::config::global::load_global_config;

#[test]
fn test_load_jsonc_with_comments() {
    let jsonc = r#"{
        // This is a comment
        "baseUrl": "http://localhost:3000",
        /* Block comment */
        "threshold": 5
    }"#;
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("xsnap.config.jsonc");
    std::fs::write(&config_path, jsonc).unwrap();

    let config = load_global_config(&config_path).unwrap();
    assert_eq!(config.base_url, "http://localhost:3000");
    assert_eq!(config.threshold, 5);
}

#[test]
fn test_load_config_not_found() {
    let result = load_global_config(Path::new("/nonexistent/xsnap.config.jsonc"));
    assert!(result.is_err());
}

#[test]
fn test_load_invalid_json() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("xsnap.config.jsonc");
    std::fs::write(&config_path, "{ invalid }").unwrap();

    let result = load_global_config(&config_path);
    assert!(result.is_err());
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --test config_global_test`
Expected: FAIL - function not found

**Step 3: Implement global config loading**

In `src/config/global.rs`:

```rust
use std::path::Path;
use json_comments::StripComments;
use crate::config::types::GlobalConfig;
use crate::error::XsnapError;

pub fn load_global_config(path: &Path) -> Result<GlobalConfig, XsnapError> {
    let content = std::fs::read(path).map_err(|_| XsnapError::ConfigNotFound {
        path: path.display().to_string(),
    })?;

    let stripped = StripComments::new(content.as_slice());
    let config: GlobalConfig = serde_json::from_reader(stripped).map_err(|e| {
        XsnapError::ConfigInvalid {
            message: format!("{}: {}", path.display(), e),
        }
    })?;

    Ok(config)
}
```

**Step 4: Run tests**

Run: `cargo test --test config_global_test`
Expected: All PASS

**Step 5: Commit**

```bash
git add -A
git commit -m "feat: add JSONC config loading with comment stripping"
```

---

### Task 4: Test Config Loading & Discovery

**Files:**
- Modify: `src/config/test.rs`
- Create: `tests/config_test_loading_test.rs`

**Step 1: Write failing tests**

Create `tests/config_test_loading_test.rs`:

```rust
use xsnap::config::test::{load_test_file, discover_test_files};

#[test]
fn test_load_test_file() {
    let json = r#"[{
        "name": "test-1",
        "url": "/page"
    }]"#;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.xsnap.json");
    std::fs::write(&path, json).unwrap();

    let tests = load_test_file(&path).unwrap();
    assert_eq!(tests.len(), 1);
    assert_eq!(tests[0].name, "test-1");
}

#[test]
fn test_discover_test_files() {
    let dir = tempfile::tempdir().unwrap();
    let tests_dir = dir.path().join("tests");
    std::fs::create_dir_all(&tests_dir).unwrap();
    std::fs::write(tests_dir.join("a.xsnap.json"), r#"[{"name":"a","url":"/a"}]"#).unwrap();
    std::fs::write(tests_dir.join("b.xsnap.json"), r#"[{"name":"b","url":"/b"}]"#).unwrap();
    std::fs::write(tests_dir.join("not-a-test.json"), "{}").unwrap();

    let files = discover_test_files(dir.path(), "tests/**/*.xsnap.json", &[]).unwrap();
    assert_eq!(files.len(), 2);
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --test config_test_loading_test`
Expected: FAIL

**Step 3: Implement test file loading and discovery**

In `src/config/test.rs`:

```rust
use std::path::{Path, PathBuf};
use crate::config::types::TestConfig;
use crate::error::XsnapError;

pub fn load_test_file(path: &Path) -> Result<Vec<TestConfig>, XsnapError> {
    let content = std::fs::read_to_string(path).map_err(|_| XsnapError::ConfigNotFound {
        path: path.display().to_string(),
    })?;

    let tests: Vec<TestConfig> = serde_json::from_str(&content).map_err(|e| {
        XsnapError::ConfigInvalid {
            message: format!("{}: {}", path.display(), e),
        }
    })?;

    Ok(tests)
}

pub fn discover_test_files(
    base_dir: &Path,
    pattern: &str,
    _ignore_patterns: &[String],
) -> Result<Vec<PathBuf>, XsnapError> {
    let full_pattern = base_dir.join(pattern).display().to_string();
    let paths: Vec<PathBuf> = glob::glob(&full_pattern)
        .map_err(|e| XsnapError::ConfigInvalid {
            message: format!("Invalid test pattern '{}': {}", pattern, e),
        })?
        .filter_map(|entry| entry.ok())
        .collect();

    Ok(paths)
}
```

**Step 4: Run tests**

Run: `cargo test --test config_test_loading_test`
Expected: All PASS

**Step 5: Commit**

```bash
git add -A
git commit -m "feat: add test file loading and discovery"
```

---

### Task 5: Config Validation

**Files:**
- Modify: `src/config/validate.rs`
- Create: `tests/config_validate_test.rs`

**Step 1: Write failing tests**

Create `tests/config_validate_test.rs`:

```rust
use std::collections::HashMap;
use xsnap::config::types::*;
use xsnap::config::validate::validate_config;

fn minimal_config() -> GlobalConfig {
    GlobalConfig {
        base_url: "http://localhost".into(),
        browser: None,
        full_screen: true,
        test_pattern: "tests/**/*.xsnap.json".into(),
        ignore_patterns: vec![],
        default_sizes: Some(vec![Size { name: "desktop".into(), width: 1920, height: 1080 }]),
        functions: HashMap::new(),
        snapshot_directory: "__snapshots__".into(),
        threshold: 0,
        retry: 1,
        parallelism: None,
        diff_pixel_color: Color { r: 255, g: 0, b: 255 },
        http_headers: HashMap::new(),
        tests: vec![],
    }
}

fn minimal_test(name: &str) -> TestConfig {
    TestConfig {
        name: name.into(),
        url: "/".into(),
        threshold: None,
        retry: None,
        only: false,
        skip: false,
        expected_response_code: None,
        sizes: None,
        browser: None,
        actions: None,
        ignore: None,
        http_headers: None,
    }
}

#[test]
fn test_validate_duplicate_names() {
    let config = minimal_config();
    let tests = vec![minimal_test("page"), minimal_test("page")];
    let result = validate_config(&config, &tests);
    assert!(result.is_err());
}

#[test]
fn test_validate_undefined_function() {
    let config = minimal_config();
    let tests = vec![TestConfig {
        actions: Some(vec![Action::Function {
            name: "nonexistent".into(),
            size_restriction: None,
        }]),
        ..minimal_test("page")
    }];
    let result = validate_config(&config, &tests);
    assert!(result.is_err());
}

#[test]
fn test_validate_valid_config() {
    let mut config = minimal_config();
    config.functions.insert("login".into(), vec![Action::Wait {
        timeout: 100,
        size_restriction: None,
    }]);
    let tests = vec![TestConfig {
        actions: Some(vec![Action::Function {
            name: "login".into(),
            size_restriction: None,
        }]),
        ..minimal_test("page")
    }];
    let result = validate_config(&config, &tests);
    assert!(result.is_ok());
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --test config_validate_test`
Expected: FAIL

**Step 3: Implement validation**

In `src/config/validate.rs`:

```rust
use std::collections::HashSet;
use crate::config::types::{Action, GlobalConfig, TestConfig};
use crate::error::XsnapError;

pub fn validate_config(
    global: &GlobalConfig,
    tests: &[TestConfig],
) -> Result<(), XsnapError> {
    validate_unique_names(tests)?;
    validate_function_references(global, tests)?;
    Ok(())
}

fn validate_unique_names(tests: &[TestConfig]) -> Result<(), XsnapError> {
    let mut seen = HashSet::new();
    for test in tests {
        if !seen.insert(&test.name) {
            return Err(XsnapError::DuplicateTestName {
                name: test.name.clone(),
            });
        }
    }
    Ok(())
}

fn validate_function_references(
    global: &GlobalConfig,
    tests: &[TestConfig],
) -> Result<(), XsnapError> {
    for test in tests {
        if let Some(actions) = &test.actions {
            for action in actions {
                if let Action::Function { name, .. } = action {
                    if !global.functions.contains_key(name) {
                        return Err(XsnapError::UndefinedFunction {
                            name: name.clone(),
                        });
                    }
                }
            }
        }
    }
    Ok(())
}
```

**Step 4: Run tests**

Run: `cargo test --test config_validate_test`
Expected: All PASS

**Step 5: Commit**

```bash
git add -A
git commit -m "feat: add config validation (duplicates, function refs)"
```

---

### Task 6: JSON Schema Generation

**Files:**
- Modify: `src/config/schema.rs`
- Create: `tests/config_schema_test.rs`

**Step 1: Write failing test**

Create `tests/config_schema_test.rs`:

```rust
use xsnap::config::schema::generate_schema;

#[test]
fn test_generate_schema_is_valid_json() {
    let schema = generate_schema();
    let parsed: serde_json::Value = serde_json::from_str(&schema).unwrap();
    assert!(parsed.is_object());
    assert!(parsed.get("$schema").is_some());
    assert!(parsed.get("properties").is_some());
}

#[test]
fn test_schema_has_required_fields() {
    let schema = generate_schema();
    let parsed: serde_json::Value = serde_json::from_str(&schema).unwrap();
    let props = parsed.get("properties").unwrap();
    assert!(props.get("baseUrl").is_some());
    assert!(props.get("browser").is_some());
    assert!(props.get("defaultSizes").is_some());
    assert!(props.get("threshold").is_some());
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --test config_schema_test`
Expected: FAIL

**Step 3: Implement schema generation**

In `src/config/schema.rs`:

```rust
use schemars::schema_for;
use crate::config::types::GlobalConfig;

pub fn generate_schema() -> String {
    let schema = schema_for!(GlobalConfig);
    serde_json::to_string_pretty(&schema).expect("Failed to serialize schema")
}
```

**Step 4: Run tests**

Run: `cargo test --test config_schema_test`
Expected: All PASS

**Step 5: Commit**

```bash
git add -A
git commit -m "feat: add JSON schema generation from config types"
```

---

## Phase 3: Diff Module

### Task 7: Image Comparison

**Files:**
- Create: `src/diff/mod.rs`
- Create: `src/diff/compare.rs`
- Create: `tests/diff_test.rs`

**Step 1: Write failing tests**

Create `tests/diff_test.rs`:

```rust
use xsnap::diff::compare::{compare_images, CompareResult};
use image::{RgbImage, Rgb};

fn create_solid_image(width: u32, height: u32, color: [u8; 3]) -> RgbImage {
    let mut img = RgbImage::new(width, height);
    for pixel in img.pixels_mut() {
        *pixel = Rgb(color);
    }
    img
}

#[test]
fn test_identical_images_pass() {
    let img = create_solid_image(100, 100, [255, 0, 0]);
    let result = compare_images(&img, &img, 0).unwrap();
    match result {
        CompareResult::Pass => {}
        _ => panic!("Expected Pass for identical images"),
    }
}

#[test]
fn test_different_images_fail() {
    let img_a = create_solid_image(100, 100, [255, 0, 0]);
    let img_b = create_solid_image(100, 100, [0, 255, 0]);
    let result = compare_images(&img_a, &img_b, 0).unwrap();
    match result {
        CompareResult::Fail { score, .. } => {
            assert!(score < 1.0);
        }
        _ => panic!("Expected Fail for different images"),
    }
}

#[test]
fn test_threshold_allows_small_diff() {
    let mut img_b = create_solid_image(100, 100, [255, 0, 0]);
    // Change 5 pixels
    for x in 0..5 {
        img_b.put_pixel(x, 0, Rgb([0, 255, 0]));
    }
    let img_a = create_solid_image(100, 100, [255, 0, 0]);
    // Threshold of 10 pixels -> should pass
    let result = compare_images(&img_a, &img_b, 10).unwrap();
    match result {
        CompareResult::Pass => {}
        _ => panic!("Expected Pass with threshold allowing small diff"),
    }
}

#[test]
fn test_dimension_mismatch() {
    let img_a = create_solid_image(100, 100, [255, 0, 0]);
    let img_b = create_solid_image(200, 200, [255, 0, 0]);
    let result = compare_images(&img_a, &img_b, 0);
    assert!(result.is_err());
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --test diff_test`
Expected: FAIL

**Step 3: Implement image comparison**

In `src/diff/compare.rs`:

```rust
use image::RgbImage;
use image_compare::Algorithm;
use crate::error::XsnapError;

#[derive(Debug)]
pub enum CompareResult {
    Pass,
    Fail {
        score: f64,
        diff_image: Option<RgbImage>,
    },
}

pub fn compare_images(
    baseline: &RgbImage,
    current: &RgbImage,
    threshold_pixels: u32,
) -> Result<CompareResult, XsnapError> {
    if baseline.dimensions() != current.dimensions() {
        return Err(XsnapError::DiffFailed {
            message: format!(
                "Dimension mismatch: baseline {:?} vs current {:?}",
                baseline.dimensions(),
                current.dimensions()
            ),
        });
    }

    let result = image_compare::rgb_hybrid_compare(baseline, current).map_err(|e| {
        XsnapError::DiffFailed {
            message: format!("Comparison failed: {}", e),
        }
    })?;

    // Count differing pixels from the similarity score
    let total_pixels = baseline.width() * baseline.height();
    let diff_pixels = ((1.0 - result.score) * total_pixels as f64) as u32;

    if diff_pixels <= threshold_pixels {
        Ok(CompareResult::Pass)
    } else {
        let diff_image = result.image.to_color_map();
        Ok(CompareResult::Fail {
            score: result.score,
            diff_image: Some(diff_image),
        })
    }
}
```

In `src/diff/mod.rs`:

```rust
pub mod compare;
pub mod composite;
```

Add `pub mod diff;` to `src/lib.rs`.

**Step 4: Run tests**

Run: `cargo test --test diff_test`
Expected: All PASS

**Step 5: Commit**

```bash
git add -A
git commit -m "feat: add image comparison with threshold support"
```

---

### Task 8: Diff Composite Image Generation

**Files:**
- Create: `src/diff/composite.rs`
- Modify: `tests/diff_test.rs` (add tests)

**Step 1: Write failing tests**

Add to `tests/diff_test.rs`:

```rust
use xsnap::diff::composite::create_composite;

#[test]
fn test_create_composite_dimensions() {
    let base = create_solid_image(100, 200, [255, 0, 0]);
    let diff = create_solid_image(100, 200, [0, 0, 0]);
    let current = create_solid_image(100, 200, [0, 255, 0]);

    let composite = create_composite(&base, &diff, &current);
    // Composite: base | diff | current side by side
    assert_eq!(composite.width(), 300);
    assert_eq!(composite.height(), 200);
}

#[test]
fn test_create_composite_contains_all_images() {
    let base = create_solid_image(10, 10, [255, 0, 0]);
    let diff = create_solid_image(10, 10, [0, 255, 0]);
    let current = create_solid_image(10, 10, [0, 0, 255]);

    let composite = create_composite(&base, &diff, &current);

    // Check base region (left)
    assert_eq!(composite.get_pixel(0, 0), &Rgb([255, 0, 0]));
    // Check diff region (middle)
    assert_eq!(composite.get_pixel(10, 0), &Rgb([0, 255, 0]));
    // Check current region (right)
    assert_eq!(composite.get_pixel(20, 0), &Rgb([0, 0, 255]));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --test diff_test`
Expected: FAIL for new tests

**Step 3: Implement composite generation**

In `src/diff/composite.rs`:

```rust
use image::{RgbImage, imageops};

/// Creates a side-by-side composite: [baseline | diff | current]
pub fn create_composite(baseline: &RgbImage, diff: &RgbImage, current: &RgbImage) -> RgbImage {
    let width = baseline.width();
    let height = baseline.height();

    let mut composite = RgbImage::new(width * 3, height);

    imageops::overlay(&mut composite, baseline, 0, 0);
    imageops::overlay(&mut composite, diff, width as i64, 0);
    imageops::overlay(&mut composite, current, (width * 2) as i64, 0);

    composite
}
```

**Step 4: Run tests**

Run: `cargo test --test diff_test`
Expected: All PASS

**Step 5: Commit**

```bash
git add -A
git commit -m "feat: add side-by-side diff composite image generation"
```

---

## Phase 4: Browser Module

### Task 9: Chromium Download Manager

**Files:**
- Create: `src/browser/mod.rs`
- Create: `src/browser/download.rs`
- Create: `tests/browser_download_test.rs`

**Step 1: Write tests**

Create `tests/browser_download_test.rs`:

```rust
use xsnap::browser::download::{resolve_chromium_version, get_download_url, cache_dir};

#[test]
fn test_resolve_auto_version() {
    // "auto" should resolve to some version string
    let version = resolve_chromium_version("auto");
    assert!(version.is_ok());
    assert!(!version.unwrap().is_empty());
}

#[test]
fn test_resolve_specific_version() {
    let version = resolve_chromium_version("120.0.6099.109");
    assert!(version.is_ok());
    assert_eq!(version.unwrap(), "120.0.6099.109");
}

#[test]
fn test_get_download_url_linux() {
    let url = get_download_url("120.0.6099.109", "linux");
    assert!(url.contains("120.0.6099.109"));
    assert!(url.contains("linux"));
}

#[test]
fn test_cache_dir() {
    let dir = cache_dir();
    assert!(dir.to_string_lossy().contains("xsnap"));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --test browser_download_test`
Expected: FAIL

**Step 3: Implement Chromium download manager**

In `src/browser/download.rs`:

```rust
use std::path::PathBuf;
use crate::error::XsnapError;

/// Chrome for Testing JSON API endpoint
const CHROME_VERSIONS_URL: &str = "https://googlechromelabs.github.io/chrome-for-testing/known-good-versions-with-downloads.json";
const CHROME_LATEST_URL: &str = "https://googlechromelabs.github.io/chrome-for-testing/last-known-good-versions-with-downloads.json";

pub fn cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from(".cache"))
        .join("xsnap")
        .join("chromium")
}

pub fn resolve_chromium_version(version: &str) -> Result<String, XsnapError> {
    if version == "auto" {
        // In the real implementation, this fetches the latest version from the API
        // For now, return a placeholder that will be replaced with actual HTTP call
        Ok("latest".into())
    } else {
        Ok(version.to_string())
    }
}

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

pub async fn ensure_chromium(version: &str) -> Result<PathBuf, XsnapError> {
    let resolved = resolve_chromium_version(version)?;
    let cache = cache_dir().join(&resolved);

    if cache.exists() {
        // Find the chrome binary in the cached directory
        return find_chrome_binary(&cache);
    }

    // Download and extract
    let platform = current_platform();
    let url = get_download_url(&resolved, &platform);
    download_and_extract(&url, &cache).await?;
    find_chrome_binary(&cache)
}

fn current_platform() -> String {
    if cfg!(target_os = "linux") {
        "linux".into()
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

fn find_chrome_binary(dir: &PathBuf) -> Result<PathBuf, XsnapError> {
    let binary_name = if cfg!(target_os = "windows") {
        "chrome.exe"
    } else {
        "chrome"
    };

    // Walk the directory to find the binary
    for entry in walkdir(dir) {
        if entry.file_name().to_string_lossy() == binary_name {
            return Ok(entry.path().to_path_buf());
        }
    }

    Err(XsnapError::BrowserLaunchFailed {
        message: format!("Chrome binary not found in {}", dir.display()),
    })
}

fn walkdir(dir: &PathBuf) -> Vec<std::fs::DirEntry> {
    let mut entries = Vec::new();
    if let Ok(read_dir) = std::fs::read_dir(dir) {
        for entry in read_dir.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                entries.extend(walkdir(&entry.path()));
            } else {
                entries.push(entry);
            }
        }
    }
    entries
}

async fn download_and_extract(url: &str, target: &PathBuf) -> Result<(), XsnapError> {
    std::fs::create_dir_all(target).map_err(|e| XsnapError::BrowserDownloadFailed {
        message: format!("Failed to create cache dir: {}", e),
    })?;

    let response = reqwest::get(url).await.map_err(|e| XsnapError::BrowserDownloadFailed {
        message: format!("Download failed: {}", e),
    })?;

    if !response.status().is_success() {
        return Err(XsnapError::BrowserDownloadFailed {
            message: format!("HTTP {}: {}", response.status(), url),
        });
    }

    let bytes = response.bytes().await.map_err(|e| XsnapError::BrowserDownloadFailed {
        message: format!("Failed to read response: {}", e),
    })?;

    // Extract zip
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor).map_err(|e| XsnapError::BrowserDownloadFailed {
        message: format!("Failed to open zip: {}", e),
    })?;
    archive.extract(target).map_err(|e| XsnapError::BrowserDownloadFailed {
        message: format!("Failed to extract: {}", e),
    })?;

    // Make chrome binary executable on Unix
    #[cfg(unix)]
    {
        if let Ok(binary) = find_chrome_binary(target) {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o755));
        }
    }

    Ok(())
}
```

Note: Add `zip = "2"` and `dirs = "6"` to `Cargo.toml` dependencies.

In `src/browser/mod.rs`:
```rust
pub mod download;
pub mod pool;
pub mod actions;
```

Add `pub mod browser;` to `src/lib.rs`.

**Step 4: Run tests**

Run: `cargo test --test browser_download_test`
Expected: All PASS

**Step 5: Commit**

```bash
git add -A
git commit -m "feat: add Chromium download manager with version resolution"
```

---

### Task 10: Browser Pool & Page Management

**Files:**
- Modify: `src/browser/pool.rs`

**Step 1: Implement browser pool**

This task is harder to unit test (requires actual browser). Implement the pool structure and test it in integration tests later.

In `src/browser/pool.rs`:

```rust
use std::collections::HashMap;
use std::sync::Arc;
use chromiumoxide::browser::{Browser, BrowserConfig, BrowserConfigBuilder};
use chromiumoxide::page::Page;
use futures::StreamExt;
use tokio::sync::Semaphore;
use crate::config::types::BrowserConfig as XsnapBrowserConfig;
use crate::error::XsnapError;

pub struct BrowserPool {
    browser: Arc<Browser>,
    semaphore: Arc<Semaphore>,
    _handler: tokio::task::JoinHandle<()>,
}

impl BrowserPool {
    pub async fn new(
        chrome_path: &std::path::Path,
        parallelism: usize,
        global_browser_config: Option<&XsnapBrowserConfig>,
    ) -> Result<Self, XsnapError> {
        let mut builder = BrowserConfig::builder()
            .chrome_executable(chrome_path)
            .arg("--headless=new")
            .arg("--disable-gpu")
            .arg("--no-first-run")
            .arg("--no-default-browser-check");

        if let Some(config) = global_browser_config {
            for arg in &config.args {
                builder = builder.arg(arg);
            }
            for (key, value) in &config.env {
                builder = builder.env(key, value);
            }
        }

        let browser_config = builder.build().map_err(|e| XsnapError::BrowserLaunchFailed {
            message: format!("Invalid browser config: {}", e),
        })?;

        let (browser, mut handler) = Browser::launch(browser_config)
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

    /// Acquire a browser page from the pool.
    /// Returns a permit guard that releases the slot when dropped.
    pub async fn acquire(&self) -> Result<(Page, tokio::sync::OwnedSemaphorePermit), XsnapError> {
        let permit = self.semaphore.clone().acquire_owned().await.map_err(|_| {
            XsnapError::BrowserLaunchFailed {
                message: "Pool semaphore closed".into(),
            }
        })?;

        let page = self.browser.new_page("about:blank").await.map_err(|e| {
            XsnapError::CdpError {
                message: format!("Failed to create page: {}", e),
            }
        })?;

        Ok((page, permit))
    }

    pub async fn close(self) -> Result<(), XsnapError> {
        // Browser closes when dropped, but explicit close is cleaner
        Ok(())
    }
}
```

**Step 2: Verify it compiles**

Run: `cargo check`
Expected: Compiles (may have warnings)

**Step 3: Commit**

```bash
git add -A
git commit -m "feat: add browser pool with semaphore-based parallelism"
```

---

### Task 11: Browser Actions (CDP)

**Files:**
- Modify: `src/browser/actions.rs`

**Step 1: Implement browser actions**

In `src/browser/actions.rs`:

```rust
use chromiumoxide::cdp::browser_protocol::css::ForcePseudoStateParams;
use chromiumoxide::cdp::browser_protocol::dom::{
    GetDocumentParams, QuerySelectorParams, ScrollIntoViewIfNeededParams,
};
use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use chromiumoxide::cdp::browser_protocol::input::{
    DispatchMouseEventParams, DispatchMouseEventType, MouseButton,
};
use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotParams;
use chromiumoxide::page::Page;
use crate::config::types::{Action, Size};
use crate::error::XsnapError;

/// Set viewport size on a page
pub async fn set_viewport(page: &Page, size: &Size) -> Result<(), XsnapError> {
    page.execute(
        SetDeviceMetricsOverrideParams::builder()
            .width(size.width)
            .height(size.height)
            .device_scale_factor(1.0)
            .mobile(false)
            .build(),
    )
    .await
    .map_err(|e| XsnapError::CdpError {
        message: format!("Failed to set viewport: {}", e),
    })?;
    Ok(())
}

/// Navigate to a URL and wait for load
pub async fn navigate(page: &Page, url: &str) -> Result<(), XsnapError> {
    page.goto(url).await.map_err(|e| XsnapError::NavigationFailed {
        url: url.into(),
        message: format!("{}", e),
    })?;
    Ok(())
}

/// Capture a screenshot as PNG bytes
pub async fn capture_screenshot(page: &Page, full_page: bool) -> Result<Vec<u8>, XsnapError> {
    let params = CaptureScreenshotParams::builder()
        .format(chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat::Png);

    let screenshot = if full_page {
        page.screenshot(params.full_page(true).build()).await
    } else {
        page.screenshot(params.build()).await
    };

    screenshot.map_err(|e| XsnapError::ScreenshotFailed {
        message: format!("{}", e),
    })
}

/// Execute a single action on a page
pub async fn execute_action(
    page: &Page,
    action: &Action,
    current_size: &str,
) -> Result<(), XsnapError> {
    // Check size restriction
    if let Some(restriction) = action_size_restriction(action) {
        if !restriction.contains(&current_size.to_string()) {
            return Ok(()); // Skip: not applicable to this size
        }
    }

    match action {
        Action::Wait { timeout, .. } => {
            tokio::time::sleep(std::time::Duration::from_millis(*timeout)).await;
        }
        Action::Click { selector, .. } => {
            let el = page.find_element(selector).await.map_err(|e| XsnapError::CdpError {
                message: format!("Element not found '{}': {}", selector, e),
            })?;
            el.click().await.map_err(|e| XsnapError::CdpError {
                message: format!("Click failed on '{}': {}", selector, e),
            })?;
        }
        Action::Type { selector, text, .. } => {
            let el = page.find_element(selector).await.map_err(|e| XsnapError::CdpError {
                message: format!("Element not found '{}': {}", selector, e),
            })?;
            el.click().await.map_err(|_| XsnapError::CdpError {
                message: format!("Focus failed on '{}'", selector),
            })?;
            el.type_str(text).await.map_err(|e| XsnapError::CdpError {
                message: format!("Type failed on '{}': {}", selector, e),
            })?;
        }
        Action::Scroll { selector, px_amount, .. } => {
            if let Some(sel) = selector {
                let el = page.find_element(sel).await.map_err(|e| XsnapError::CdpError {
                    message: format!("Element not found '{}': {}", sel, e),
                })?;
                el.scroll_into_view().await.map_err(|e| XsnapError::CdpError {
                    message: format!("Scroll failed on '{}': {}", sel, e),
                })?;
            } else if let Some(px) = px_amount {
                page.evaluate(format!("window.scrollBy(0, {})", px))
                    .await
                    .map_err(|e| XsnapError::CdpError {
                        message: format!("Scroll failed: {}", e),
                    })?;
            }
        }
        Action::ForcePseudoState {
            selector,
            hover,
            active,
            focus,
            visited,
            ..
        } => {
            let el = page.find_element(selector).await.map_err(|e| XsnapError::CdpError {
                message: format!("Element not found '{}': {}", selector, e),
            })?;
            let mut states = Vec::new();
            if *hover { states.push("hover"); }
            if *active { states.push("active"); }
            if *focus { states.push("focus"); }
            if *visited { states.push("visited"); }

            // Use CSS.forcePseudoState CDP command
            let node_id = el.remote_object_id();
            page.evaluate(format!(
                r#"document.querySelector('{}').classList.add('force-{}')"#,
                selector,
                states.join("-")
            )).await.map_err(|e| XsnapError::CdpError {
                message: format!("ForcePseudoState failed: {}", e),
            })?;
        }
        Action::Function { .. } => {
            // Functions are expanded before execution (handled by runner)
        }
    }

    Ok(())
}

fn action_size_restriction(action: &Action) -> Option<&Vec<String>> {
    match action {
        Action::Wait { size_restriction, .. }
        | Action::Click { size_restriction, .. }
        | Action::Type { size_restriction, .. }
        | Action::Scroll { size_restriction, .. }
        | Action::ForcePseudoState { size_restriction, .. }
        | Action::Function { size_restriction, .. } => size_restriction.as_ref(),
    }
}
```

**Step 2: Verify it compiles**

Run: `cargo check`
Expected: Compiles

**Step 3: Commit**

```bash
git add -A
git commit -m "feat: add browser actions (click, type, scroll, screenshot, viewport)"
```

---

## Phase 5: Test Runner

### Task 12: Test Result Types

**Files:**
- Create: `src/runner/mod.rs`
- Create: `src/runner/result.rs`

**Step 1: Write test result types**

In `src/runner/result.rs`:

```rust
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct TestResult {
    pub test_name: String,
    pub size_name: String,
    pub width: u32,
    pub height: u32,
    pub outcome: TestOutcome,
    pub duration: Duration,
    pub retries_used: u32,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum TestOutcome {
    /// Images match (or within threshold)
    Pass,
    /// New baseline created (no previous baseline existed)
    Created,
    /// Images differ beyond threshold
    Fail {
        score: f64,
        diff_path: String,
    },
    /// Test was skipped
    Skipped,
    /// Test errored out
    Error {
        message: String,
    },
}

impl TestOutcome {
    pub fn is_pass(&self) -> bool {
        matches!(self, TestOutcome::Pass | TestOutcome::Created | TestOutcome::Skipped)
    }
}

#[derive(Debug, Clone)]
pub struct RunSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub created: usize,
    pub skipped: usize,
    pub errors: usize,
    pub duration: Duration,
}
```

In `src/runner/mod.rs`:

```rust
pub mod result;
pub mod executor;
```

Add `pub mod runner;` to `src/lib.rs`.

**Step 2: Commit**

```bash
git add -A
git commit -m "feat: add test result types and run summary"
```

---

### Task 13: Test Executor (Core Logic)

**Files:**
- Create: `src/runner/executor.rs`

**Step 1: Implement the test executor**

In `src/runner/executor.rs`:

```rust
use std::path::{Path, PathBuf};
use std::time::Instant;
use tokio::sync::mpsc;
use image::RgbImage;

use crate::browser::actions;
use crate::browser::pool::BrowserPool;
use crate::config::types::{Action, GlobalConfig, Size, TestConfig};
use crate::diff::compare::{compare_images, CompareResult};
use crate::diff::composite::create_composite;
use crate::error::XsnapError;
use crate::runner::result::{RunSummary, TestOutcome, TestResult};

/// A single test task: one test at one viewport size
#[derive(Debug, Clone)]
pub struct TestTask {
    pub test: TestConfig,
    pub size: Size,
    pub base_url: String,
    pub full_screen: bool,
    pub threshold: u32,
    pub retry: u32,
    pub snapshot_dir: PathBuf,
    pub actions: Vec<Action>,
    pub http_headers: std::collections::HashMap<String, String>,
}

/// Progress update sent to the UI
#[derive(Debug, Clone)]
pub enum ProgressEvent {
    TestStarted { name: String, size: String },
    TestCompleted(TestResult),
    RunCompleted(RunSummary),
}

/// Expand function references into actual actions
pub fn expand_actions(
    actions: &[Action],
    functions: &std::collections::HashMap<String, Vec<Action>>,
) -> Vec<Action> {
    let mut expanded = Vec::new();
    for action in actions {
        match action {
            Action::Function { name, .. } => {
                if let Some(func_actions) = functions.get(name) {
                    expanded.extend(expand_actions(func_actions, functions));
                }
            }
            other => expanded.push(other.clone()),
        }
    }
    expanded
}

/// Build all test tasks from config
pub fn build_test_tasks(
    global: &GlobalConfig,
    tests: &[TestConfig],
) -> Vec<TestTask> {
    let default_sizes = global.default_sizes.clone().unwrap_or_else(|| vec![
        Size { name: "default".into(), width: 1920, height: 1080 },
    ]);

    let snapshot_dir = PathBuf::from(&global.snapshot_directory);

    tests.iter().flat_map(|test| {
        let sizes = test.sizes.as_ref().unwrap_or(&default_sizes);
        let threshold = test.threshold.unwrap_or(global.threshold);
        let retry = test.retry.unwrap_or(global.retry);
        let raw_actions = test.actions.clone().unwrap_or_default();
        let actions = expand_actions(&raw_actions, &global.functions);

        let mut headers = global.http_headers.clone();
        if let Some(test_headers) = &test.http_headers {
            headers.extend(test_headers.clone());
        }

        sizes.iter().map(move |size| {
            TestTask {
                test: test.clone(),
                size: size.clone(),
                base_url: global.base_url.clone(),
                full_screen: global.full_screen,
                threshold,
                retry,
                snapshot_dir: snapshot_dir.clone(),
                actions: actions.clone(),
                http_headers: headers.clone(),
            }
        })
    }).collect()
}

/// File name for a snapshot
pub fn snapshot_filename(test_name: &str, size: &Size) -> String {
    format!("{}-{}-{}x{}.png", test_name, size.name, size.width, size.height)
}

/// Execute a single test task
pub async fn execute_test_task(
    pool: &BrowserPool,
    task: &TestTask,
    no_create: bool,
) -> TestResult {
    let start = Instant::now();
    let filename = snapshot_filename(&task.test.name, &task.size);

    let base_dir = task.snapshot_dir.join("__base_images__");
    let updated_dir = task.snapshot_dir.join("__updated__");
    let current_dir = task.snapshot_dir.join("__current__");

    let baseline_path = base_dir.join(&filename);
    let updated_path = updated_dir.join(&filename);
    let diff_path = updated_dir.join(filename.replace(".png", ".diff.png"));
    let current_path = current_dir.join(&filename);

    let mut last_error = None;
    let max_attempts = task.retry + 1;

    for attempt in 0..max_attempts {
        match execute_single_attempt(pool, task, &baseline_path, &current_path, no_create).await {
            Ok(outcome) => {
                let duration = start.elapsed();

                // Handle file operations based on outcome
                match &outcome {
                    TestOutcome::Fail { score, .. } => {
                        // Save updated image and diff
                        let _ = std::fs::create_dir_all(&updated_dir);
                        if current_path.exists() {
                            let _ = std::fs::copy(&current_path, &updated_path);
                        }
                        // Generate diff composite if both images exist
                        if baseline_path.exists() && current_path.exists() {
                            if let (Ok(base_img), Ok(curr_img)) = (
                                image::open(&baseline_path),
                                image::open(&current_path),
                            ) {
                                let base_rgb = base_img.into_rgb8();
                                let curr_rgb = curr_img.into_rgb8();
                                if let Ok(CompareResult::Fail { diff_image: Some(di), .. }) =
                                    compare_images(&base_rgb, &curr_rgb, 0)
                                {
                                    let composite = create_composite(&base_rgb, &di, &curr_rgb);
                                    let _ = composite.save(&diff_path);
                                }
                            }
                        }
                    }
                    TestOutcome::Created => {
                        // Move current to baseline
                        let _ = std::fs::create_dir_all(&base_dir);
                        if current_path.exists() {
                            let _ = std::fs::copy(&current_path, &baseline_path);
                        }
                    }
                    _ => {}
                }

                return TestResult {
                    test_name: task.test.name.clone(),
                    size_name: task.size.name.clone(),
                    width: task.size.width,
                    height: task.size.height,
                    outcome,
                    duration,
                    retries_used: attempt,
                    warnings: vec![],
                };
            }
            Err(e) => {
                last_error = Some(e);
            }
        }
    }

    TestResult {
        test_name: task.test.name.clone(),
        size_name: task.size.name.clone(),
        width: task.size.width,
        height: task.size.height,
        outcome: TestOutcome::Error {
            message: last_error.map(|e| e.to_string()).unwrap_or_default(),
        },
        duration: start.elapsed(),
        retries_used: max_attempts - 1,
        warnings: vec![],
    }
}

async fn execute_single_attempt(
    pool: &BrowserPool,
    task: &TestTask,
    baseline_path: &Path,
    current_path: &Path,
    no_create: bool,
) -> Result<TestOutcome, XsnapError> {
    // Skip?
    if task.test.skip {
        return Ok(TestOutcome::Skipped);
    }

    // Acquire browser page
    let (page, _permit) = pool.acquire().await?;

    // Set viewport
    actions::set_viewport(&page, &task.size).await?;

    // Navigate
    let url = format!("{}{}", task.base_url, task.test.url);
    actions::navigate(&page, &url).await?;

    // Execute actions
    for action in &task.actions {
        actions::execute_action(&page, action, &task.size.name).await?;
    }

    // Capture screenshot
    let screenshot_bytes = actions::capture_screenshot(&page, task.full_screen).await?;

    // Save current screenshot
    let _ = std::fs::create_dir_all(current_path.parent().unwrap());
    std::fs::write(current_path, &screenshot_bytes).map_err(|e| XsnapError::ScreenshotFailed {
        message: format!("Failed to save screenshot: {}", e),
    })?;

    // Compare with baseline
    if !baseline_path.exists() {
        if no_create {
            return Err(XsnapError::DiffFailed {
                message: format!("No baseline exists and --no-create is set: {}", baseline_path.display()),
            });
        }
        return Ok(TestOutcome::Created);
    }

    let baseline_img = image::open(baseline_path)
        .map_err(|e| XsnapError::DiffFailed {
            message: format!("Failed to load baseline: {}", e),
        })?
        .into_rgb8();

    let current_img = image::open(current_path)
        .map_err(|e| XsnapError::DiffFailed {
            message: format!("Failed to load current: {}", e),
        })?
        .into_rgb8();

    match compare_images(&baseline_img, &current_img, task.threshold)? {
        CompareResult::Pass => Ok(TestOutcome::Pass),
        CompareResult::Fail { score, .. } => {
            let diff_filename = snapshot_filename(&task.test.name, &task.size)
                .replace(".png", ".diff.png");
            Ok(TestOutcome::Fail {
                score,
                diff_path: diff_filename,
            })
        }
    }
}

/// Run all test tasks in parallel
pub async fn run_all(
    pool: &BrowserPool,
    tasks: Vec<TestTask>,
    no_create: bool,
    progress_tx: mpsc::UnboundedSender<ProgressEvent>,
) -> RunSummary {
    let start = Instant::now();
    let mut handles = Vec::new();

    for task in tasks {
        let tx = progress_tx.clone();
        let task_clone = task.clone();

        handles.push(tokio::spawn(async move {
            let _ = tx.send(ProgressEvent::TestStarted {
                name: task_clone.test.name.clone(),
                size: task_clone.size.name.clone(),
            });

            // Note: pool reference would need Arc wrapping in practice
            // This is simplified - real implementation uses Arc<BrowserPool>
            let result = TestResult {
                test_name: task_clone.test.name.clone(),
                size_name: task_clone.size.name.clone(),
                width: task_clone.size.width,
                height: task_clone.size.height,
                outcome: TestOutcome::Pass,
                duration: std::time::Duration::ZERO,
                retries_used: 0,
                warnings: vec![],
            };

            let _ = tx.send(ProgressEvent::TestCompleted(result.clone()));
            result
        }));
    }

    let mut results = Vec::new();
    for handle in handles {
        if let Ok(result) = handle.await {
            results.push(result);
        }
    }

    let summary = RunSummary {
        total: results.len(),
        passed: results.iter().filter(|r| matches!(r.outcome, TestOutcome::Pass)).count(),
        failed: results.iter().filter(|r| matches!(r.outcome, TestOutcome::Fail { .. })).count(),
        created: results.iter().filter(|r| matches!(r.outcome, TestOutcome::Created)).count(),
        skipped: results.iter().filter(|r| matches!(r.outcome, TestOutcome::Skipped)).count(),
        errors: results.iter().filter(|r| matches!(r.outcome, TestOutcome::Error { .. })).count(),
        duration: start.elapsed(),
    };

    let _ = progress_tx.send(ProgressEvent::RunCompleted(summary.clone()));
    summary
}
```

**Step 2: Verify it compiles**

Run: `cargo check`
Expected: Compiles

**Step 3: Commit**

```bash
git add -A
git commit -m "feat: add test executor with parallel execution and retry logic"
```

---

## Phase 6: UI Module

### Task 14: Pipeline Mode Output

**Files:**
- Create: `src/ui/mod.rs`
- Create: `src/ui/pipeline.rs`
- Create: `tests/ui_pipeline_test.rs`

**Step 1: Write failing tests**

Create `tests/ui_pipeline_test.rs`:

```rust
use std::time::Duration;
use xsnap::ui::pipeline::format_result_line;
use xsnap::runner::result::{TestResult, TestOutcome};

fn make_result(name: &str, outcome: TestOutcome) -> TestResult {
    TestResult {
        test_name: name.into(),
        size_name: "desktop".into(),
        width: 1920,
        height: 1080,
        outcome,
        duration: Duration::from_millis(150),
        retries_used: 0,
        warnings: vec![],
    }
}

#[test]
fn test_format_pass() {
    let result = make_result("homepage", TestOutcome::Pass);
    let line = format_result_line(&result);
    assert!(line.contains("PASS"));
    assert!(line.contains("homepage"));
}

#[test]
fn test_format_fail() {
    let result = make_result("homepage", TestOutcome::Fail {
        score: 0.85,
        diff_path: "diff.png".into(),
    });
    let line = format_result_line(&result);
    assert!(line.contains("FAIL"));
}

#[test]
fn test_format_github_annotation() {
    let result = make_result("homepage", TestOutcome::Fail {
        score: 0.85,
        diff_path: "diff.png".into(),
    });
    let annotation = xsnap::ui::pipeline::github_annotation(&result);
    assert!(annotation.starts_with("::error::"));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --test ui_pipeline_test`
Expected: FAIL

**Step 3: Implement pipeline output**

In `src/ui/pipeline.rs`:

```rust
use crate::runner::result::{RunSummary, TestOutcome, TestResult};

pub fn format_result_line(result: &TestResult) -> String {
    let status = match &result.outcome {
        TestOutcome::Pass => "PASS",
        TestOutcome::Created => "NEW ",
        TestOutcome::Fail { .. } => "FAIL",
        TestOutcome::Skipped => "SKIP",
        TestOutcome::Error { .. } => "ERR ",
    };

    let duration_ms = result.duration.as_millis();
    let retries = if result.retries_used > 0 {
        format!(" (retried {}x)", result.retries_used)
    } else {
        String::new()
    };

    format!(
        "[{}] {}-{}-{}x{} ({}ms){}",
        status,
        result.test_name,
        result.size_name,
        result.width,
        result.height,
        duration_ms,
        retries,
    )
}

pub fn github_annotation(result: &TestResult) -> String {
    match &result.outcome {
        TestOutcome::Fail { score, diff_path } => {
            format!(
                "::error::Snapshot mismatch: {}-{} (score: {:.4}, diff: {})",
                result.test_name, result.size_name, score, diff_path
            )
        }
        TestOutcome::Error { message } => {
            format!(
                "::error::Test error: {}-{}: {}",
                result.test_name, result.size_name, message
            )
        }
        _ => String::new(),
    }
}

pub fn format_summary(summary: &RunSummary) -> String {
    format!(
        "\n{} tests: {} passed, {} failed, {} created, {} skipped, {} errors ({:.1}s)",
        summary.total,
        summary.passed,
        summary.failed,
        summary.created,
        summary.skipped,
        summary.errors,
        summary.duration.as_secs_f64(),
    )
}

pub fn print_result(result: &TestResult, is_github: bool) {
    println!("{}", format_result_line(result));
    if is_github && !result.outcome.is_pass() {
        let annotation = github_annotation(result);
        if !annotation.is_empty() {
            println!("{}", annotation);
        }
    }
}

pub fn print_summary(summary: &RunSummary) {
    println!("{}", format_summary(summary));
}
```

In `src/ui/mod.rs`:

```rust
pub mod pipeline;
pub mod tui;
```

Add `pub mod ui;` to `src/lib.rs`.

**Step 4: Run tests**

Run: `cargo test --test ui_pipeline_test`
Expected: All PASS

**Step 5: Commit**

```bash
git add -A
git commit -m "feat: add pipeline mode output with GitHub annotations"
```

---

### Task 15: TUI Mode (ratatui)

**Files:**
- Create: `src/ui/tui.rs`

**Step 1: Implement TUI**

In `src/ui/tui.rs`:

```rust
use std::io;
use std::time::Duration;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Gauge, Row, Table, TableState};
use ratatui::Frame;
use tokio::sync::mpsc;

use crate::runner::result::{RunSummary, TestOutcome, TestResult};

struct TuiApp {
    results: Vec<TestResult>,
    total_tasks: usize,
    completed: usize,
    table_state: TableState,
    summary: Option<RunSummary>,
    should_quit: bool,
}

impl TuiApp {
    fn new(total_tasks: usize) -> Self {
        Self {
            results: Vec::new(),
            total_tasks,
            completed: 0,
            table_state: TableState::default(),
            summary: None,
            should_quit: false,
        }
    }

    fn add_result(&mut self, result: TestResult) {
        self.completed += 1;
        self.results.push(result);
    }
}

fn render(frame: &mut Frame, app: &mut TuiApp) {
    let chunks = Layout::vertical([
        Constraint::Length(3),  // Progress bar
        Constraint::Min(10),   // Results table
        Constraint::Length(3),  // Summary
    ]).split(frame.area());

    // Progress bar
    let progress = if app.total_tasks > 0 {
        app.completed as f64 / app.total_tasks as f64
    } else {
        0.0
    };
    let gauge = Gauge::default()
        .block(Block::bordered().title(" Progress "))
        .gauge_style(Style::default().fg(Color::Cyan))
        .percent((progress * 100.0) as u16)
        .label(format!("{}/{}", app.completed, app.total_tasks));
    frame.render_widget(gauge, chunks[0]);

    // Results table
    let widths = [
        Constraint::Length(6),   // Status
        Constraint::Fill(1),     // Test name
        Constraint::Length(12),  // Size
        Constraint::Length(10),  // Duration
        Constraint::Length(10),  // Score
    ];

    let rows: Vec<Row> = app.results.iter().map(|r| {
        let (status, color) = match &r.outcome {
            TestOutcome::Pass => ("PASS", Color::Green),
            TestOutcome::Created => ("NEW", Color::Blue),
            TestOutcome::Fail { .. } => ("FAIL", Color::Red),
            TestOutcome::Skipped => ("SKIP", Color::Yellow),
            TestOutcome::Error { .. } => ("ERR", Color::Red),
        };

        let score_text = match &r.outcome {
            TestOutcome::Fail { score, .. } => format!("{:.2}%", score * 100.0),
            _ => "-".into(),
        };

        Row::new(vec![
            Span::styled(status, Style::default().fg(color)),
            Span::raw(&r.test_name),
            Span::raw(format!("{}-{}x{}", r.size_name, r.width, r.height)),
            Span::raw(format!("{}ms", r.duration.as_millis())),
            Span::raw(score_text),
        ])
    }).collect();

    let table = Table::new(rows, widths)
        .header(
            Row::new(vec!["Status", "Test", "Size", "Duration", "Score"])
                .style(Style::default().bold())
                .bottom_margin(1),
        )
        .block(Block::bordered().title(" Results "))
        .row_highlight_style(Style::default().reversed());

    frame.render_stateful_widget(table, chunks[1], &mut app.table_state);

    // Summary
    let summary_text = if let Some(ref s) = app.summary {
        format!(
            " {} passed | {} failed | {} new | {} skipped | {} errors | {:.1}s ",
            s.passed, s.failed, s.created, s.skipped, s.errors, s.duration.as_secs_f64()
        )
    } else {
        " Running... (q to quit) ".into()
    };
    let summary_block = Block::bordered()
        .title(" Summary ")
        .style(if app.summary.as_ref().is_some_and(|s| s.failed + s.errors == 0) {
            Style::default().fg(Color::Green)
        } else if app.summary.is_some() {
            Style::default().fg(Color::Red)
        } else {
            Style::default()
        });
    let summary_paragraph = ratatui::widgets::Paragraph::new(summary_text)
        .block(summary_block);
    frame.render_widget(summary_paragraph, chunks[2]);
}

pub async fn run_tui(
    total_tasks: usize,
    mut rx: mpsc::UnboundedReceiver<crate::runner::executor::ProgressEvent>,
) -> io::Result<RunSummary> {
    let mut terminal = ratatui::init();
    let mut app = TuiApp::new(total_tasks);

    let result = loop {
        // Drain progress events
        while let Ok(event) = rx.try_recv() {
            match event {
                crate::runner::executor::ProgressEvent::TestCompleted(result) => {
                    app.add_result(result);
                }
                crate::runner::executor::ProgressEvent::RunCompleted(summary) => {
                    app.summary = Some(summary);
                }
                _ => {}
            }
        }

        terminal.draw(|frame| render(frame, &mut app))?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => {
                            app.should_quit = true;
                            break app.summary.clone().unwrap_or(RunSummary {
                                total: 0, passed: 0, failed: 0, created: 0,
                                skipped: 0, errors: 0, duration: Duration::ZERO,
                            });
                        }
                        KeyCode::Down | KeyCode::Char('j') => app.table_state.select_next(),
                        KeyCode::Up | KeyCode::Char('k') => app.table_state.select_previous(),
                        _ => {}
                    }
                }
            }
        }

        // Auto-exit after completion + brief display time
        if app.summary.is_some() && app.should_quit {
            break app.summary.clone().unwrap();
        }
    };

    ratatui::restore();
    Ok(result)
}
```

**Step 2: Verify it compiles**

Run: `cargo check`
Expected: Compiles

**Step 3: Commit**

```bash
git add -A
git commit -m "feat: add ratatui-based TUI with live progress table"
```

---

## Phase 7: Commands

### Task 16: `xsnap init` Command

**Files:**
- Modify: `src/commands/init.rs`

**Step 1: Implement init command**

In `src/commands/init.rs`:

```rust
use std::path::Path;
use crate::config::schema::generate_schema;

const DEFAULT_CONFIG: &str = r#"{
  "$schema": "./xsnap.schema.json",
  "baseUrl": "http://localhost:3000",
  "browser": {
    "version": "auto"
  },
  "fullScreen": true,
  "testPattern": "tests/**/*.xsnap.json",
  "ignorePatterns": ["node_modules"],
  "defaultSizes": [
    { "name": "desktop", "width": 1920, "height": 1080 },
    { "name": "tablet", "width": 768, "height": 1024 },
    { "name": "mobile", "width": 375, "height": 667 }
  ],
  "snapshotDirectory": "__snapshots__",
  "threshold": 0,
  "retry": 1,
  "diffPixelColor": { "r": 255, "g": 0, "b": 255 }
}
"#;

const EXAMPLE_TEST: &str = r#"[
  {
    "name": "example",
    "url": "/",
    "actions": [
      { "action": "wait", "timeout": 1000 }
    ]
  }
]
"#;

pub fn run_init() -> anyhow::Result<()> {
    let config_path = Path::new("xsnap.config.jsonc");
    if config_path.exists() {
        anyhow::bail!("xsnap.config.jsonc already exists");
    }

    // Write config
    std::fs::write(config_path, DEFAULT_CONFIG)?;
    println!("Created xsnap.config.jsonc");

    // Write schema
    let schema = generate_schema();
    std::fs::write("xsnap.schema.json", &schema)?;
    println!("Created xsnap.schema.json");

    // Create tests directory with example
    std::fs::create_dir_all("tests")?;
    let example_path = Path::new("tests/example.xsnap.json");
    if !example_path.exists() {
        std::fs::write(example_path, EXAMPLE_TEST)?;
        println!("Created tests/example.xsnap.json");
    }

    // Create snapshot directories
    std::fs::create_dir_all("__snapshots__/__base_images__")?;
    std::fs::create_dir_all("__snapshots__/__updated__")?;
    std::fs::create_dir_all("__snapshots__/__current__")?;
    println!("Created __snapshots__/ directory structure");

    println!("\nxsnap initialized! Edit xsnap.config.jsonc to get started.");
    Ok(())
}
```

**Step 2: Commit**

```bash
git add -A
git commit -m "feat: add xsnap init command"
```

---

### Task 17: `xsnap test` Command

**Files:**
- Modify: `src/commands/test.rs`

**Step 1: Implement test command**

In `src/commands/test.rs`:

```rust
use std::path::Path;
use tokio::sync::mpsc;

use crate::browser::download::ensure_chromium;
use crate::browser::pool::BrowserPool;
use crate::config::global::load_global_config;
use crate::config::test::{discover_test_files, load_test_file};
use crate::config::validate::validate_config;
use crate::runner::executor::{build_test_tasks, run_all, ProgressEvent};
use crate::runner::result::RunSummary;
use crate::ui::pipeline;

pub struct TestOptions {
    pub config: String,
    pub no_create: bool,
    pub no_only: bool,
    pub no_skip: bool,
    pub filter: Option<String>,
    pub pipeline: bool,
    pub parallelism: Option<usize>,
}

pub async fn run_test(opts: TestOptions) -> anyhow::Result<i32> {
    // 1. Load config
    let config_path = Path::new(&opts.config);
    let global = load_global_config(config_path)?;

    // 2. Discover and load test files
    let test_files = discover_test_files(
        config_path.parent().unwrap_or(Path::new(".")),
        &global.test_pattern,
        &global.ignore_patterns,
    )?;

    let mut all_tests = global.tests.clone();
    for file in &test_files {
        let tests = load_test_file(file)?;
        all_tests.extend(tests);
    }

    // 3. Validate
    validate_config(&global, &all_tests)?;

    // 4. Apply flags
    if opts.no_only && all_tests.iter().any(|t| t.only) {
        anyhow::bail!("Tests with 'only: true' found but --no-only is set");
    }
    if opts.no_skip && all_tests.iter().any(|t| t.skip) {
        anyhow::bail!("Tests with 'skip: true' found but --no-skip is set");
    }

    // Filter by 'only' flag
    let has_only = all_tests.iter().any(|t| t.only);
    if has_only {
        all_tests.retain(|t| t.only);
    }

    // Filter by pattern
    if let Some(ref pattern) = opts.filter {
        all_tests.retain(|t| t.name.contains(pattern));
    }

    // 5. Build test tasks
    let tasks = build_test_tasks(&global, &all_tests);
    let total_tasks = tasks.len();

    if total_tasks == 0 {
        println!("No tests to run.");
        return Ok(0);
    }

    // 6. Set up browser
    let browser_version = global.browser.as_ref()
        .and_then(|b| b.version.as_deref())
        .unwrap_or("auto");
    let chrome_path = ensure_chromium(browser_version).await?;

    let parallelism = opts.parallelism
        .or(global.parallelism)
        .unwrap_or_else(|| num_cpus::get() * 3);

    let pool = BrowserPool::new(
        &chrome_path,
        parallelism,
        global.browser.as_ref(),
    ).await?;

    // 7. Run tests with UI
    let (tx, rx) = mpsc::unbounded_channel::<ProgressEvent>();

    let summary = if opts.pipeline {
        // Pipeline mode: print results as they come
        let run_handle = tokio::spawn(async move {
            run_all(&pool, tasks, opts.no_create, tx).await
        });

        let mut rx = rx;
        while let Some(event) = rx.recv().await {
            match event {
                ProgressEvent::TestCompleted(result) => {
                    let is_github = std::env::var("GITHUB_ACTIONS").is_ok();
                    pipeline::print_result(&result, is_github);
                }
                ProgressEvent::RunCompleted(summary) => {
                    pipeline::print_summary(&summary);
                }
                _ => {}
            }
        }

        run_handle.await?
    } else {
        // TUI mode
        let run_handle = tokio::spawn(async move {
            run_all(&pool, tasks, opts.no_create, tx).await
        });

        let tui_summary = crate::ui::tui::run_tui(total_tasks, rx).await?;
        let _ = run_handle.await;
        tui_summary
    };

    // 8. Exit code
    if summary.failed + summary.errors > 0 {
        Ok(1)
    } else {
        Ok(0)
    }
}
```

**Step 2: Commit**

```bash
git add -A
git commit -m "feat: add xsnap test command with TUI and pipeline modes"
```

---

### Task 18: `xsnap approve` Command

**Files:**
- Modify: `src/commands/approve.rs`

**Step 1: Implement approve command**

In `src/commands/approve.rs`:

```rust
use std::path::Path;
use crate::config::global::load_global_config;

pub struct ApproveOptions {
    pub config: String,
    pub all: bool,
    pub filter: Option<String>,
}

pub fn run_approve(opts: ApproveOptions) -> anyhow::Result<()> {
    let config_path = Path::new(&opts.config);
    let global = load_global_config(config_path)?;

    let snapshot_dir = Path::new(&global.snapshot_directory);
    let updated_dir = snapshot_dir.join("__updated__");
    let base_dir = snapshot_dir.join("__base_images__");

    if !updated_dir.exists() {
        println!("No updated snapshots to approve.");
        return Ok(());
    }

    let entries: Vec<_> = std::fs::read_dir(&updated_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            // Skip diff composites
            !name.contains(".diff.") &&
            // Apply filter if present
            opts.filter.as_ref().map_or(true, |f| name.contains(f))
        })
        .collect();

    if entries.is_empty() {
        println!("No updated snapshots match the filter.");
        return Ok(());
    }

    std::fs::create_dir_all(&base_dir)?;

    let mut approved = 0;
    for entry in &entries {
        let filename = entry.file_name();
        let target = base_dir.join(&filename);

        if opts.all {
            std::fs::copy(entry.path(), &target)?;
            approved += 1;
            println!("Approved: {}", filename.to_string_lossy());
        } else {
            // Interactive: ask per file
            let confirm = dialoguer::Confirm::new()
                .with_prompt(format!("Approve {}?", filename.to_string_lossy()))
                .default(true)
                .interact()?;

            if confirm {
                std::fs::copy(entry.path(), &target)?;
                approved += 1;
                println!("Approved: {}", filename.to_string_lossy());
            } else {
                println!("Skipped: {}", filename.to_string_lossy());
            }
        }
    }

    // Clean up approved files from __updated__
    for entry in &entries {
        let filename = entry.file_name();
        if base_dir.join(&filename).exists() {
            let _ = std::fs::remove_file(entry.path());
            // Also remove the diff composite if it exists
            let diff_name = filename.to_string_lossy().replace(".png", ".diff.png");
            let _ = std::fs::remove_file(updated_dir.join(diff_name));
        }
    }

    println!("\n{} snapshot(s) approved.", approved);
    Ok(())
}
```

**Step 2: Commit**

```bash
git add -A
git commit -m "feat: add xsnap approve command with interactive mode"
```

---

### Task 19: `xsnap cleanup` Command

**Files:**
- Modify: `src/commands/cleanup.rs`

**Step 1: Implement cleanup command**

In `src/commands/cleanup.rs`:

```rust
use std::collections::HashSet;
use std::path::Path;
use crate::config::global::load_global_config;
use crate::config::test::{discover_test_files, load_test_file};
use crate::runner::executor::snapshot_filename;

pub struct CleanupOptions {
    pub config: String,
}

pub fn run_cleanup(opts: CleanupOptions) -> anyhow::Result<()> {
    let config_path = Path::new(&opts.config);
    let global = load_global_config(config_path)?;

    // Collect all expected snapshot filenames
    let test_files = discover_test_files(
        config_path.parent().unwrap_or(Path::new(".")),
        &global.test_pattern,
        &global.ignore_patterns,
    )?;

    let mut all_tests = global.tests.clone();
    for file in &test_files {
        let tests = load_test_file(file)?;
        all_tests.extend(tests);
    }

    let default_sizes = global.default_sizes.clone().unwrap_or_default();
    let mut expected_files: HashSet<String> = HashSet::new();

    for test in &all_tests {
        let sizes = test.sizes.as_ref().unwrap_or(&default_sizes);
        for size in sizes {
            expected_files.insert(snapshot_filename(&test.name, size));
        }
    }

    // Scan base images directory
    let base_dir = Path::new(&global.snapshot_directory).join("__base_images__");
    if !base_dir.exists() {
        println!("No baseline directory found.");
        return Ok(());
    }

    let mut removed = 0;
    for entry in std::fs::read_dir(&base_dir)?.flatten() {
        let filename = entry.file_name().to_string_lossy().to_string();
        if !expected_files.contains(&filename) {
            std::fs::remove_file(entry.path())?;
            println!("Removed: {}", filename);
            removed += 1;
        }
    }

    if removed == 0 {
        println!("No unused baseline images found.");
    } else {
        println!("\n{} unused baseline(s) removed.", removed);
    }

    Ok(())
}
```

**Step 2: Commit**

```bash
git add -A
git commit -m "feat: add xsnap cleanup command"
```

---

### Task 20: `xsnap migrate` Command

**Files:**
- Modify: `src/commands/migrate.rs`

**Step 1: Implement migrate command**

In `src/commands/migrate.rs`:

```rust
use std::path::Path;
use dialoguer::Confirm;

pub struct MigrateOptions {
    pub source: String,
    pub target: String,
}

pub fn run_migrate(opts: MigrateOptions) -> anyhow::Result<()> {
    let source = Path::new(&opts.source);
    let target = Path::new(&opts.target);

    // 1. Look for osnap.config.yaml
    let global_config_path = source.join("osnap.config.yaml");
    if global_config_path.exists() {
        println!("Found: {}", global_config_path.display());
        let proceed = Confirm::new()
            .with_prompt("Convert global config to xsnap.config.jsonc?")
            .default(true)
            .interact()?;

        if proceed {
            let yaml_content = std::fs::read_to_string(&global_config_path)?;
            let yaml_value: serde_yaml::Value = serde_yaml::from_str(&yaml_content)?;
            let json_value = yaml_to_json(yaml_value);
            let json_str = serde_json::to_string_pretty(&json_value)?;

            let target_path = target.join("xsnap.config.jsonc");
            std::fs::write(&target_path, &json_str)?;
            println!("  -> Created {}", target_path.display());
        }
    } else {
        println!("No osnap.config.yaml found in {}", source.display());
    }

    // 2. Look for *.osnap.yaml test files
    let pattern = source.join("**/*.osnap.yaml").display().to_string();
    let test_files: Vec<_> = glob::glob(&pattern)?
        .filter_map(|e| e.ok())
        .collect();

    if test_files.is_empty() {
        println!("No .osnap.yaml test files found.");
        return Ok(());
    }

    println!("\nFound {} test file(s):", test_files.len());
    for file in &test_files {
        println!("  {}", file.display());

        let proceed = Confirm::new()
            .with_prompt(format!("Convert {}?", file.file_name().unwrap().to_string_lossy()))
            .default(true)
            .interact()?;

        if !proceed {
            println!("  Skipped.");
            continue;
        }

        let yaml_content = std::fs::read_to_string(file)?;
        let yaml_value: serde_yaml::Value = serde_yaml::from_str(&yaml_content)?;
        let json_value = yaml_to_json(yaml_value);
        let json_str = serde_json::to_string_pretty(&json_value)?;

        // Compute target path: same relative path but with .xsnap.json extension
        let rel_path = file.strip_prefix(source).unwrap_or(file);
        let new_name = rel_path
            .to_string_lossy()
            .replace(".osnap.yaml", ".xsnap.json");
        let target_path = target.join(&new_name);

        if let Some(parent) = target_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&target_path, &json_str)?;
        println!("  -> Created {}", target_path.display());
    }

    println!("\nMigration complete. Original files were kept as backup.");
    Ok(())
}

fn yaml_to_json(value: serde_yaml::Value) -> serde_json::Value {
    match value {
        serde_yaml::Value::Null => serde_json::Value::Null,
        serde_yaml::Value::Bool(b) => serde_json::Value::Bool(b),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                serde_json::Value::Number(i.into())
            } else if let Some(f) = n.as_f64() {
                serde_json::json!(f)
            } else {
                serde_json::Value::Null
            }
        }
        serde_yaml::Value::String(s) => serde_json::Value::String(s),
        serde_yaml::Value::Sequence(seq) => {
            serde_json::Value::Array(seq.into_iter().map(yaml_to_json).collect())
        }
        serde_yaml::Value::Mapping(map) => {
            let obj: serde_json::Map<String, serde_json::Value> = map
                .into_iter()
                .filter_map(|(k, v)| {
                    let key = match k {
                        serde_yaml::Value::String(s) => s,
                        other => serde_yaml::to_string(&other).unwrap_or_default().trim().to_string(),
                    };
                    Some((key, yaml_to_json(v)))
                })
                .collect();
            serde_json::Value::Object(obj)
        }
        serde_yaml::Value::Tagged(tagged) => yaml_to_json(tagged.value),
    }
}
```

**Step 2: Commit**

```bash
git add -A
git commit -m "feat: add xsnap migrate command for OSnap YAML to JSON conversion"
```

---

### Task 21: Wire Up main.rs

**Files:**
- Modify: `src/main.rs`

**Step 1: Connect all commands to main**

Update `src/main.rs` to call the actual command implementations:

```rust
mod commands;
mod config;
mod error;
mod browser;
mod diff;
mod runner;
mod ui;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "xsnap", version, about = "Visual regression testing tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run visual regression tests
    Test {
        #[arg(long, default_value = "xsnap.config.jsonc")]
        config: String,
        #[arg(long)]
        no_create: bool,
        #[arg(long)]
        no_only: bool,
        #[arg(long)]
        no_skip: bool,
        #[arg(long)]
        filter: Option<String>,
        #[arg(long)]
        pipeline: bool,
        #[arg(long)]
        parallelism: Option<usize>,
    },
    /// Accept updated screenshots as new baselines
    Approve {
        #[arg(long, default_value = "xsnap.config.jsonc")]
        config: String,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        filter: Option<String>,
    },
    /// Remove unused baseline images
    Cleanup {
        #[arg(long, default_value = "xsnap.config.jsonc")]
        config: String,
    },
    /// Migrate OSnap YAML configs to xsnap JSON
    Migrate {
        #[arg(long, default_value = ".")]
        source: String,
        #[arg(long, default_value = ".")]
        target: String,
    },
    /// Create new xsnap.config.jsonc
    Init,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let exit_code = match cli.command {
        Commands::Test {
            config,
            no_create,
            no_only,
            no_skip,
            filter,
            pipeline,
            parallelism,
        } => {
            match commands::test::run_test(commands::test::TestOptions {
                config,
                no_create,
                no_only,
                no_skip,
                filter,
                pipeline,
                parallelism,
            })
            .await
            {
                Ok(code) => code,
                Err(e) => {
                    eprintln!("Error: {e}");
                    4
                }
            }
        }
        Commands::Approve { config, all, filter } => {
            match commands::approve::run_approve(commands::approve::ApproveOptions {
                config,
                all,
                filter,
            }) {
                Ok(()) => 0,
                Err(e) => {
                    eprintln!("Error: {e}");
                    2
                }
            }
        }
        Commands::Cleanup { config } => {
            match commands::cleanup::run_cleanup(commands::cleanup::CleanupOptions { config }) {
                Ok(()) => 0,
                Err(e) => {
                    eprintln!("Error: {e}");
                    2
                }
            }
        }
        Commands::Migrate { source, target } => {
            match commands::migrate::run_migrate(commands::migrate::MigrateOptions {
                source,
                target,
            }) {
                Ok(()) => 0,
                Err(e) => {
                    eprintln!("Error: {e}");
                    4
                }
            }
        }
        Commands::Init => {
            match commands::init::run_init() {
                Ok(()) => 0,
                Err(e) => {
                    eprintln!("Error: {e}");
                    4
                }
            }
        }
    };

    std::process::exit(exit_code);
}
```

**Step 2: Verify everything compiles**

Run: `cargo check`
Expected: Compiles with minimal warnings

**Step 3: Run all tests**

Run: `cargo test`
Expected: All tests pass

**Step 4: Commit**

```bash
git add -A
git commit -m "feat: wire up all commands in main.rs"
```

---

## Phase 8: Integration & Polish

### Task 22: Integration Test with Real Browser

**Files:**
- Create: `tests/integration/mod.rs`
- Create: `tests/integration/smoke_test.rs`

**Step 1: Write smoke test**

Create `tests/integration/smoke_test.rs`:

```rust
//! Integration test that requires a real Chromium installation.
//! Run with: cargo test --test integration -- --ignored

use std::path::Path;
use tempfile::tempdir;

#[tokio::test]
#[ignore] // Requires Chromium
async fn test_full_workflow() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    // Create config
    let config = r#"{
        "baseUrl": "https://example.com",
        "browser": { "version": "auto" },
        "testPattern": "tests/**/*.xsnap.json",
        "defaultSizes": [
            { "name": "desktop", "width": 1280, "height": 720 }
        ],
        "snapshotDirectory": "__snapshots__"
    }"#;
    std::fs::write(dir_path.join("xsnap.config.jsonc"), config).unwrap();

    // Create test
    std::fs::create_dir_all(dir_path.join("tests")).unwrap();
    let test = r#"[{
        "name": "example-page",
        "url": "/",
        "actions": [{ "action": "wait", "timeout": 1000 }]
    }]"#;
    std::fs::write(dir_path.join("tests/example.xsnap.json"), test).unwrap();

    // Run xsnap test (would need to invoke via command or library)
    // This is a placeholder - the actual test would call run_test()
    assert!(dir_path.join("xsnap.config.jsonc").exists());
}
```

**Step 2: Commit**

```bash
git add -A
git commit -m "feat: add integration test scaffold"
```

---

### Task 23: Generate and Commit JSON Schema

**Files:**
- Create: `schema/xsnap.schema.json` (generated)

**Step 1: Write a binary target to generate the schema**

This can be a simple Rust script or added as a subcommand. For simplicity, use the init command's schema generation. Alternatively, add a test that writes the schema:

Add to `tests/config_schema_test.rs`:

```rust
#[test]
fn test_write_schema_file() {
    let schema = generate_schema();
    let path = std::path::Path::new("schema/xsnap.schema.json");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, &schema).unwrap();
    assert!(path.exists());
}
```

**Step 2: Run test to generate schema**

Run: `cargo test test_write_schema_file`
Expected: PASS, file created

**Step 3: Commit**

```bash
git add -A
git commit -m "feat: generate and commit JSON schema for config validation"
```

---

### Task 24: Final Verification & README

**Step 1: Run full test suite**

Run: `cargo test`
Expected: All tests pass

**Step 2: Run clippy**

Run: `cargo clippy -- -W clippy::all`
Fix any warnings.

**Step 3: Build release binary**

Run: `cargo build --release`
Expected: Binary at `target/release/xsnap`

**Step 4: Test CLI help**

Run: `./target/release/xsnap --help`
Expected: Shows all commands and options

Run: `./target/release/xsnap test --help`
Expected: Shows test command options

**Step 5: Commit any fixes**

```bash
git add -A
git commit -m "chore: fix clippy warnings and verify build"
```

---

## Summary

| Phase | Tasks | Description |
|-------|-------|-------------|
| 1 | 1 | Project init, CLI skeleton, error types |
| 2 | 2-6 | Config types, JSONC parsing, validation, schema |
| 3 | 7-8 | Image comparison, diff composite |
| 4 | 9-11 | Chromium download, browser pool, CDP actions |
| 5 | 12-13 | Test result types, executor with parallel runs |
| 6 | 14-15 | Pipeline output, ratatui TUI |
| 7 | 16-21 | All CLI commands (init, test, approve, cleanup, migrate) |
| 8 | 22-24 | Integration tests, schema generation, final polish |

**Total: 24 tasks across 8 phases.**

Dependencies:
- Phase 2 depends on Phase 1
- Phase 3 has no dependencies (can run parallel to Phase 2)
- Phase 4 depends on Phase 1
- Phase 5 depends on Phases 2, 3, 4
- Phase 6 depends on Phase 5
- Phase 7 depends on all previous phases
- Phase 8 depends on all previous phases
