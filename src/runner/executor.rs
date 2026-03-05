use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use image::ImageReader;
use tokio::sync::mpsc;

use crate::browser::actions::{
    capture_screenshot, execute_action, navigate, set_extra_headers, set_viewport,
};
use crate::browser::pool::BrowserPool;
use crate::config::types::{Action, BrowserConfig, GlobalConfig, Size, TestConfig};
use crate::diff::compare::{CompareResult, compare_images};
use crate::diff::composite::create_composite;
use crate::error::XsnapError;
use crate::runner::result::{RunSummary, TestOutcome, TestResult};

// ---------------------------------------------------------------------------
// TestTask
// ---------------------------------------------------------------------------

/// A single test task: one test at one viewport size.
#[derive(Debug, Clone)]
pub struct TestTask {
    pub test: TestConfig,
    pub size: Size,
    pub base_url: String,
    pub full_screen: bool,
    pub threshold: u32,
    pub threshold_percent: f64,
    pub retry: u32,
    pub base_dir: PathBuf,
    pub diff_dir: PathBuf,
    pub updated_dir: PathBuf,
    pub actions: Vec<Action>,
    pub http_headers: HashMap<String, String>,
    pub browser_fingerprint: String,
}

// ---------------------------------------------------------------------------
// ProgressEvent
// ---------------------------------------------------------------------------

/// Progress update sent to the UI.
#[derive(Debug, Clone)]
pub enum ProgressEvent {
    ServerWaiting { url: String, elapsed_secs: u32 },
    ServerReady,
    TestStarted { name: String, size: String },
    TestCompleted(TestResult),
    RunCompleted(RunSummary),
}

// ---------------------------------------------------------------------------
// expand_actions
// ---------------------------------------------------------------------------

/// Expand function references into actual actions.
///
/// When an `Action::Function { name }` is encountered, it is replaced by the
/// actions defined in the `functions` map under that name. The expansion is
/// recursive so functions may reference other functions. Circular references
/// are detected and skipped to prevent infinite recursion.
pub fn expand_actions(actions: &[Action], functions: &HashMap<String, Vec<Action>>) -> Vec<Action> {
    let mut seen = HashSet::new();
    expand_actions_inner(actions, functions, &mut seen)
}

fn expand_actions_inner(
    actions: &[Action],
    functions: &HashMap<String, Vec<Action>>,
    seen: &mut HashSet<String>,
) -> Vec<Action> {
    let mut result = Vec::new();
    for action in actions {
        match action {
            Action::Function { name, .. } => {
                if seen.contains(name) {
                    // Circular reference detected, skip to prevent infinite recursion.
                    continue;
                }
                if let Some(fn_actions) = functions.get(name) {
                    // Track this function name to detect circular references.
                    seen.insert(name.clone());
                    // Recursively expand in case functions reference other functions.
                    let expanded = expand_actions_inner(fn_actions, functions, seen);
                    result.extend(expanded);
                    seen.remove(name);
                } else {
                    // If the function is not found, keep the action as-is.
                    // Validation should catch this earlier, but we are defensive.
                    result.push(action.clone());
                }
            }
            other => {
                result.push(other.clone());
            }
        }
    }
    result
}

// ---------------------------------------------------------------------------
// build_test_tasks
// ---------------------------------------------------------------------------

/// Default viewport sizes when none are specified.
fn default_sizes() -> Vec<Size> {
    vec![Size {
        name: "default".into(),
        width: 1280,
        height: 800,
    }]
}

