use std::collections::HashMap;
use std::io;
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{Bar, BarChart, BarGroup, Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::engram::{Engram, EnrichedEvent};

#[derive(PartialEq, Clone, Copy)]
enum Panel {
    Memories,
    Events,
}

#[derive(PartialEq, Clone, Copy)]
enum ChartMode {
    Both,
    AddOnly,
    TapOnly,
}

impl ChartMode {
    fn next(self) -> Self {
        match self {
            ChartMode::Both => ChartMode::AddOnly,
            ChartMode::AddOnly => ChartMode::TapOnly,
            ChartMode::TapOnly => ChartMode::Both,
        }
    }

    fn label(self) -> &'static str {
        match self {
            ChartMode::Both => "All",
            ChartMode::AddOnly => "ADD",
            ChartMode::TapOnly => "TAP",
        }
    }
}

/// Holds the full content for expansion
struct ExpandedContent {
    title: String,
    content: String,
}

struct AppState {
    focused: Panel,
    memories_state: ListState,
    events_state: ListState,
    memories_count: usize,
    events_count: usize,
    chart_mode: ChartMode,
    expanded: Option<ExpandedContent>,
}

impl AppState {
    fn new() -> Self {
        let mut memories_state = ListState::default();
        memories_state.select(Some(0));
        let mut events_state = ListState::default();
        events_state.select(Some(0));
        Self {
            focused: Panel::Memories,
            memories_state,
            events_state,
            memories_count: 0,
            events_count: 0,
            chart_mode: ChartMode::Both,
            expanded: None,
        }
    }

    fn move_up(&mut self) {
        let (state, count) = match self.focused {
            Panel::Memories => (&mut self.memories_state, self.memories_count),
            Panel::Events => (&mut self.events_state, self.events_count),
        };
        if count == 0 {
            return;
        }
        let i = state.selected().unwrap_or(0);
        let new_i = if i == 0 { count - 1 } else { i - 1 };
        state.select(Some(new_i));
    }

    fn move_down(&mut self) {
        let (state, count) = match self.focused {
            Panel::Memories => (&mut self.memories_state, self.memories_count),
            Panel::Events => (&mut self.events_state, self.events_count),
        };
        if count == 0 {
            return;
        }
        let i = state.selected().unwrap_or(0);
        let new_i = if i >= count - 1 { 0 } else { i + 1 };
        state.select(Some(new_i));
    }

    fn toggle_panel(&mut self) {
        self.focused = match self.focused {
            Panel::Memories => Panel::Events,
            Panel::Events => Panel::Memories,
        };
    }
}

pub fn run() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    // Main loop
    let result = run_loop(&mut terminal);

    // Restore terminal
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    result
}

