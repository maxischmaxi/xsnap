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
    Pass,
    Created,
    Fail { score: f64, diff_path: String },
    Skipped,
    Error { message: String },
}

impl TestOutcome {
    pub fn is_pass(&self) -> bool {
        matches!(
            self,
            TestOutcome::Pass | TestOutcome::Created | TestOutcome::Skipped
        )
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