/// Build all test tasks from a global config and list of test configs.
///
/// For each test, a `TestTask` is produced for every viewport size. Test-level
/// settings override global defaults where applicable.
///
/// Returns the list of tasks and a map from browser fingerprint to merged
/// `BrowserConfig` (used to create one `BrowserPool` per unique fingerprint).
pub fn build_test_tasks(
    global: &GlobalConfig,
    tests: &[TestConfig],
) -> (Vec<TestTask>, HashMap<String, Option<BrowserConfig>>) {
    let global_sizes = global
        .default_sizes
        .as_ref()
        .cloned()
        .unwrap_or_else(default_sizes);

    let mut tasks = Vec::new();
    let mut browser_configs: HashMap<String, Option<BrowserConfig>> = HashMap::new();

    for test in tests {
        let sizes = test.sizes.as_ref().unwrap_or(&global_sizes);
        let threshold = test.threshold.unwrap_or(global.threshold);
        let threshold_percent = test.threshold_percent.unwrap_or(global.threshold_percent);
        let retry = test.retry.unwrap_or(global.retry);

        // Merge HTTP headers: global first, then test overrides.
        let mut http_headers = global.http_headers.clone();
        if let Some(test_headers) = &test.http_headers {
            for (k, v) in test_headers {
                http_headers.insert(k.clone(), v.clone());
            }
        }

        // Merge browser config and compute fingerprint.
        let merged_browser = BrowserConfig::merge(global.browser.as_ref(), test.browser.as_ref());
        let fingerprint = merged_browser
            .as_ref()
            .map(|c| c.fingerprint())
            .unwrap_or_default();
        browser_configs
            .entry(fingerprint.clone())
            .or_insert_with(|| merged_browser.clone());

        // Expand function references in actions.
        let raw_actions = test.actions.as_deref().unwrap_or(&[]);
        let actions = expand_actions(raw_actions, &global.functions);

        for size in sizes {
            tasks.push(TestTask {
                test: test.clone(),
                size: size.clone(),
                base_url: global.base_url.clone(),
                full_screen: global.full_screen,
                threshold,
                threshold_percent,
                retry,
                base_dir: PathBuf::from(&global.base_directory),
                diff_dir: PathBuf::from(&global.diff_directory),
                updated_dir: PathBuf::from(&global.updated_directory),
                actions: actions.clone(),
                http_headers: http_headers.clone(),
                browser_fingerprint: fingerprint.clone(),
            });
        }
    }

    (tasks, browser_configs)
}

// ---------------------------------------------------------------------------
// snapshot_filename
// ---------------------------------------------------------------------------

/// Generate a deterministic filename for a snapshot image.
pub fn snapshot_filename(test_name: &str, size: &Size) -> String {
    format!(
        "{}-{}-{}x{}.png",
        test_name, size.name, size.width, size.height
    )
}

// ---------------------------------------------------------------------------
// execute_test_task
// ---------------------------------------------------------------------------

/// Execute a single test task and return the result.
///
/// This function:
/// 1. Acquires a page from the browser pool.
/// 2. Sets the viewport size.
/// 3. Navigates to the test URL.
/// 4. Executes any configured actions.
/// 5. Captures a screenshot.
/// 6. Compares with the baseline snapshot (if one exists).
/// 7. Returns a `TestResult` describing the outcome.
///
/// The `no_create` flag controls whether new snapshots can be created when no
/// baseline exists.
pub async fn execute_test_task(pool: &BrowserPool, task: &TestTask, no_create: bool) -> TestResult {
    let start = Instant::now();

    let filename = snapshot_filename(&task.test.name, &task.size);

    let snapshot_path = task.base_dir.join(&filename);

    let diff_stem = filename.trim_end_matches(".png");
    let diff_filename = format!("{}-diff.png", diff_stem);
    let diff_path = task.diff_dir.join(&diff_filename);

    // Retry loop.
    let mut last_outcome = TestOutcome::Error {
        message: "No attempts made".into(),
    };
    let mut retries_used = 0;
    let mut warnings = Vec::new();

    for attempt in 0..=task.retry {
        if attempt > 0 {
            retries_used = attempt;
        }

        match execute_single_attempt(pool, task, &snapshot_path, &diff_path, no_create).await {
            Ok((outcome, compare_warnings)) => {
                warnings.extend(compare_warnings);
                if outcome.is_pass() || attempt == task.retry {
                    last_outcome = outcome;
                    break;
                }
                // On failure with retries remaining, continue.
                last_outcome = outcome;
            }
            Err(e) => {
                warnings.push(format!("Attempt {}: {}", attempt + 1, e));
                last_outcome = TestOutcome::Error {
                    message: e.to_string(),
                };
                if attempt == task.retry {
                    break;
                }
            }
        }
    }

    TestResult {
        test_name: task.test.name.clone(),
        size_name: task.size.name.clone(),
        width: task.size.width,
        height: task.size.height,
        outcome: last_outcome,
        duration: start.elapsed(),
        retries_used,
        warnings,
    }
}

