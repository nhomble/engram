use std::io;
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::db;

#[derive(PartialEq, Clone, Copy)]
enum Panel {
    Memories,
    Events,
}

struct AppState {
    focused: Panel,
    memories_state: ListState,
    events_state: ListState,
    memories_count: usize,
    events_count: usize,
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

    loop {
        terminal.draw(|frame| {
            let area = frame.area();

            // Split screen: memories on top, events on bottom
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(40),
                    Constraint::Percentage(60),
                ])
                .split(area);

            // Get data from DB
            let (memories_items, events_items): (Vec<ListItem>, Vec<ListItem>) = match db::open_db() {
                Ok(conn) => {
                    let memories: Vec<ListItem> = match db::list_memories(&conn) {
                        Ok(mems) => {
                            mems.iter()
                                .map(|m| {
                                    let short_id = if m.id.len() > 8 { &m.id[..8] } else { &m.id };
                                    let content = truncate(&m.content, 60);
                                    ListItem::new(format!("[{}] taps:{:2} | {}", short_id, m.tap_count, content))
                                })
                                .collect()
                        }
                        Err(e) => vec![ListItem::new(format!("Error: {}", e))],
                    };

                    let events: Vec<ListItem> = match db::get_events(&conn, 50, None, None) {
                        Ok(evts) => {
                            evts.iter()
                                .map(|e| {
                                    let time = format_timestamp(e.timestamp);
                                    let mem_id = e.memory_id.as_deref().unwrap_or("-");
                                    let short_id = if mem_id.len() > 8 { &mem_id[..8] } else { mem_id };
                                    let data = e.data.as_deref().unwrap_or("");
                                    let data_preview = truncate(data, 40);
                                    ListItem::new(format!("{} {:6} {} {}", time, e.action, short_id, data_preview))
                                })
                                .collect()
                        }
                        Err(e) => vec![ListItem::new(format!("Error: {}", e))],
                    };

                    (memories, events)
                }
                Err(e) => (vec![ListItem::new(format!("DB Error: {}", e))], vec![]),
            };

            // Update counts for navigation
            state.memories_count = memories_items.len();
            state.events_count = events_items.len();

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
            frame.render_stateful_widget(memories_list, chunks[0], &mut state.memories_state);

            // Render events panel
            let events_title = if state.focused == Panel::Events {
                " Events [*] (q:quit Tab:switch j/k:nav) "
            } else {
                " Events (q:quit Tab:switch j/k:nav) "
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
            frame.render_stateful_widget(events_list, chunks[1], &mut state.events_state);
        })?;

        // Handle input (with timeout for refresh)
        if event::poll(Duration::from_secs(1))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('j') | KeyCode::Down => state.move_down(),
                        KeyCode::Char('k') | KeyCode::Up => state.move_up(),
                        KeyCode::Tab => state.toggle_panel(),
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}

fn format_timestamp(ts: db::Timestamp) -> String {
    use std::time::{Duration, UNIX_EPOCH};
    let dt = UNIX_EPOCH + Duration::from_secs(ts as u64);
    let datetime: chrono::DateTime<chrono::Local> = dt.into();
    datetime.format("%H:%M:%S").to_string()
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
