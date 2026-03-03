# xsnap

Visual regression testing tool for web applications. Takes screenshots of your pages at configurable viewport sizes, compares them against baselines, and reports pixel-level differences.

Built in Rust. Uses Chrome for Testing via CDP (Chrome DevTools Protocol).

## Quick Start

```bash
# Build
cargo build --release

# Initialize a project
xsnap init

# Run tests (interactive TUI)
xsnap test

# Run tests (CI pipeline mode)
xsnap test --pipeline
```

`xsnap init` creates:
- `xsnap.config.jsonc` — global config (with `$schema` pointing to the latest JSON Schema on GitHub)
- `tests/example.xsnap.json` — example test file
- `__snapshots__/` — directory structure for baseline/current/updated images

## Configuration

### Global Config (`xsnap.config.jsonc`)

JSONC (JSON with comments) is supported. The `$schema` field enables editor autocompletion and validation.

```jsonc
{
  "$schema": "https://raw.githubusercontent.com/maxischmaxi/xsnap/main/xsnap.schema.json",
  // Base URL prepended to all test URLs
  "baseUrl": "http://localhost:3000",

  // Chrome version: "auto" downloads latest, or pin e.g. "131.0.6778.85"
  "browser": {
    "version": "auto",
    "args": ["--disable-gpu", "--no-sandbox"],
    "env": { "TZ": "UTC" }
  },

  // Capture full scrollable page (not just viewport)
  "fullScreen": true,

  // Glob pattern for discovering test files
  "testPattern": "tests/**/*.xsnap.json",

  // Glob patterns to exclude from test discovery
  "ignorePatterns": ["node_modules"],

  // Default viewport sizes applied to all tests (unless overridden per test)
  "defaultSizes": [
    { "name": "desktop", "width": 1920, "height": 1080 },
    { "name": "tablet", "width": 768, "height": 1024 },
    { "name": "mobile", "width": 375, "height": 667 }
  ],

  // Where snapshots are stored
  "snapshotDirectory": "__snapshots__",

  // Pixel diff threshold (0 = exact match)
  "threshold": 0,

  // Number of retries before marking a test as failed
  "retry": 1,

  // Max parallel browser instances (default: number of CPU cores)
  "parallelism": 4,

  // Color for highlighting diff pixels in composite images
  "diffPixelColor": { "r": 255, "g": 0, "b": 255 },

  // HTTP headers sent with every request
  "httpHeaders": {
    "Authorization": "Bearer token123"
  },

  // Reusable action sequences (referenced by name in tests)
  "functions": {
    "acceptCookies": [
      { "action": "click", "selector": "#cookie-accept" },
      { "action": "wait", "timeout": 500 }
    ]
  },

  // Tests can also be defined inline (alternative to separate files)
  "tests": []
}
```

### Test Files (`*.xsnap.json`)

Test files are JSON objects with a `tests` array, discovered via `testPattern`. The `$schema` field enables editor autocompletion and validation.

```json
{
  "$schema": "https://raw.githubusercontent.com/maxischmaxi/xsnap/main/xsnap.test.schema.json",
  "tests": [
    {
      "name": "homepage",
      "url": "/",
      "actions": [
        { "action": "function", "name": "acceptCookies" },
        { "action": "wait", "timeout": 1000 }
      ]
    },
    {
      "name": "login",
      "url": "/login",
      "only": false,
      "skip": false,
      "threshold": 10,
      "retry": 3,
      "sizes": [
        { "name": "mobile", "width": 375, "height": 667 }
      ],
      "browser": {
        "args": ["--force-dark-mode"]
      },
      "httpHeaders": {
        "X-Test": "value"
      },
      "actions": [
        { "action": "type", "selector": "#email", "text": "test@example.com" },
        { "action": "click", "selector": "#submit" },
        { "action": "wait", "timeout": 2000 }
      ],
      "ignore": [
        { "selector": ".dynamic-timestamp" },
        { "selectorAll": ".ad-banner" },
        { "x1": 0, "y1": 0, "x2": 200, "y2": 50 }
      ]
    }
  ]
}
```

### Test Config Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `name` | `string` | required | Unique test name (used in snapshot filenames) |
| `url` | `string` | required | Path appended to `baseUrl` |
| `threshold` | `number` | global value | Allowed pixel difference count |
| `retry` | `number` | global value | Retry attempts on failure |
| `only` | `bool` | `false` | When any test has `only: true`, only those run |
| `skip` | `bool` | `false` | Skip this test |
| `sizes` | `Size[]` | global `defaultSizes` | Override viewport sizes for this test |
| `browser` | `BrowserConfig` | — | Per-test Chrome args/env |
| `actions` | `Action[]` | — | Actions to execute before screenshot |
| `ignore` | `IgnoreRegion[]` | — | Regions to exclude from diff comparison |
| `httpHeaders` | `map` | — | Additional HTTP headers for this test |

### Actions

Actions run sequentially before each screenshot. All actions support an optional `"@"` field for size restriction (array of size names).

