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
        // Draw
        terminal.draw(|frame| {
            let area = frame.area();

            // Get events from DB
            let content = match db::open_db() {
                Ok(conn) => {
                    match db::get_events(&conn, 50, None, None) {
                        Ok(events) => {
                            events.iter()
                                .map(|e| {
                                    let time = format_timestamp(e.timestamp);
                                    let mem_id = e.memory_id.as_deref().unwrap_or("-");
                                    let short_id = if mem_id.len() > 8 { &mem_id[..8] } else { mem_id };
                                    let data = e.data.as_deref().unwrap_or("");
                                    let data_preview = if data.len() > 50 { &data[..50] } else { data };
                                    format!("{} {:8} {} {}", time, e.action, short_id, data_preview)
                                })
                                .collect::<Vec<_>>()
                                .join("\n")
                        }
                        Err(e) => format!("Error: {}", e),
                    }
                }
                Err(e) => format!("DB Error: {}", e),
            };

            let block = Block::default()
                .title(" engram log (q to quit) ")
                .borders(Borders::ALL);

            let paragraph = Paragraph::new(content)
                .block(block);

            frame.render_widget(paragraph, area);
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
