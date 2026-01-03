/// Service layer for engram - business logic and orchestration
///
/// This sits between:
/// - db.rs: Pure CRUD operations
/// - main.rs/tui.rs: Presentation layers

use crate::db;
use rusqlite::Connection;

/// Event with enriched content - looks up memory content for TAP events
#[derive(Debug)]
pub struct EnrichedEvent {
    pub timestamp: String,
    pub action: String,
    pub memory_id: Option<String>,
    pub content: String,  // Either event data or looked-up memory content
}

impl EnrichedEvent {
    /// Create from a db::Event, enriching TAP events with memory content
    pub fn from_event(conn: &Connection, event: db::Event) -> Self {
        let content = if event.data.is_some() {
            // Use event data if present (ADD, PROMOTE, EDIT, etc.)
            event.data.unwrap()
        } else if event.action == "TAP" {
            // Look up memory content for TAP events
            if let Some(ref mem_id) = event.memory_id {
                db::get_memory(conn, mem_id)
                    .ok()
                    .flatten()
                    .map(|m| m.content)
                    .unwrap_or_else(|| "(memory not found)".to_string())
            } else {
                "(no memory id)".to_string()
            }
        } else {
            // FORGET events and others with no data
            "(none)".to_string()
        };

        EnrichedEvent {
            timestamp: event.timestamp,
            action: event.action,
            memory_id: event.memory_id,
            content,
        }
    }
}

/// Get events with enriched content (TAP events include memory content)
pub fn get_enriched_events(
    conn: &Connection,
    limit: u32,
    action: Option<&str>,
    memory_id: Option<&str>,
) -> rusqlite::Result<Vec<EnrichedEvent>> {
    let events = db::get_events(conn, limit, action, memory_id)?;
    Ok(events.into_iter().map(|e| EnrichedEvent::from_event(conn, e)).collect())
}