```jsonc
// Wait
{ "action": "wait", "timeout": 1000 }

// Click an element
{ "action": "click", "selector": "#button" }

// Type into an input
{ "action": "type", "selector": "#input", "text": "hello" }

// Scroll to element or by pixel amount
{ "action": "scroll", "selector": "#section" }
{ "action": "scroll", "pxAmount": 500 }

// Force CSS pseudo-state on an element
{ "action": "forcePseudoState", "selector": ".link", "hover": true, "focus": false, "active": false, "visited": false }

// Call a reusable function defined in global config
{ "action": "function", "name": "acceptCookies" }

// Size restriction: only execute on specific viewports
{ "action": "click", "selector": "#mobile-menu", "@": ["mobile"] }
```

## Commands

### `xsnap test`

Runs all visual regression tests.

```bash
xsnap test [OPTIONS]
```

| Flag | Description |
|------|-------------|
| `--config <path>` | Config file path (default: `xsnap.config.jsonc`) |
| `--pipeline` | CI-friendly output (no TUI, prints results line by line) |
| `--filter <pattern>` | Only run tests whose name contains `<pattern>` |
| `--no-create` | Don't auto-create baseline snapshots for new tests |
| `--no-only` | Ignore `only: true` on tests (run all) |
| `--no-skip` | Ignore `skip: true` on tests (run skipped tests too) |
| `--parallelism <n>` | Override parallel browser instance count |

**Modes:**
- **TUI mode** (default): Interactive terminal UI with live progress, color-coded results table, and summary bar. Press `q` to quit.
- **Pipeline mode** (`--pipeline`): Outputs one line per result. On GitHub Actions, emits `::error` annotations for failures.

**Outcomes:**
- **Pass** — current screenshot matches baseline within threshold
- **Created** — no baseline existed, current screenshot saved as new baseline
- **Fail** — pixel diff exceeds threshold, composite diff image generated
- **Skipped** — test was skipped via `skip: true`
- **Error** — browser/navigation/screenshot error

**Exit codes:**
- `0` — all tests passed
- `1` — one or more tests failed
- `2` — config error
- `3` — browser download/launch error
- `4` — runtime error (CDP, navigation, screenshot, diff)

### `xsnap approve`

Promotes updated snapshots as new baselines.

```bash
xsnap approve [OPTIONS]
```

| Flag | Description |
|------|-------------|
| `--config <path>` | Config file path (default: `xsnap.config.jsonc`) |
| `--all` | Approve all without interactive prompts |
| `--filter <pattern>` | Only approve snapshots matching `<pattern>` |

Moves files from `__snapshots__/__updated__/` to `__snapshots__/__base_images__/` and cleans up diff/current artifacts.

### `xsnap cleanup`

Removes orphaned baseline snapshots that no longer match any configured test + size combination.

```bash
xsnap cleanup [OPTIONS]
```

| Flag | Description |
|------|-------------|
| `--config <path>` | Config file path (default: `xsnap.config.jsonc`) |

### `xsnap migrate`

Converts OSnap YAML configs to xsnap JSON format. Interactive per-file confirmation.

```bash
xsnap migrate [OPTIONS]
```

| Flag | Description |
|------|-------------|
| `--source <dir>` | Source directory with OSnap files (default: `.`) |
| `--target <dir>` | Target directory for xsnap files (default: `.`) |

Converts:
- `osnap.config.yaml` -> `xsnap.config.jsonc`
- `*.osnap.yaml` / `*.osnap.yml` -> `*.xsnap.json`

### `xsnap init`

Scaffolds a new xsnap project in the current directory.

```bash
xsnap init
```

Creates `xsnap.config.jsonc`, `tests/example.xsnap.json`, and the `__snapshots__/` directory structure. The config references the JSON Schema directly from GitHub for always up-to-date editor autocompletion.

## Snapshot Directory Structure

```text
__snapshots__/
  __base_images__/    # Approved baseline screenshots (commit these)
  __current__/        # Current test run screenshots (gitignore)
  __updated__/        # Failed screenshots + diff composites (gitignore)
```

Snapshot filenames follow the pattern: `{test-name}-{size-name}-{width}x{height}.png`

## Chrome Management

xsnap auto-downloads Chrome for Testing. Binaries are cached per version in:

- Linux: `~/.cache/xsnap/chromium/<version>/`
- macOS: `~/Library/Caches/xsnap/chromium/<version>/`
- Windows: `%LOCALAPPDATA%/xsnap/chromium/<version>/`

Set `browser.version` to `"auto"` for latest stable, or pin a specific version like `"131.0.6778.85"`.

## Development

```bash
# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy -- -D warnings

# Set up pre-commit hooks (cargo fmt + clippy)
git config core.hooksPath .githooks
```

## Tech Stack

- **chromiumoxide** — Chrome DevTools Protocol client
- **image** + **image-compare** — pixel-level image diffing (RGB hybrid compare)
- **ratatui** + **crossterm** — terminal UI
- **clap** — CLI argument parsing
- **serde** + **json_comments** — JSONC config parsing
- **schemars** — JSON Schema generation (schema is committed to repo, referenced via GitHub raw URL)
- **tokio** — async runtime with semaphore-based browser pool
- **reqwest** — HTTP client for Chrome downloads
