use std::io;
use std::time::Duration;

use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind, KeyModifiers};
use futures::StreamExt;
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table, TableState, Wrap},
};
use tokio::sync::mpsc;

use crate::runner::executor::ProgressEvent;
use crate::runner::result::{RunSummary, TestOutcome, TestResult};

// ---------------------------------------------------------------------------
// TuiState
// ---------------------------------------------------------------------------

/// Internal state for the TUI.
struct TuiState {
    total_tasks: usize,
    completed: usize,
    results: Vec<TestResult>,
    table_state: TableState,
    summary: Option<RunSummary>,
    running: Vec<String>,
    logs: Vec<String>,
    start_command: Option<String>,
    server_status: Option<(u32, u32)>,
    server_ready: bool,
}

impl TuiState {
    fn new(total_tasks: usize, start_command: Option<String>) -> Self {
        Self {
            total_tasks,
            completed: 0,
            results: Vec::new(),
            table_state: TableState::default(),
            summary: None,
            running: Vec::new(),
            logs: Vec::new(),
            start_command,
            server_status: None,
            server_ready: false,
        }
    }

    fn handle_event(&mut self, event: ProgressEvent) {
        match event {
            ProgressEvent::TestStarted { name, size } => {
                self.running.push(format!("{} ({})", name, size));
            }
            ProgressEvent::TestCompleted(result) => {
                // Remove from running list.
                let label = format!(
                    "{} ({}-{}x{})",
                    result.test_name, result.size_name, result.width, result.height
                );
                self.running.retain(|r| *r != label);
                self.completed += 1;
                self.results.push(result);
            }
            ProgressEvent::RunCompleted(summary) => {
                self.summary = Some(summary);
            }
            ProgressEvent::ServerWaiting {
                attempt,
                max_attempts,
            } => {
                self.server_status = Some((attempt, max_attempts));
            }
            ProgressEvent::ServerReady => {
                self.server_ready = true;
            }
        }
    }

    fn scroll_up(&mut self) {
        let selected = self.table_state.selected().unwrap_or(0);
        if selected > 0 {
            self.table_state.select(Some(selected - 1));
        }
    }

    fn scroll_down(&mut self) {
        let max = if self.results.is_empty() {
            0
        } else {
            self.results.len() - 1
        };
        let selected = self.table_state.selected().unwrap_or(0);
        if selected < max {
            self.table_state.select(Some(selected + 1));
        }
    }

    fn progress_ratio(&self) -> f64 {
        if self.total_tasks == 0 {
            return 1.0;
        }
        self.completed as f64 / self.total_tasks as f64
    }

