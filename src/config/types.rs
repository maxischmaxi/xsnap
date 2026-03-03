use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// GlobalConfig
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GlobalConfig {
    pub base_url: String,

    #[serde(default)]
    pub browser: Option<BrowserConfig>,

    #[serde(default = "default_true")]
    pub full_screen: bool,

    #[serde(default = "default_test_pattern")]
    pub test_pattern: String,

    #[serde(default)]
    pub ignore_patterns: Vec<String>,

    #[serde(default)]
    pub default_sizes: Option<Vec<Size>>,

    #[serde(default)]
    pub functions: HashMap<String, Vec<Action>>,

    #[serde(default = "default_snapshot_dir")]
    pub snapshot_directory: String,

    #[serde(default)]
    pub threshold: u32,

    #[serde(default = "default_retry")]
    pub retry: u32,

    #[serde(default)]
    pub parallelism: Option<usize>,

    #[serde(default = "default_diff_color")]
    pub diff_pixel_color: Color,

    #[serde(default)]
    pub http_headers: HashMap<String, String>,

    #[serde(default)]
    pub tests: Vec<TestConfig>,
}

// ---------------------------------------------------------------------------
// BrowserConfig
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BrowserConfig {
    #[serde(default)]
    pub version: Option<String>,

    #[serde(default)]
    pub args: Vec<String>,

    #[serde(default)]
    pub env: HashMap<String, String>,
}

// ---------------------------------------------------------------------------
// Size
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Size {
    pub name: String,
    pub width: u32,
    pub height: u32,
}

// ---------------------------------------------------------------------------
// Color
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

// ---------------------------------------------------------------------------
// TestConfig
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TestConfig {
    pub name: String,
    pub url: String,

    #[serde(default)]
    pub threshold: Option<u32>,

    #[serde(default)]
    pub retry: Option<u32>,

    #[serde(default)]
    pub only: bool,

    #[serde(default)]
    pub skip: bool,

    #[serde(default)]
    pub expected_response_code: Option<u16>,

    #[serde(default)]
    pub sizes: Option<Vec<Size>>,

    #[serde(default)]
    pub browser: Option<BrowserConfig>,

    #[serde(default)]
    pub actions: Option<Vec<Action>>,

    #[serde(default)]
    pub ignore: Option<Vec<IgnoreRegion>>,

    #[serde(default)]
    pub http_headers: Option<HashMap<String, String>>,
}

// ---------------------------------------------------------------------------
// Action (internally tagged enum)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "camelCase")]
pub enum Action {
    Wait {
        timeout: u64,
        #[serde(rename = "@", default)]
        size_restriction: Option<Vec<String>>,
    },
    Click {
        selector: String,
        #[serde(rename = "@", default)]
        size_restriction: Option<Vec<String>>,
    },
    #[serde(rename = "type")]
    Type {
        selector: String,
        text: String,
        #[serde(rename = "@", default)]
        size_restriction: Option<Vec<String>>,
    },
    Scroll {
        #[serde(default)]
        selector: Option<String>,
        #[serde(default, rename = "pxAmount")]
        px_amount: Option<i32>,
        #[serde(rename = "@", default)]
        size_restriction: Option<Vec<String>>,
    },
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
    Function {
        name: String,
        #[serde(rename = "@", default)]
        size_restriction: Option<Vec<String>>,
    },
}

// ---------------------------------------------------------------------------
// IgnoreRegion (untagged enum)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum IgnoreRegion {
    Coordinates {
        x1: u32,
        y1: u32,
        x2: u32,
        y2: u32,
        #[serde(rename = "@", default)]
        size_restriction: Option<Vec<String>>,
    },
    Selector {
        selector: String,
        #[serde(rename = "@", default)]
        size_restriction: Option<Vec<String>>,
    },
    SelectorAll {
        #[serde(rename = "selectorAll")]
        selector_all: String,
        #[serde(rename = "@", default)]
        size_restriction: Option<Vec<String>>,
    },
}

// ---------------------------------------------------------------------------
// Default helpers
// ---------------------------------------------------------------------------

fn default_true() -> bool {
    true
}

fn default_test_pattern() -> String {
    "tests/**/*.xsnap.json".into()
}

fn default_snapshot_dir() -> String {
    "__snapshots__".into()
}

fn default_retry() -> u32 {
    1
}

fn default_diff_color() -> Color {
    Color {
        r: 255,
        g: 0,
        b: 255,
    }
}
