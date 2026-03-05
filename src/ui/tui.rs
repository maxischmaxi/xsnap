use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Gauge, List, ListItem, Paragraph, Row, Table, TableState, Wrap,
    },
};
use tokio::sync::mpsc;

use crate::runner::executor::ProgressEvent;
use crate::runner::result::{RunSummary, TestOutcome, TestResult};

// ---------------------------------------------------------------------------
// Spinner
// ---------------------------------------------------------------------------

const SPINNER_CHARS: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

// ---------------------------------------------------------------------------
// TuiState
// ---------------------------------------------------------------------------

/// Internal state for the TUI.
struct TuiState {
    total_tasks: usize,
    completed: usize,
    results: Vec<TestResult>,
    table_state: TableState,
    auto_scroll: bool,
    summary: Option<RunSummary>,
    running: Vec<String>,
    logs: Vec<String>,
    start_command: Option<String>,
    server_ready: bool,
    waiting_url: String,
    waiting_elapsed: u32,
    completed_area_height: u16,
}

impl TuiState {
    fn new(total_tasks: usize, start_command: Option<String>, base_url: String) -> Self {
        Self {
            total_tasks,
            completed: 0,
            results: Vec::new(),
            table_state: TableState::default(),
            auto_scroll: true,
            summary: None,
            running: Vec::new(),
            logs: Vec::new(),
            start_command,
            server_ready: false,
            waiting_url: base_url,
            waiting_elapsed: 0,
            completed_area_height: 0,
        }
    }

    fn handle_event(&mut self, event: ProgressEvent) {
        match event {
            ProgressEvent::ServerWaiting { url, elapsed_secs } => {
                self.waiting_url = url;
                self.waiting_elapsed = elapsed_secs;
            }
            ProgressEvent::ServerReady => {
                self.server_ready = true;
            }
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

                // Auto-scroll to latest result.
                if self.auto_scroll {
                    self.table_state.select(Some(self.results.len() - 1));
                }
            }
            ProgressEvent::RunCompleted(summary) => {
                self.summary = Some(summary);
            }
        }
    }

    fn scroll_up(&mut self) {
        let selected = self.table_state.selected().unwrap_or(0);
        if selected > 0 {
            self.table_state.select(Some(selected - 1));
            self.auto_scroll = false;
        }
    }

    fn scroll_down(&mut self) {
        if self.results.is_empty() {
            return;
        }
        let max = self.results.len() - 1;
        let selected = self.table_state.selected().unwrap_or(0);
        if selected < max {
            self.table_state.select(Some(selected + 1));
            // Re-enable auto-scroll when user reaches the bottom.
            if selected + 1 == max {
                self.auto_scroll = true;
            }
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
    if state.server_ready {
        render_testing(frame, state);
    } else {
        render_waiting(frame, state);
    }
}

// ---------------------------------------------------------------------------
// Waiting phase
// ---------------------------------------------------------------------------

fn render_waiting(frame: &mut Frame, state: &mut TuiState) {
    let area = frame.area();

    // Layout: [Waiting 3] [Logs (fill) | or empty fill] [Summary 3]
    let mut constraints = vec![Constraint::Length(3)]; // waiting status
    if state.has_logs() {
        constraints.push(Constraint::Min(5)); // logs
    } else {
        constraints.push(Constraint::Min(1)); // empty fill
    }
    constraints.push(Constraint::Length(3)); // summary

    let chunks = Layout::vertical(constraints).split(area);

    let waiting_area = chunks[0];
    let middle_area = chunks[1];
    let summary_area = chunks[2];

    // Waiting status bar with spinner.
    let spinner = SPINNER_CHARS[(state.waiting_elapsed as usize) % SPINNER_CHARS.len()];
    let elapsed_text = if state.waiting_elapsed > 0 {
        format!(" ({}s)", state.waiting_elapsed)
    } else {
        String::new()
    };

    let waiting_text = Line::from(vec![
        Span::styled(format!("{} ", spinner), Style::default().fg(Color::Yellow)),
        Span::raw(format!(
            "Waiting for {}...{}",
            state.waiting_url, elapsed_text
        )),
    ]);

    let waiting_widget = Paragraph::new(waiting_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Waiting ")
            .border_style(Style::default().fg(Color::Yellow)),
    );
    frame.render_widget(waiting_widget, waiting_area);

    // Middle: logs panel or empty block.
    if state.has_logs() {
        render_logs(frame, state, middle_area);
    } else {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));
        frame.render_widget(block, middle_area);
    }

    // Summary bar.
    let summary_text = Line::from(vec![
        Span::styled("Waiting for server", Style::default().fg(Color::Yellow)),
        Span::raw("  |  Press q to quit"),
    ]);
    let summary_bar = Paragraph::new(summary_text)
        .block(Block::default().borders(Borders::ALL).title(" Summary "));
    frame.render_widget(summary_bar, summary_area);
}