fn run_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let mut state = AppState::new();

    let engram = match Engram::from_env() {
        Ok(e) => e,
        Err(_) => return Ok(()), // Exit gracefully if DB can't be opened
    };

    loop {
        // Fetch data outside of draw closure so we can use it for expansion
        let memories = engram.list_memories_filtered(false).unwrap_or_default();
        let events = engram.get_enriched_events(100, None, None, false).unwrap_or_default();

        // Update counts
        state.memories_count = memories.len();
        state.events_count = events.len();

        // Compute activity for chart
        let activity = compute_hourly_activity(&events);

        terminal.draw(|frame| {
            let area = frame.area();

            // Split screen: left (memories+events) and right (chart)
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(70),
                    Constraint::Percentage(30),
                ])
                .split(area);

            // Split left side: memories on top, events on bottom
            let left_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(40),
                    Constraint::Percentage(60),
                ])
                .split(main_chunks[0]);

            // Build list items
            let memories_items: Vec<ListItem> = memories
                .iter()
                .map(|m| {
                    let short_id = if m.id.len() > 8 { &m.id[..8] } else { &m.id };
                    let content = truncate(&m.content, 60);
                    ListItem::new(format!("[{}] taps:{:2} | {}", short_id, m.tap_count, content))
                })
                .collect();

            let events_items: Vec<ListItem> = events
                .iter()
                .take(50)
                .map(|e| {
                    let time = format_timestamp(&e.timestamp);
                    let mem_id = e.memory_id.as_deref().unwrap_or("-");
                    let short_id = if mem_id.len() > 8 { &mem_id[..8] } else { mem_id };

                    // For TAP events, don't show content in list (reduces duplication)
                    // Content is shown when pressing Enter to expand
                    let data_preview = if e.action == "TAP" {
                        String::new()
                    } else {
                        truncate(&e.content, 40)
                    };

                    let text = format!("{} {:6} {} {}", time, e.action, short_id, data_preview);

                    // Color code by memory_id
                    let style = if mem_id != "-" {
                        Style::default().fg(color_for_memory_id(mem_id))
                    } else {
                        Style::default()
                    };
                    ListItem::new(text).style(style)
                })
                .collect();

            // Render memories panel
            let memories_title = if state.focused == Panel::Memories {
                " Memories [*] "
            } else {
                " Memories "
            };
            let memories_block = Block::default()
                .title(memories_title)
                .borders(Borders::ALL)
                .border_style(if state.focused == Panel::Memories {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                });
            let memories_list = List::new(memories_items)
                .block(memories_block)
                .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
                .highlight_symbol("> ");
            frame.render_stateful_widget(memories_list, left_chunks[0], &mut state.memories_state);

            // Render events panel
            let events_title = if state.focused == Panel::Events {
                " Events [*] (q:quit Tab:switch j/k:nav Enter:expand) "
            } else {
                " Events (q:quit Tab:switch j/k:nav Enter:expand) "
            };
            let events_block = Block::default()
                .title(events_title)
                .borders(Borders::ALL)
                .border_style(if state.focused == Panel::Events {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                });
            let events_list = List::new(events_items)
                .block(events_block)
                .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
                .highlight_symbol("> ");
            frame.render_stateful_widget(events_list, left_chunks[1], &mut state.events_state);

            // Render activity bar chart based on mode
            let bar_groups: Vec<BarGroup> = activity
                .iter()
                .map(|(hour, adds, taps)| {
                    let bars: Vec<Bar> = match state.chart_mode {
                        ChartMode::Both => vec![
                            Bar::default()
                                .value(*adds)
                                .style(Style::default().fg(Color::Green)),
                            Bar::default()
                                .value(*taps)
                                .style(Style::default().fg(Color::Cyan)),
                        ],
                        ChartMode::AddOnly => vec![
                            Bar::default()
                                .value(*adds)
                                .style(Style::default().fg(Color::Green)),
                        ],
                        ChartMode::TapOnly => vec![
                            Bar::default()
                                .value(*taps)
                                .style(Style::default().fg(Color::Cyan)),
                        ],
                    };
                    BarGroup::default()
                        .label(hour.clone().into())
                        .bars(&bars)
                })
                .collect();

            let chart_title = format!(
                " Last 24h [{}] (c:cycle) ",
                state.chart_mode.label()
            );
            let chart = BarChart::default()
                .block(
                    Block::default()
                        .title(chart_title)
                        .borders(Borders::ALL),
                )
                .bar_width(3)
                .bar_gap(1)
                .group_gap(1)
                .data(BarGroup::default().bars(&[])) // placeholder
                .max(10);

            // Render chart with groups
            let chart = bar_groups.iter().fold(chart, |c, g| c.data(g.clone()));
            frame.render_widget(chart, main_chunks[1]);

            // Render expansion popup if active
            if let Some(ref expanded) = state.expanded {
                let popup_area = centered_rect(80, 60, area);
                frame.render_widget(Clear, popup_area);
                let popup = Paragraph::new(expanded.content.as_str())
                    .block(
                        Block::default()
                            .title(format!(" {} (Esc to close) ", expanded.title))
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Cyan)),
                    )
                    .wrap(Wrap { trim: false });
                frame.render_widget(popup, popup_area);
            }
        })?;

        // Handle input (with timeout for refresh)
        if event::poll(Duration::from_secs(1))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    // If popup is open, only handle Esc
                    if state.expanded.is_some() {
                        if key.code == KeyCode::Esc {
                            state.expanded = None;
                        }
                        continue;
                    }

                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('j') | KeyCode::Down => state.move_down(),
                        KeyCode::Char('k') | KeyCode::Up => state.move_up(),
                        KeyCode::Char('c') => state.chart_mode = state.chart_mode.next(),
                        KeyCode::Tab => state.toggle_panel(),
                        KeyCode::Enter => {
                            // Expand selected item
                            match state.focused {
                                Panel::Memories => {
                                    if let Some(idx) = state.memories_state.selected() {
                                        if let Some(m) = memories.get(idx) {
                                            state.expanded = Some(ExpandedContent {
                                                title: format!("Memory {}", &m.id[..8.min(m.id.len())]),
                                                content: format!(
                                                    "ID: {}\nTaps: {}\nCreated: {}\n\n{}",
                                                    m.id,
                                                    m.tap_count,
                                                    format_timestamp(&m.created_at),
                                                    m.content
                                                ),
                                            });
                                        }
                                    }
                                }
                                Panel::Events => {
                                    if let Some(idx) = state.events_state.selected() {
                                        if let Some(e) = events.get(idx) {
                                            let mem_id = e.memory_id.as_deref().unwrap_or("-");
                                            state.expanded = Some(ExpandedContent {
                                                title: format!("{} Event", e.action),
                                                content: format!(
                                                    "Time: {}\nAction: {}\nMemory: {}\n\nData:\n{}",
                                                    format_timestamp(&e.timestamp),
                                                    e.action,
                                                    mem_id,
                                                    e.content
                                                ),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}

/// Create a centered rectangle for popups
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn format_timestamp(ts: &str) -> String {
    // Parse RFC3339 datetime and extract time portion
    chrono::DateTime::parse_from_rfc3339(ts)
        .map(|dt| dt.format("%H:%M:%S").to_string())
        .unwrap_or_else(|_| "Invalid".to_string())
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Get a consistent color for a memory ID
fn color_for_memory_id(id: &str) -> Color {
    const COLORS: [Color; 6] = [
        Color::Red,
        Color::Green,
        Color::Yellow,
        Color::Blue,
        Color::Magenta,
        Color::Cyan,
    ];

    // Simple hash based on first few chars
    let hash: usize = id.bytes().take(8).map(|b| b as usize).sum();
    COLORS[hash % COLORS.len()]
}

/// Compute hourly activity counts from events for the last 24 hours
fn compute_hourly_activity(events: &[EnrichedEvent]) -> Vec<(String, u64, u64)> {
    let now = chrono::Local::now();
    let cutoff = now - chrono::Duration::hours(24);

    // Initialize buckets for last 24 hours
    let mut counts: HashMap<String, (u64, u64)> = HashMap::new();

    for e in events {
        // Parse timestamp
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&e.timestamp) {
            let dt_local: chrono::DateTime<chrono::Local> = dt.into();
            if dt_local < cutoff {
                continue;
            }
            // Round down to hour
            let hour_key = dt_local.format("%Y-%m-%d %H:00").to_string();
            let entry = counts.entry(hour_key).or_insert((0, 0));
            match e.action.as_str() {
                "ADD" => entry.0 += 1,
                "TAP" => entry.1 += 1,
                _ => {}
            }
        }
    }

    // Sort by timestamp and format labels
    let mut result: Vec<_> = counts.into_iter().collect();
    result.sort_by_key(|(ts, _)| ts.clone());

    result
        .into_iter()
        .map(|(hour_key, (a, t))| {
            // Extract hour from "YYYY-MM-DD HH:00" format
            let label = hour_key.split_whitespace()
                .nth(1)
                .and_then(|time| time.split(':').next())
                .unwrap_or("??")
                .to_string();
            (label, a, t)
        })
        .collect()
}
