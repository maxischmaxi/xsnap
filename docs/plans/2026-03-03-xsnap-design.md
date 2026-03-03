# xsnap Design Document

## Overview

xsnap is a visual regression testing tool written in Rust that replaces OSnap.
It takes screenshots of web pages via headless Chromium (CDP), compares them against
baseline images, and reports visual differences.

**Key differentiators from OSnap:**
- JSON/JSONC config instead of YAML
- JSON Schema for config validation
- Custom Chrome args/env per test and globally
- Rust for performance and safety
- ratatui-based TUI with pipeline mode for CI
- `approve` command instead of manual file copying
- Flexible config: inline tests or separate files

## CLI Commands

```bash
xsnap test [OPTIONS]          # Run visual regression tests
  --config <path>             # Config path (default: xsnap.config.jsonc)
  --no-create                 # Don't create new baselines (CI)
  --no-only                   # Fail if any test has only: true (CI)
  --no-skip                   # Fail if any test has skip: true (CI)
  --filter <pattern>          # Only run matching tests
  --pipeline                  # Pipeline mode (no TUI, CI-optimized output)
  --parallelism <n>           # Override parallelism

xsnap approve [OPTIONS]       # Accept updated screenshots as new baselines
  --all                       # Approve all at once
  --filter <pattern>          # Only approve matching tests

xsnap cleanup                 # Remove unused baseline images

xsnap migrate [OPTIONS]       # Migrate OSnap YAML -> xsnap JSON (interactive)
  --source <dir>              # Source directory (default: .)
  --target <dir>              # Target directory (default: .)

xsnap init                    # Create new xsnap.config.jsonc
```

**Exit Codes:**
- `0` - All tests passed
- `1` - Test failures
- `2` - Config error
- `3` - Browser error
- `4` - Internal error

## Config Format

### Global Config (`xsnap.config.jsonc`)

```jsonc
{
  "$schema": "./xsnap.schema.json",
  "baseUrl": "http://localhost:3000",
  "browser": {
    "version": "auto",      // "auto" = latest, or specific e.g. "120.0.6099.109"
    "args": ["--disable-gpu", "--no-sandbox"],
    "env": { "DISPLAY": ":99" }
  },
  "fullScreen": true,
  "testPattern": "tests/**/*.xsnap.json",
  "ignorePatterns": ["node_modules", "target"],
  "defaultSizes": [
    { "name": "desktop", "width": 1920, "height": 1080 },
    { "name": "tablet", "width": 768, "height": 1024 },
    { "name": "mobile", "width": 375, "height": 667 }
  ],
  "functions": {
    "login": [
      { "action": "type", "selector": "#email", "text": "user@test.com" },
      { "action": "type", "selector": "#password", "text": "secret" },
      { "action": "click", "selector": "#login-btn" },
      { "action": "wait", "timeout": 1000 }
    ]
  },
  "snapshotDirectory": "__snapshots__",
  "threshold": 0,
  "retry": 1,
  "parallelism": null,
  "diffPixelColor": { "r": 255, "g": 0, "b": 255 },
  "httpHeaders": { "Authorization": "Bearer token" },
  "tests": []
}
```

### Test Config (`*.xsnap.json`)

```jsonc
[
  {
    "name": "homepage-hero",
    "url": "/hero",
    "threshold": 5,
    "retry": 2,
    "only": false,
    "skip": false,
    "sizes": [{ "name": "wide", "width": 2560, "height": 1440 }],
    "browser": {
      "args": ["--force-dark-mode"],
      "env": { "TZ": "Europe/Berlin" }
    },
    "actions": [
      { "action": "function", "name": "login" },
      { "action": "wait", "timeout": 500 },
      { "action": "scroll", "pxAmount": 200 }
    ],
    "ignore": [
      { "selector": ".timestamp" },
      { "selectorAll": ".ad-banner" },
      { "x1": 0, "y1": 0, "x2": 100, "y2": 50 },
      { "@": ["mobile"], "selector": ".sidebar" }
    ],
    "httpHeaders": { "Accept-Language": "de-DE" }
  }
]
```

### Supported Actions

| Action | Fields | Description |
|--------|--------|-------------|
| `wait` | `timeout` (ms) | Wait for network idle or timeout |
| `click` | `selector` | Click element center |
| `type` | `selector`, `text` | Type text into element |
| `scroll` | `selector` OR `pxAmount` | Scroll element into view or scroll by pixels |
| `forcePseudoState` | `selector`, `hover`, `active`, `focus`, `visited` | Force CSS pseudo-state |
| `function` | `name` | Execute reusable action sequence from global config |

All actions support size restriction via `"@": ["mobile", "tablet"]`.

## Architecture