// ---------------------------------------------------------------------------
// Testing phase
// ---------------------------------------------------------------------------

fn render_testing(frame: &mut Frame, state: &mut TuiState) {
    let area = frame.area();

    // Vertical: [Gauge 3] [Lists (fill)] [Logs if applicable] [Summary 3]
    let mut vertical_constraints = vec![
        Constraint::Length(3), // gauge
        Constraint::Min(5),    // running + completed lists
    ];
    if state.has_logs() {
        vertical_constraints.push(Constraint::Length(10)); // logs
    }
    vertical_constraints.push(Constraint::Length(3)); // summary

    let main_chunks = Layout::vertical(vertical_constraints).split(area);

    let gauge_area = main_chunks[0];
    let lists_area = main_chunks[1];
    let (logs_area, summary_area) = if state.has_logs() {
        (Some(main_chunks[2]), main_chunks[3])
    } else {
        (None, main_chunks[2])
    };

    render_gauge(frame, state, gauge_area);
    render_lists(frame, state, lists_area);
    if let Some(logs_area) = logs_area {
        render_logs(frame, state, logs_area);
    }
    render_summary(frame, state, summary_area);
}

fn render_gauge(frame: &mut Frame, state: &TuiState, area: Rect) {
    let progress_label = format!("{}/{}", state.completed, state.total_tasks);
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" Progress "))
        .gauge_style(Style::default().fg(Color::Cyan).bg(Color::DarkGray))
        .ratio(state.progress_ratio())
        .label(progress_label);
    frame.render_widget(gauge, area);
}

fn render_lists(frame: &mut Frame, state: &mut TuiState, area: Rect) {
    // Horizontal split: Running (30%) | Completed (70%)
    let [left, right] =
        Layout::horizontal([Constraint::Percentage(30), Constraint::Percentage(70)]).areas(area);

    render_running(frame, state, left);
    render_completed(frame, state, right);
}