/// CPU-intensive image processing: decode, compare, and save snapshots.
///
/// Runs on a blocking thread (via `spawn_blocking`) to avoid starving the
/// tokio async runtime. Without this, synchronous image work (MSSIM comparison,
/// PNG decode/encode) would block tokio worker threads, preventing the browser
/// event handler and CDP commands from being processed.
#[allow(clippy::too_many_arguments)]
fn process_images(
    screenshot_bytes: Vec<u8>,
    snapshot_path: PathBuf,
    diff_path: PathBuf,
    updated_dir: PathBuf,
    threshold: u32,
    threshold_percent: f64,
    test_name: &str,
    size: &Size,
    no_create: bool,
) -> Result<(TestOutcome, Vec<String>), XsnapError> {
    // Decode screenshot into an image.
    let current_img = image::load_from_memory(&screenshot_bytes)
        .map_err(|e| XsnapError::ScreenshotFailed {
            message: format!("Failed to decode screenshot: {}", e),
        })?
        .to_rgb8();

    // Check if baseline exists.
    if !snapshot_path.exists() {
        if no_create {
            return Ok((
                TestOutcome::Error {
                    message: format!(
                        "No baseline snapshot exists and --no-create is set: {}",
                        snapshot_path.display()
                    ),
                },
                vec![],
            ));
        }

        // Create base directory if needed.
        if let Some(parent) = snapshot_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| XsnapError::ScreenshotFailed {
                message: format!("Failed to create base directory: {}", e),
            })?;
        }

        // Save the new baseline.
        current_img
            .save(&snapshot_path)
            .map_err(|e| XsnapError::ScreenshotFailed {
                message: format!("Failed to save snapshot: {}", e),
            })?;

        return Ok((TestOutcome::Created, vec![]));
    }

    // Load baseline image.
    let baseline_img = ImageReader::open(&snapshot_path)
        .map_err(|e| XsnapError::DiffFailed {
            message: format!("Failed to open baseline: {}", e),
        })?
        .decode()
        .map_err(|e| XsnapError::DiffFailed {
            message: format!("Failed to decode baseline: {}", e),
        })?
        .to_rgb8();

    // Compare images.
    let (compare_result, compare_warnings) =
        compare_images(&baseline_img, &current_img, threshold, threshold_percent)?;
    match compare_result {
        CompareResult::Pass => Ok((TestOutcome::Pass, compare_warnings)),
        CompareResult::Fail { score, diff_image } => {
            // Ensure diff directory exists.
            if let Some(parent) = diff_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| XsnapError::ScreenshotFailed {
                    message: format!("Failed to create diff directory: {}", e),
                })?;
            }

            // Save the diff composite if we have a diff image.
            let diff_path_str = diff_path.to_string_lossy().to_string();

            if let Some(diff_img) = diff_image {
                let composite = create_composite(&baseline_img, &diff_img, &current_img);
                if let Err(e) = composite.save(&diff_path) {
                    eprintln!("Warning: failed to save diff image: {}", e);
                }
            }

            // Save the current screenshot into updated directory for the approve workflow.
            let filename = snapshot_filename(test_name, size);
            std::fs::create_dir_all(&updated_dir).map_err(|e| XsnapError::ScreenshotFailed {
                message: format!("Failed to create updated directory: {}", e),
            })?;
            let updated_path = updated_dir.join(&filename);
            if let Err(e) = current_img.save(&updated_path) {
                eprintln!("Warning: failed to save updated screenshot: {}", e);
            }

            Ok((
                TestOutcome::Fail {
                    score,
                    diff_path: diff_path_str,
                },
                compare_warnings,
            ))
        }
    }
}

