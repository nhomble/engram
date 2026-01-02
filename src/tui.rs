use std::io;
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

use crate::db;

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
            let (memories_content, events_content) = match db::open_db() {
                Ok(conn) => {
                    let memories = match db::list_memories(&conn) {
                        Ok(mems) => {
                            if mems.is_empty() {
                                "No memories.".to_string()
                            } else {
                                mems.iter()
                                    .map(|m| {
                                        let short_id = if m.id.len() > 8 { &m.id[..8] } else { &m.id };
                                        let content = truncate(&m.content, 60);
                                        format!("[{}] taps:{:2} | {}", short_id, m.tap_count, content)
                                    })
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            }
                        }
                        Err(e) => format!("Error: {}", e),
                    };

                    let events = match db::get_events(&conn, 30, None, None) {
                        Ok(evts) => {
                            if evts.is_empty() {
                                "No events.".to_string()
                            } else {
                                evts.iter()
                                    .map(|e| {
                                        let time = format_timestamp(e.timestamp);
                                        let mem_id = e.memory_id.as_deref().unwrap_or("-");
                                        let short_id = if mem_id.len() > 8 { &mem_id[..8] } else { mem_id };
                                        let data = e.data.as_deref().unwrap_or("");
                                        let data_preview = truncate(data, 40);
                                        format!("{} {:6} {} {}", time, e.action, short_id, data_preview)
                                    })
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            }
                        }
                        Err(e) => format!("Error: {}", e),
                    };

                    (memories, events)
                }
                Err(e) => (format!("DB Error: {}", e), String::new()),
            };

            // Render memories panel
            let memories_block = Block::default()
                .title(" Memories ")
                .borders(Borders::ALL);
            let memories_widget = Paragraph::new(memories_content).block(memories_block);
            frame.render_widget(memories_widget, chunks[0]);

            // Render events panel
            let events_block = Block::default()
                .title(" Events (q to quit) ")
                .borders(Borders::ALL);
            let events_widget = Paragraph::new(events_content).block(events_block);
            frame.render_widget(events_widget, chunks[1]);
        })?;

        // Handle input (with timeout for refresh)
        if event::poll(Duration::from_secs(1))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    break;
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