fn render_running(frame: &mut Frame, state: &TuiState, area: Rect) {
    let title = format!(" Running ({}) ", state.running.len());

    let items: Vec<ListItem> = state
        .running
        .iter()
        .map(|r| {
            ListItem::new(Line::from(vec![
                Span::styled("● ", Style::default().fg(Color::Yellow)),
                Span::raw(r.as_str()),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(Color::Yellow)),
    );

    frame.render_widget(list, area);
}

fn render_completed(frame: &mut Frame, state: &mut TuiState, area: Rect) {
    state.completed_area_height = area.height;

    let title = format!(" Completed ({}) ", state.results.len());

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

    let mut rows: Vec<Row> = Vec::new();
    let mut visual_index_map: Vec<usize> = Vec::new();

    for r in state.results.iter() {
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

        visual_index_map.push(rows.len());

        rows.push(Row::new(vec![
            Cell::from(status_text).style(style),
            Cell::from(r.test_name.as_str()),
            Cell::from(size_str),
            Cell::from(duration_str),
            Cell::from(retry_str),
        ]));

        if let TestOutcome::Error { message } = &r.outcome {
            rows.push(Row::new(vec![
                Cell::from(""),
                Cell::from(format!("\u{2192} {}", message))
                    .style(Style::default().fg(Color::DarkGray)),
                Cell::from(""),
                Cell::from(""),
                Cell::from(""),
            ]));
        }
    }

    let visual_selected = state
        .table_state
        .selected()
        .and_then(|logical| visual_index_map.get(logical).copied());
    let mut visual_table_state = TableState::default();
    visual_table_state.select(visual_selected);

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
    .block(Block::default().borders(Borders::ALL).title(title))
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_stateful_widget(table, area, &mut visual_table_state);
}

fn render_summary(frame: &mut Frame, state: &TuiState, area: Rect) {
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
        Line::from(vec![
            Span::raw(format!("{}/{}", state.completed, state.total_tasks)),
            Span::raw("  |  Press q to quit"),
        ])
    };

    let summary_bar = Paragraph::new(summary_text)
        .block(Block::default().borders(Borders::ALL).title(" Summary "));

    frame.render_widget(summary_bar, area);
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
/// The TUI starts immediately in a "waiting for server" phase, showing a
/// spinner and the log panel. Once `ServerReady` is received, it transitions
/// to the normal test progress view.
pub async fn run_tui(
    total_tasks: usize,
    rx: mpsc::UnboundedReceiver<ProgressEvent>,
    log_rx: Option<mpsc::UnboundedReceiver<String>>,
    start_command: Option<String>,
    base_url: String,
) -> io::Result<RunSummary> {
    let mut terminal = ratatui::init();
    let result = run_tui_inner(
        &mut terminal,
        total_tasks,
        rx,
        log_rx,
        start_command,
        base_url,
    )
    .await;
    ratatui::restore();
    result
}

async fn run_tui_inner(
    terminal: &mut DefaultTerminal,
    total_tasks: usize,
    mut rx: mpsc::UnboundedReceiver<ProgressEvent>,
    mut log_rx: Option<mpsc::UnboundedReceiver<String>>,
    start_command: Option<String>,
    base_url: String,
) -> io::Result<RunSummary> {
    let mut state = TuiState::new(total_tasks, start_command, base_url);

    // Blocking thread for keyboard events — crossterm's EventStream doesn't
    // work reliably with ratatui's keyboard-enhancement protocol, so we use
    // the synchronous event::read() in a dedicated thread instead.
    let (key_tx, mut key_rx) = mpsc::unbounded_channel();
    std::thread::spawn(move || {
        while let Ok(ev) = event::read() {
            if key_tx.send(ev).is_err() {
                break; // Receiver dropped, TUI beendet
            }
        }
    });

    // Initial render.
    terminal.draw(|frame| render(frame, &mut state))?;

    let mut rx_closed = false;

    loop {
        tokio::select! {
            // Bias toward keyboard events so q/Esc always work immediately.
            biased;

            // Handle keyboard and terminal events (highest priority).
            Some(event) = key_rx.recv() => {
                match event {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
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
                    Event::Resize(_, _) => {
                        terminal.draw(|frame| render(frame, &mut state))?;
                    }
                    _ => {}
                }
            }

            // Handle progress events from the runner.
            result = rx.recv(), if !rx_closed => {
                match result {
                    Some(progress) => {
                        state.handle_event(progress);
                        terminal.draw(|frame| render(frame, &mut state))?;
                    }
                    None => {
                        rx_closed = true;
                    }
                }
            }

            // Handle log lines from the child process.
            result = async {
                match &mut log_rx {
                    Some(rx) => rx.recv().await,
                    None => futures::future::pending().await,
                }
            } => {
                match result {
                    Some(line) => {
                        state.logs.push(line);
                        terminal.draw(|frame| render(frame, &mut state))?;
                    }
                    None => {
                        // Channel closed, stop polling.
                        log_rx = None;
                    }
                }
            }

            // All data channels closed and event stream ended.
            else => {
                return Ok(make_summary(&state, total_tasks));
            }
        }
    }
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