```bash
CLI (clap)
  |
  v
Config Parser (serde_json + json_comments)
  - Parse xsnap.config.jsonc -> GlobalConfig
  - Parse *.xsnap.json -> Vec<TestConfig>
  - Merge global defaults + test overrides
  - Validate schema, duplicate names, undefined functions
  |
  v
Test Runner (tokio task pool)
  - Create Test x Size combinations
  - Distribute across N parallel browser targets
  - Send progress updates to UI channel
  |
  v (per test task)
Browser Controller (chromiumoxide)
  1. Download & launch Chromium
  2. Create target/tab
  3. Set viewport (Emulation.setDeviceMetricsOverride)
  4. Navigate URL + wait for networkIdle
  5. Execute actions (click, type, scroll, etc.)
  6. Capture screenshot (Page.captureScreenshot)
  |
  v
Diff Engine (image-compare + image crate)
  1. Load baseline (or: new baseline = auto-pass)
  2. Mask ignore regions
  3. Pixel comparison
  4. Generate diff composite (Base | Diff | New)
  5. diffCount vs threshold -> Pass/Fail
  |
  v
Output Handler
  - TUI mode: ratatui live table + progress
  - Pipeline mode: one line per result, GitHub annotations
  - File output: __snapshots__/ directory
```

### Rust Crates

| Crate | Purpose |
|-------|---------|
| `clap` | CLI argument parsing |
| `serde` + `serde_json` | JSON serialization |
| `json_comments` | Strip JSONC comments |
| `chromiumoxide` | CDP browser control |
| `tokio` | Async runtime for parallelism |
| `image` | PNG load/save |
| `image-compare` | Pixel diffing |
| `ratatui` + `crossterm` | Terminal UI |
| `indicatif` | Pipeline mode progress |
| `glob` | File pattern matching |
| `schemars` | JSON schema generation |
| `dialoguer` | Interactive migration prompts |
| `reqwest` | Chromium download |

## Project Structure

```text
xsnap/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── config/
│   │   ├── mod.rs
│   │   ├── global.rs        # GlobalConfig parsing
│   │   ├── test.rs          # TestConfig parsing
│   │   ├── schema.rs        # JSON schema generation
│   │   └── validate.rs      # Config validation
│   ├── browser/
│   │   ├── mod.rs
│   │   ├── download.rs      # Chromium auto-download
│   │   ├── pool.rs          # Browser target pool
│   │   └── actions.rs       # CDP actions
│   ├── diff/
│   │   ├── mod.rs
│   │   ├── compare.rs       # Image comparison
│   │   └── composite.rs     # Diff image generation
│   ├── runner/
│   │   ├── mod.rs            # Test runner
│   │   └── result.rs         # Test result types
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── tui.rs           # ratatui terminal UI
│   │   └── pipeline.rs      # Pipeline mode output
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── test.rs
│   │   ├── approve.rs
│   │   ├── cleanup.rs
│   │   ├── migrate.rs
│   │   └── init.rs
│   └── error.rs
├── schema/
│   └── xsnap.schema.json
└── tests/
    ├── config_test.rs
    ├── diff_test.rs
    └── integration/
```

## Snapshot Directory Structure

```text
__snapshots__/
├── __base_images__/
│   ├── homepage-hero-desktop-1920x1080.png
│   ├── homepage-hero-tablet-768x1024.png
│   └── homepage-hero-mobile-375x667.png
├── __updated__/
│   ├── homepage-hero-desktop-1920x1080.png
│   └── homepage-hero-desktop-1920x1080.diff.png
└── __current__/
    └── homepage-hero-desktop-1920x1080.png
```

**Naming convention:** `{test-name}-{size-name}-{width}x{height}.png`

## Browser Management

- `"version": "auto"` downloads the latest stable Chromium
- `"version": "120.0.6099.109"` downloads that exact version
- Downloads are cached locally (platform-specific binary)
- Browser args and env can be set globally and per-test (merged, test overrides global)

## Error Handling

| Error | Description |
|-------|-------------|
| `ConfigNotFound` | No xsnap.config.jsonc found |
| `ConfigInvalid` | JSON parse or schema validation error |
| `DuplicateTestName` | Same test name in different files |
| `UndefinedFunction` | Reference to non-existent function |
| `BrowserDownloadFailed` | Chromium download failed |
| `BrowserLaunchFailed` | Chromium won't start |
| `CdpError` | CDP communication error |
| `NavigationFailed` | Page unreachable / wrong response code |
| `ScreenshotFailed` | Screenshot capture failed |
| `DiffFailed` | Image comparison failed (e.g. dimension mismatch) |

**Retry logic:**
- Test failure: Retry up to `retry` times
- CDP error: Recreate target, retry test
- Navigation timeout: Retry once, then report as failure

## Pipeline Mode

- No TUI, structured text output (one line per test result)
- GitHub Actions compatible: `::error::` and `::warning::` annotations
- Clean exit codes for CI integration

## Migration Command

Interactive migration from OSnap YAML to xsnap JSON:
1. Scan for `osnap.config.yaml` and `*.osnap.yaml` files
2. Show each file to be converted, ask for confirmation
3. Convert YAML structure to equivalent JSON
4. Write new `.jsonc` / `.xsnap.json` files
5. Keep original files as backup