/// Execute a single attempt of a test task.
async fn execute_single_attempt(
    pool: &BrowserPool,
    task: &TestTask,
    snapshot_path: &Path,
    diff_path: &Path,
    no_create: bool,
) -> Result<(TestOutcome, Vec<String>), XsnapError> {
    // Acquire a page from the pool.
    let (page, _permit) = pool.acquire().await?;

    // Set viewport.
    set_viewport(&page, &task.size).await?;

    // Set extra HTTP headers before navigation.
    set_extra_headers(&page, &task.http_headers).await?;

    // TODO: Check expected_response_code if set.
    // This requires capturing the HTTP response from navigation.

    // TODO: Apply ignore regions by masking areas before comparison.
    // Coordinate regions: mask pixel areas directly.
    // Selector regions: query element bounds via CDP, then mask.

    // Build full URL.
    let full_url = if task.test.url.starts_with("http://") || task.test.url.starts_with("https://")
    {
        task.test.url.clone()
    } else {
        let base = task.base_url.trim_end_matches('/');
        let path = task.test.url.trim_start_matches('/');
        format!("{}/{}", base, path)
    };

    // Navigate.
    navigate(&page, &full_url).await?;

    // Execute actions.
    for action in &task.actions {
        execute_action(&page, action, &task.size.name).await?;
    }

    // Capture screenshot.
    let screenshot_bytes = capture_screenshot(&page, task.full_screen).await?;

    // Close the browser page explicitly to free CDP session resources, then
    // drop the handle. The semaphore permit is intentionally held until this
    // function returns, so that the CPU-intensive image comparison counts
    // toward the parallelism limit.
    let _ = page.close().await;

    // Move CPU-intensive image processing to a blocking thread to prevent
    // starving the tokio runtime (browser event handler, CDP commands).
    let snapshot_path = snapshot_path.to_path_buf();
    let diff_path = diff_path.to_path_buf();
    let updated_dir = task.updated_dir.clone();
    let threshold = task.threshold;
    let threshold_percent = task.threshold_percent;
    let test_name = task.test.name.clone();
    let size = task.size.clone();

    tokio::task::spawn_blocking(move || {
        process_images(
            screenshot_bytes,
            snapshot_path,
            diff_path,
            updated_dir,
            threshold,
            threshold_percent,
            &test_name,
            &size,
            no_create,
        )
    })
    .await
    .map_err(|e| XsnapError::ScreenshotFailed {
        message: format!("Image processing task panicked: {}", e),
    })?
}

// ---------------------------------------------------------------------------
// run_all
// ---------------------------------------------------------------------------

