use std::path::Path;
use std::sync::Arc;

use tokio::sync::mpsc;

use crate::browser::download::ensure_chromium;
use crate::browser::pool::BrowserPool;
use crate::config::global::load_global_config;
use crate::config::test::{discover_test_files, load_test_file};
use crate::config::types::TestConfig;
use crate::config::validate::validate_config;
use crate::runner::executor::{build_test_tasks, run_all, ProgressEvent};
use crate::ui::pipeline::{print_result, print_summary};
use crate::ui::tui::run_tui;

/// Options for the `xsnap test` command.
pub struct TestOptions {
    pub config: String,
    pub no_create: bool,
    pub no_only: bool,
    pub no_skip: bool,
    pub filter: Option<String>,
    pub pipeline: bool,
    pub parallelism: Option<usize>,
}

/// Run the test command.
///
/// Returns an exit code: 0 for success, 1 for test failures.
pub async fn run_test(opts: TestOptions) -> anyhow::Result<i32> {
    // 1. Load global config.
    let config_path = Path::new(&opts.config);
    let global = load_global_config(config_path)?;

    // 2. Discover and load test files.
    let base_dir = config_path
        .parent()
        .unwrap_or_else(|| Path::new("."));
    let test_files = discover_test_files(base_dir, &global.test_pattern, &global.ignore_patterns)?;

    let mut all_tests: Vec<TestConfig> = Vec::new();

    // Load tests from external files.
    for file in &test_files {
        let file_tests = load_test_file(file)?;
        all_tests.extend(file_tests);
    }

    // Include inline tests from the global config.
    all_tests.extend(global.tests.clone());

    // 3. Validate config.
    validate_config(&global, &all_tests)?;

    // 4. Apply flags: no_only, no_skip, filter.
    let tests = apply_flags(all_tests, opts.no_only, opts.no_skip, opts.filter.as_deref());

    // If no tests remain after filtering, exit early.
    if tests.is_empty() {
        println!("No tests found matching the specified criteria.");
        return Ok(0);
    }

    // 5. Build test tasks (test x size expansion).
    let tasks = build_test_tasks(&global, &tests);
    let total_tasks = tasks.len();

    if total_tasks == 0 {
        println!("No test tasks to run.");
        return Ok(0);
    }

    // 6. Set up browser.
    let browser_version = global
        .browser
        .as_ref()
        .and_then(|b| b.version.as_deref())
        .unwrap_or("auto");

    let chrome_path = ensure_chromium(browser_version).await?;

    let parallelism = opts
        .parallelism
        .or(global.parallelism)
        .unwrap_or_else(|| num_cpus::get().max(1));

    let pool = Arc::new(
        BrowserPool::new(
            &chrome_path,
            parallelism,
            global.browser.as_ref(),
        )
        .await?,
    );

    // 7. Run tests.
    let summary = if opts.pipeline {
        // Pipeline mode: print results as they arrive, no TUI.
        let (tx, mut rx) = mpsc::unbounded_channel::<ProgressEvent>();

        let is_github = std::env::var("GITHUB_ACTIONS").is_ok();

        // Spawn the runner.
        let pool_clone = Arc::clone(&pool);
        let runner_handle = tokio::spawn(async move {
            run_all(pool_clone, tasks, opts.no_create, Some(tx)).await
        });

        // Print results as they arrive.
        while let Some(event) = rx.recv().await {
            match event {
                ProgressEvent::TestStarted { .. } => {
                    // In pipeline mode, we don't print start events.
                }
                ProgressEvent::TestCompleted(result) => {
                    print_result(&result, is_github);
                }
                ProgressEvent::RunCompleted(ref summary) => {
                    print_summary(summary);
                }
            }
        }

        runner_handle.await?
    } else {
        // TUI mode.
        let (tx, rx) = mpsc::unbounded_channel::<ProgressEvent>();

        // Spawn the runner.
        let pool_clone = Arc::clone(&pool);
        let runner_handle = tokio::spawn(async move {
            run_all(pool_clone, tasks, opts.no_create, Some(tx)).await
        });

        // Run the TUI (blocks until user quits).
        let tui_result = run_tui(total_tasks, rx).await;

        // Wait for the runner to finish.
        let summary = runner_handle.await?;

        // If TUI had an error, print it but still use the runner summary.
        if let Err(e) = tui_result {
            eprintln!("TUI error: {}", e);
        }

        summary
    };

    // 8. Return exit code.
    if summary.failed > 0 || summary.errors > 0 {
        Ok(1)
    } else {
        Ok(0)
    }
}

/// Apply only/skip/filter flags to the list of tests.
///
/// - If any test has `only: true` and `no_only` is false, only those tests run.
/// - Tests with `skip: true` are excluded unless `no_skip` is true.
/// - If `filter` is set, only tests whose name contains the filter string are kept.
fn apply_flags(
    tests: Vec<TestConfig>,
    no_only: bool,
    no_skip: bool,
    filter: Option<&str>,
) -> Vec<TestConfig> {
    let mut result = tests;

    // Apply "only" filter: if any test has only=true and --no-only is not set,
    // keep only those tests.
    if !no_only {
        let has_only = result.iter().any(|t| t.only);
        if has_only {
            result.retain(|t| t.only);
        }
    }

    // Apply "skip" filter: remove skipped tests unless --no-skip is set.
    if !no_skip {
        result.retain(|t| !t.skip);
    }

    // Apply name filter.
    if let Some(pattern) = filter {
        result.retain(|t| t.name.contains(pattern));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test(name: &str, only: bool, skip: bool) -> TestConfig {
        TestConfig {
            name: name.to_string(),
            url: "/".to_string(),
            threshold: None,
            retry: None,
            only,
            skip,
            expected_response_code: None,
            sizes: None,
            browser: None,
            actions: None,
            ignore: None,
            http_headers: None,
        }
    }

    #[test]
    fn test_apply_flags_no_filters() {
        let tests = vec![
            make_test("a", false, false),
            make_test("b", false, false),
        ];
        let result = apply_flags(tests, false, false, None);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_apply_flags_only() {
        let tests = vec![
            make_test("a", true, false),
            make_test("b", false, false),
        ];
        let result = apply_flags(tests, false, false, None);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "a");
    }

    #[test]
    fn test_apply_flags_no_only_disables_only() {
        let tests = vec![
            make_test("a", true, false),
            make_test("b", false, false),
        ];
        let result = apply_flags(tests, true, false, None);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_apply_flags_skip() {
        let tests = vec![
            make_test("a", false, true),
            make_test("b", false, false),
        ];
        let result = apply_flags(tests, false, false, None);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "b");
    }

    #[test]
    fn test_apply_flags_no_skip_disables_skip() {
        let tests = vec![
            make_test("a", false, true),
            make_test("b", false, false),
        ];
        let result = apply_flags(tests, false, true, None);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_apply_flags_filter() {
        let tests = vec![
            make_test("homepage", false, false),
            make_test("login-page", false, false),
            make_test("dashboard", false, false),
        ];
        let result = apply_flags(tests, false, false, Some("page"));
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "homepage");
        assert_eq!(result[1].name, "login-page");
    }

    #[test]
    fn test_apply_flags_combined() {
        let tests = vec![
            make_test("homepage", true, false),
            make_test("login-page", true, false),
            make_test("dashboard", false, true),
            make_test("settings", false, false),
        ];
        // only=true filters to homepage and login-page;
        // filter="home" further narrows to homepage only.
        let result = apply_flags(tests, false, false, Some("home"));
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "homepage");
    }
}
