use std::time::Duration;
use xsnap::runner::result::{RunSummary, TestOutcome, TestResult};
use xsnap::ui::pipeline::{format_result_line, format_summary, github_annotation};

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
    assert!(line.contains("desktop"));
    assert!(line.contains("1920x1080"));
}

#[test]
fn test_format_fail() {
    let result = make_result(
        "homepage",
        TestOutcome::Fail {
            score: 0.85,
            diff_path: "diff.png".into(),
        },
    );
    let line = format_result_line(&result);
    assert!(line.contains("FAIL"));
}

#[test]
fn test_format_created() {
    let result = make_result("new-page", TestOutcome::Created);
    let line = format_result_line(&result);
    assert!(line.contains("NEW"));
}

#[test]
fn test_format_with_retries() {
    let mut result = make_result("flaky", TestOutcome::Pass);
    result.retries_used = 2;
    let line = format_result_line(&result);
    assert!(line.contains("retried 2x"));
}

#[test]
fn test_github_annotation_fail() {
    let result = make_result(
        "homepage",
        TestOutcome::Fail {
            score: 0.85,
            diff_path: "diff.png".into(),
        },
    );
    let annotation = github_annotation(&result);
    assert!(annotation.starts_with("::error::"));
    assert!(annotation.contains("homepage"));
}

#[test]
fn test_github_annotation_pass_is_empty() {
    let result = make_result("homepage", TestOutcome::Pass);
    let annotation = github_annotation(&result);
    assert!(annotation.is_empty());
}

#[test]
fn test_format_summary() {
    let summary = RunSummary {
        total: 10,
        passed: 8,
        failed: 1,
        created: 0,
        skipped: 1,
        errors: 0,
        duration: Duration::from_secs(5),
    };
    let text = format_summary(&summary);
    assert!(text.contains("10 tests"));
    assert!(text.contains("8 passed"));
    assert!(text.contains("1 failed"));
}