/// Run all test tasks with parallel execution and progress reporting.
pub async fn run_all(
    pools: Arc<HashMap<String, Arc<BrowserPool>>>,
    tasks: Vec<TestTask>,
    no_create: bool,
    parallelism: usize,
    progress_tx: Option<mpsc::UnboundedSender<ProgressEvent>>,
) -> RunSummary {
    use futures::stream::{self, StreamExt};

    let start = Instant::now();
    let total = tasks.len();

    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut created = 0usize;
    let mut skipped = 0usize;
    let mut errors = 0usize;

    let mut result_stream = stream::iter(tasks)
        .map(|task| {
            let pool = pools
                .get(&task.browser_fingerprint)
                .expect("pool must exist for fingerprint")
                .clone();
            let tx = progress_tx.clone();

            async move {
                if let Some(ref tx) = tx {
                    let _ = tx.send(ProgressEvent::TestStarted {
                        name: task.test.name.clone(),
                        size: format!(
                            "{}-{}x{}",
                            task.size.name, task.size.width, task.size.height
                        ),
                    });
                }

                let result = execute_test_task(&pool, &task, no_create).await;

                if let Some(ref tx) = tx {
                    let _ = tx.send(ProgressEvent::TestCompleted(result.clone()));
                }

                result
            }
        })
        .buffer_unordered(parallelism);

    while let Some(result) = result_stream.next().await {
        match &result.outcome {
            TestOutcome::Pass => passed += 1,
            TestOutcome::Created => created += 1,
            TestOutcome::Fail { .. } => failed += 1,
            TestOutcome::Skipped => skipped += 1,
            TestOutcome::Error { .. } => errors += 1,
        }
    }

    // Drop the stream to release the borrow on progress_tx.
    drop(result_stream);

    let summary = RunSummary {
        total,
        passed,
        failed,
        created,
        skipped,
        errors,
        duration: start.elapsed(),
    };

    if let Some(tx) = progress_tx {
        let _ = tx.send(ProgressEvent::RunCompleted(summary.clone()));
    }

    summary
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_filename() {
        let size = Size {
            name: "desktop".into(),
            width: 1920,
            height: 1080,
        };
        assert_eq!(
            snapshot_filename("homepage", &size),
            "homepage-desktop-1920x1080.png"
        );
    }

    #[test]
    fn test_snapshot_filename_special_chars() {
        let size = Size {
            name: "mobile".into(),
            width: 375,
            height: 812,
        };
        assert_eq!(
            snapshot_filename("login-page", &size),
            "login-page-mobile-375x812.png"
        );
    }

    #[test]
    fn test_expand_actions_no_functions() {
        let actions = vec![Action::Wait {
            timeout: 100,
            size_restriction: None,
        }];
        let functions = HashMap::new();
        let expanded = expand_actions(&actions, &functions);
        assert_eq!(expanded.len(), 1);
    }

    #[test]
    fn test_expand_actions_with_function() {
        let actions = vec![Action::Function {
            name: "login".into(),
            size_restriction: None,
        }];
        let mut functions = HashMap::new();
        functions.insert(
            "login".to_string(),
            vec![
                Action::Click {
                    selector: "#username".into(),
                    size_restriction: None,
                },
                Action::Type {
                    selector: "#username".into(),
                    text: "admin".into(),
                    size_restriction: None,
                },
            ],
        );
        let expanded = expand_actions(&actions, &functions);
        assert_eq!(expanded.len(), 2);
        assert!(matches!(&expanded[0], Action::Click { selector, .. } if selector == "#username"));
        assert!(matches!(&expanded[1], Action::Type { text, .. } if text == "admin"));
    }

    #[test]
    fn test_expand_actions_recursive() {
        let actions = vec![Action::Function {
            name: "setup".into(),
            size_restriction: None,
        }];
        let mut functions = HashMap::new();
        functions.insert(
            "setup".to_string(),
            vec![
                Action::Wait {
                    timeout: 50,
                    size_restriction: None,
                },
                Action::Function {
                    name: "login".into(),
                    size_restriction: None,
                },
            ],
        );
        functions.insert(
            "login".to_string(),
            vec![Action::Click {
                selector: "#btn".into(),
                size_restriction: None,
            }],
        );
        let expanded = expand_actions(&actions, &functions);
        assert_eq!(expanded.len(), 2);
        assert!(matches!(&expanded[0], Action::Wait { timeout: 50, .. }));
        assert!(matches!(&expanded[1], Action::Click { selector, .. } if selector == "#btn"));
    }

    #[test]
    fn test_expand_actions_unknown_function_preserved() {
        let actions = vec![Action::Function {
            name: "nonexistent".into(),
            size_restriction: None,
        }];
        let functions = HashMap::new();
        let expanded = expand_actions(&actions, &functions);
        assert_eq!(expanded.len(), 1);
        assert!(matches!(&expanded[0], Action::Function { name, .. } if name == "nonexistent"));
    }

    #[test]
    fn test_build_test_tasks_uses_global_defaults() {
        let global = GlobalConfig {
            base_url: "http://localhost:3000".into(),
            browser: None,
            full_screen: true,
            test_pattern: "tests/**/*.xsnap.jsonc".into(),
            ignore_patterns: vec![],
            default_sizes: Some(vec![
                Size {
                    name: "desktop".into(),
                    width: 1920,
                    height: 1080,
                },
                Size {
                    name: "mobile".into(),
                    width: 375,
                    height: 812,
                },
            ]),
            functions: HashMap::new(),
            base_directory: "__snapshots__/__base_images__".into(),
            diff_directory: "__snapshots__/__diff__".into(),
            updated_directory: "__snapshots__/__updated__".into(),
            threshold: 10,
            threshold_percent: 0.5,
            retry: 2,
            parallelism: None,
            diff_pixel_color: crate::config::types::Color {
                r: 255,
                g: 0,
                b: 255,
            },
            http_headers: HashMap::new(),
            start_command: None,
            tests: vec![],
        };

        let tests = vec![TestConfig {
            name: "homepage".into(),
            url: "/".into(),
            threshold: None,
            threshold_percent: None,
            retry: None,
            only: false,
            skip: false,
            expected_response_code: None,
            sizes: None,
            browser: None,
            actions: None,
            ignore: None,
            http_headers: None,
        }];

        let (tasks, browser_configs) = build_test_tasks(&global, &tests);
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].size.name, "desktop");
        assert_eq!(tasks[1].size.name, "mobile");
        assert_eq!(tasks[0].threshold, 10);
        assert_eq!(tasks[0].retry, 2);
        // No browser config → default empty fingerprint
        assert_eq!(tasks[0].browser_fingerprint, "");
        assert_eq!(browser_configs.len(), 1);
        assert!(browser_configs.contains_key(""));
    }

    #[test]
    fn test_build_test_tasks_test_overrides() {
        let global = GlobalConfig {
            base_url: "http://localhost:3000".into(),
            browser: None,
            full_screen: true,
            test_pattern: "tests/**/*.xsnap.jsonc".into(),
            ignore_patterns: vec![],
            default_sizes: Some(vec![Size {
                name: "desktop".into(),
                width: 1920,
                height: 1080,
            }]),
            functions: HashMap::new(),
            base_directory: "__snapshots__/__base_images__".into(),
            diff_directory: "__snapshots__/__diff__".into(),
            updated_directory: "__snapshots__/__updated__".into(),
            threshold: 10,
            threshold_percent: 0.5,
            retry: 2,
            parallelism: None,
            diff_pixel_color: crate::config::types::Color {
                r: 255,
                g: 0,
                b: 255,
            },
            http_headers: {
                let mut m = HashMap::new();
                m.insert("Authorization".into(), "Bearer global".into());
                m
            },
            start_command: None,
            tests: vec![],
        };

        let tests = vec![TestConfig {
            name: "login".into(),
            url: "/login".into(),
            threshold: Some(5),
            threshold_percent: None,
            retry: Some(3),
            only: false,
            skip: false,
            expected_response_code: None,
            sizes: Some(vec![Size {
                name: "tablet".into(),
                width: 768,
                height: 1024,
            }]),
            browser: None,
            actions: None,
            ignore: None,
            http_headers: Some({
                let mut m = HashMap::new();
                m.insert("X-Test".into(), "true".into());
                m
            }),
        }];

        let (tasks, _browser_configs) = build_test_tasks(&global, &tests);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].size.name, "tablet");
        assert_eq!(tasks[0].threshold, 5);
        assert_eq!(tasks[0].retry, 3);
        assert_eq!(
            tasks[0].http_headers.get("Authorization").unwrap(),
            "Bearer global"
        );
        assert_eq!(tasks[0].http_headers.get("X-Test").unwrap(), "true");
    }

    #[test]
    fn test_build_test_tasks_default_size_when_none_configured() {
        let global = GlobalConfig {
            base_url: "http://localhost:3000".into(),
            browser: None,
            full_screen: true,
            test_pattern: "tests/**/*.xsnap.jsonc".into(),
            ignore_patterns: vec![],
            default_sizes: None,
            functions: HashMap::new(),
            base_directory: "__snapshots__/__base_images__".into(),
            diff_directory: "__snapshots__/__diff__".into(),
            updated_directory: "__snapshots__/__updated__".into(),
            threshold: 0,
            threshold_percent: 0.5,
            retry: 1,
            parallelism: None,
            diff_pixel_color: crate::config::types::Color {
                r: 255,
                g: 0,
                b: 255,
            },
            http_headers: HashMap::new(),
            start_command: None,
            tests: vec![],
        };

        let tests = vec![TestConfig {
            name: "test".into(),
            url: "/test".into(),
            threshold: None,
            threshold_percent: None,
            retry: None,
            only: false,
            skip: false,
            expected_response_code: None,
            sizes: None,
            browser: None,
            actions: None,
            ignore: None,
            http_headers: None,
        }];

        let (tasks, _browser_configs) = build_test_tasks(&global, &tests);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].size.name, "default");
        assert_eq!(tasks[0].size.width, 1280);
        assert_eq!(tasks[0].size.height, 800);
    }

    #[test]
    fn test_browser_config_fingerprint_deterministic() {
        let config = BrowserConfig {
            version: Some("120".into()),
            args: vec!["--lang=de".into(), "--disable-gpu".into()],
            env: {
                let mut m = HashMap::new();
                m.insert("LANG".into(), "de_DE".into());
                m
            },
        };
        // Calling twice gives same result
        assert_eq!(config.fingerprint(), config.fingerprint());
        // Args are sorted, so order doesn't matter
        let config2 = BrowserConfig {
            version: Some("120".into()),
            args: vec!["--disable-gpu".into(), "--lang=de".into()],
            env: {
                let mut m = HashMap::new();
                m.insert("LANG".into(), "de_DE".into());
                m
            },
        };
        assert_eq!(config.fingerprint(), config2.fingerprint());
    }

    #[test]
    fn test_browser_config_fingerprint_empty() {
        let config = BrowserConfig {
            version: None,
            args: vec![],
            env: HashMap::new(),
        };
        assert_eq!(config.fingerprint(), "");
    }

    #[test]
    fn test_browser_config_merge_both_none() {
        assert!(BrowserConfig::merge(None, None).is_none());
    }

    #[test]
    fn test_browser_config_merge_global_only() {
        let global = BrowserConfig {
            version: Some("120".into()),
            args: vec!["--no-sandbox".into()],
            env: HashMap::new(),
        };
        let merged = BrowserConfig::merge(Some(&global), None).unwrap();
        assert_eq!(merged.args, vec!["--no-sandbox"]);
        assert_eq!(merged.version, Some("120".into()));
    }

    #[test]
    fn test_browser_config_merge_test_only() {
        let test = BrowserConfig {
            version: None,
            args: vec!["--lang=de".into()],
            env: HashMap::new(),
        };
        let merged = BrowserConfig::merge(None, Some(&test)).unwrap();
        assert_eq!(merged.args, vec!["--lang=de"]);
    }

    #[test]
    fn test_browser_config_merge_combined() {
        let global = BrowserConfig {
            version: Some("120".into()),
            args: vec!["--no-sandbox".into()],
            env: {
                let mut m = HashMap::new();
                m.insert("LANG".into(), "en_US".into());
                m
            },
        };
        let test = BrowserConfig {
            version: Some("121".into()),
            args: vec!["--lang=de".into()],
            env: {
                let mut m = HashMap::new();
                m.insert("LANG".into(), "de_DE".into());
                m
            },
        };
        let merged = BrowserConfig::merge(Some(&global), Some(&test)).unwrap();
        assert_eq!(merged.args, vec!["--no-sandbox", "--lang=de"]);
        assert_eq!(merged.env.get("LANG").unwrap(), "de_DE");
        assert_eq!(merged.version, Some("121".into()));
    }

    #[test]
    fn test_build_test_tasks_different_browser_fingerprints() {
        let global = GlobalConfig {
            base_url: "http://localhost:3000".into(),
            browser: None,
            full_screen: true,
            test_pattern: "tests/**/*.xsnap.jsonc".into(),
            ignore_patterns: vec![],
            default_sizes: Some(vec![Size {
                name: "desktop".into(),
                width: 1920,
                height: 1080,
            }]),
            functions: HashMap::new(),
            base_directory: "__snapshots__/__base_images__".into(),
            diff_directory: "__snapshots__/__diff__".into(),
            updated_directory: "__snapshots__/__updated__".into(),
            threshold: 0,
            threshold_percent: 0.5,
            retry: 1,
            parallelism: None,
            diff_pixel_color: crate::config::types::Color {
                r: 255,
                g: 0,
                b: 255,
            },
            http_headers: HashMap::new(),
            start_command: None,
            tests: vec![],
        };

        let tests = vec![
            TestConfig {
                name: "default-test".into(),
                url: "/".into(),
                threshold: None,
                threshold_percent: None,
                retry: None,
                only: false,
                skip: false,
                expected_response_code: None,
                sizes: None,
                browser: None,
                actions: None,
                ignore: None,
                http_headers: None,
            },
            TestConfig {
                name: "german-test".into(),
                url: "/de".into(),
                threshold: None,
                threshold_percent: None,
                retry: None,
                only: false,
                skip: false,
                expected_response_code: None,
                sizes: None,
                browser: Some(BrowserConfig {
                    version: None,
                    args: vec!["--lang=de".into()],
                    env: HashMap::new(),
                }),
                actions: None,
                ignore: None,
                http_headers: None,
            },
            TestConfig {
                name: "also-default".into(),
                url: "/about".into(),
                threshold: None,
                threshold_percent: None,
                retry: None,
                only: false,
                skip: false,
                expected_response_code: None,
                sizes: None,
                browser: None,
                actions: None,
                ignore: None,
                http_headers: None,
            },
        ];

        let (tasks, browser_configs) = build_test_tasks(&global, &tests);
        assert_eq!(tasks.len(), 3);
        // default-test and also-default share the same fingerprint
        assert_eq!(tasks[0].browser_fingerprint, tasks[2].browser_fingerprint);
        // german-test has a different fingerprint
        assert_ne!(tasks[0].browser_fingerprint, tasks[1].browser_fingerprint);
        // Two unique fingerprints → two pools needed
        assert_eq!(browser_configs.len(), 2);
    }
}
