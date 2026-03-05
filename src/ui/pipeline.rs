use crate::runner::result::{RunSummary, TestOutcome, TestResult};

pub fn format_result_line(result: &TestResult) -> String {
    let status = match &result.outcome {
        TestOutcome::Pass => "PASS",
        TestOutcome::Created => "NEW ",
        TestOutcome::Fail { .. } => "FAIL",
        TestOutcome::Skipped => "SKIP",
        TestOutcome::Error { .. } => "ERR ",
    };

    let retries = if result.retries_used > 0 {
        format!(" (retried {}x)", result.retries_used)
    } else {
        String::new()
    };

    let mut line = format!(
        "[{}] {}-{}-{}x{} ({}ms){}",
        status,
        result.test_name,
        result.size_name,
        result.width,
        result.height,
        result.duration.as_millis(),
        retries,
    );

    if let TestOutcome::Error { message } = &result.outcome {
        line.push_str(&format!("\n       → {}", message));
    }

    line
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