    fn has_logs(&self) -> bool {
        self.start_command.is_some()
    }
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

fn render(frame: &mut Frame, state: &mut TuiState) {
    let area = frame.area();

    if state.has_logs() {
        // Horizontal split: tests (60%) | logs (40%)
        let [left, right] =
            Layout::horizontal([Constraint::Percentage(60), Constraint::Percentage(40)])
                .areas(area);

        render_tests(frame, state, left);
        render_logs(frame, state, right);
    } else {
        render_tests(frame, state, area);
    }
}

fn render_tests(frame: &mut Frame, state: &mut TuiState, area: Rect) {
    // Layout: [Gauge 3 rows] [Table (fill)] [Summary 3 rows]
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(5),
        Constraint::Length(3),
    ])
    .split(area);

    // -- Progress gauge --
    if let Some((attempt, max)) = state.server_status {
        if !state.server_ready {
            let gauge = Gauge::default()
                .block(Block::default().borders(Borders::ALL).title(" Server "))
                .gauge_style(Style::default().fg(Color::Yellow).bg(Color::DarkGray))
                .ratio(attempt as f64 / max as f64)
                .label(format!("Waiting for server... ({}/{})", attempt, max));
            frame.render_widget(gauge, chunks[0]);
        } else {
            let progress_label = format!("{}/{}", state.completed, state.total_tasks);
            let gauge = Gauge::default()
                .block(Block::default().borders(Borders::ALL).title(" Progress "))
                .gauge_style(Style::default().fg(Color::Cyan).bg(Color::DarkGray))
                .ratio(state.progress_ratio())
                .label(progress_label);
            frame.render_widget(gauge, chunks[0]);
        }
    } else {
        let progress_label = format!("{}/{}", state.completed, state.total_tasks);
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title(" Progress "))
            .gauge_style(Style::default().fg(Color::Cyan).bg(Color::DarkGray))
            .ratio(state.progress_ratio())
            .label(progress_label);
        frame.render_widget(gauge, chunks[0]);
    }

    // -- Results table --
    let header = Row::new(vec![
        Cell::from("Status"),
        Cell::from("Test"),
        Cell::from("Size"),
        Cell::from("Duration"),
        Cell::from("Retries"),
    ])
    .style(
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(Color::Yellow),
    );

    let rows: Vec<Row> = state
        .results
        .iter()
        .map(|r| {
            let (status_text, style) = match &r.outcome {
                TestOutcome::Pass => ("PASS", Style::default().fg(Color::Green)),
                TestOutcome::Created => ("NEW ", Style::default().fg(Color::Cyan)),
                TestOutcome::Fail { .. } => ("FAIL", Style::default().fg(Color::Red)),
                TestOutcome::Skipped => ("SKIP", Style::default().fg(Color::Yellow)),
                TestOutcome::Error { .. } => (
                    "ERR ",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
            };

            let size_str = format!("{}-{}x{}", r.size_name, r.width, r.height);
            let duration_str = format!("{}ms", r.duration.as_millis());
            let retry_str = if r.retries_used > 0 {
                format!("{}x", r.retries_used)
            } else {
                "-".to_string()
            };

            Row::new(vec![
                Cell::from(status_text).style(style),
                Cell::from(r.test_name.as_str()),
                Cell::from(size_str),
                Cell::from(duration_str),
                Cell::from(retry_str),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(6),
            Constraint::Percentage(35),
            Constraint::Percentage(25),
            Constraint::Length(12),
            Constraint::Length(8),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Results "))
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_stateful_widget(table, chunks[1], &mut state.table_state);

    // -- Summary bar --
    let summary_text = if let Some(ref s) = state.summary {
        Line::from(vec![
            Span::raw("Total: "),
            Span::styled(format!("{}", s.total), Style::default().bold()),
            Span::raw("  "),
            Span::styled(
                format!("{} passed", s.passed),
                Style::default().fg(Color::Green),
            ),
            Span::raw("  "),
            Span::styled(
                format!("{} failed", s.failed),
                Style::default().fg(Color::Red),
            ),
            Span::raw("  "),
            Span::styled(
                format!("{} created", s.created),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw("  "),
            Span::styled(
                format!("{} skipped", s.skipped),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw("  "),
            Span::styled(
                format!("{} errors", s.errors),
                Style::default().fg(Color::Red),
            ),
            Span::raw(format!("  ({:.1}s)", s.duration.as_secs_f64())),
            Span::raw("  |  Press q to quit"),
        ])
    } else {
        let running_text = if state.running.is_empty() {
            "Waiting...".to_string()
        } else {
            let display: Vec<&str> = state.running.iter().take(3).map(|s| s.as_str()).collect();
            let suffix = if state.running.len() > 3 {
                format!(" +{} more", state.running.len() - 3)
            } else {
                String::new()
            };
            format!("Running: {}{}", display.join(", "), suffix)
        };
        Line::from(vec![
            Span::raw(running_text),
            Span::raw("  |  Press q to quit"),
        ])
    };

    let summary_bar = Paragraph::new(summary_text)
        .block(Block::default().borders(Borders::ALL).title(" Summary "));

    frame.render_widget(summary_bar, chunks[2]);
}

fn render_logs(frame: &mut Frame, state: &TuiState, area: Rect) {
    let title = format!(" $ {} ", state.start_command.as_deref().unwrap_or(""));

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Show the last N lines that fit in the panel.
    let visible_lines = inner.height as usize;
    let start = state.logs.len().saturating_sub(visible_lines);
    let visible = &state.logs[start..];

    let text: Vec<Line> = visible
        .iter()
        .map(|l| Line::from(Span::styled(l.as_str(), Style::default().fg(Color::Gray))))
        .collect();

    let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, inner);
}

// ---------------------------------------------------------------------------
// run_tui
// ---------------------------------------------------------------------------

/// Run the TUI, receiving progress updates from the test runner.
///
/// The TUI displays a progress gauge, a scrollable results table, and a
/// summary bar. When a `start_command` is active, a log panel is shown on the
/// right side. It handles keyboard input for scrolling (j/k, up/down) and
/// quitting (q, Esc, Ctrl+C).
///
/// Returns the final `RunSummary` once all tests complete and the user exits.
pub async fn run_tui(
    total_tasks: usize,
    rx: mpsc::UnboundedReceiver<ProgressEvent>,
    log_rx: Option<mpsc::UnboundedReceiver<String>>,
    start_command: Option<String>,
) -> io::Result<RunSummary> {
    let mut terminal = ratatui::init();
    let result = run_tui_inner(&mut terminal, total_tasks, rx, log_rx, start_command).await;
    ratatui::restore();
    result
}

async fn run_tui_inner(
    terminal: &mut DefaultTerminal,
    total_tasks: usize,
    mut rx: mpsc::UnboundedReceiver<ProgressEvent>,
    mut log_rx: Option<mpsc::UnboundedReceiver<String>>,
    start_command: Option<String>,
) -> io::Result<RunSummary> {
    let mut state = TuiState::new(total_tasks, start_command);
    let mut event_stream = EventStream::new();

    // Initial render.
    terminal.draw(|frame| render(frame, &mut state))?;

    loop {
        tokio::select! {
            // Handle progress events from the runner.
            Some(progress) = rx.recv() => {
                let is_run_completed = matches!(&progress, ProgressEvent::RunCompleted(_));
                state.handle_event(progress);
                terminal.draw(|frame| render(frame, &mut state))?;

                // Auto-scroll to latest result if no manual selection.
                if !state.results.is_empty() && state.table_state.selected().is_none() {
                    state.table_state.select(Some(state.results.len() - 1));
                }

                // If all tests are done and we have a summary, wait for user to
                // press q before exiting. Don't auto-exit.
                if is_run_completed {
                    // Continue the loop to handle keyboard events.
                }
            }

            // Handle log lines from the child process.
            line = async {
                match &mut log_rx {
                    Some(rx) => rx.recv().await,
                    None => futures::future::pending().await,
                }
            } => {
                if let Some(line) = line {
                    state.logs.push(line);
                    terminal.draw(|frame| render(frame, &mut state))?;
                }
            }

            // Handle keyboard events.
            Some(Ok(event)) = event_stream.next() => {
                if let Event::Key(key) = event
                    && key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => {
                                return Ok(make_summary(&state, total_tasks));
                            }
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                return Ok(make_summary(&state, total_tasks));
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                state.scroll_up();
                                terminal.draw(|frame| render(frame, &mut state))?;
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                state.scroll_down();
                                terminal.draw(|frame| render(frame, &mut state))?;
                            }
                            _ => {}
                        }
                    }
            }

            // If both channels are closed, break.
            else => {
                break;
            }
        }
    }

    Ok(make_summary(&state, total_tasks))
}

fn make_summary(state: &TuiState, total_tasks: usize) -> RunSummary {
    state.summary.clone().unwrap_or(RunSummary {
        total: total_tasks,
        passed: 0,
        failed: 0,
        created: 0,
        skipped: 0,
        errors: total_tasks,
        duration: Duration::ZERO,
    })
}
