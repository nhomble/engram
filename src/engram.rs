/// Service layer for engram - business logic and orchestration
///
/// This sits between:
/// - db.rs: Pure CRUD operations
/// - main.rs/tui.rs: Presentation layers
///
/// This is the ONLY module that should import db.rs.
/// Presentation layers (main.rs, tui.rs) should only import engram.rs.

use crate::db;
use rusqlite::Connection;

// Re-export types from db layer
pub use db::{Config, Memory};

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

// Database connection wrapper
pub fn open_db(config: &Config) -> rusqlite::Result<Connection> {
    db::open_db(config)
}

// Memory CRUD operations
pub fn add_memory(conn: &Connection, content: &str) -> rusqlite::Result<String> {
    db::add_memory(conn, content)
}

pub fn list_memories_filtered(conn: &Connection, include_terminal: bool) -> rusqlite::Result<Vec<Memory>> {
    db::list_memories_filtered(conn, include_terminal)
}

pub fn get_memory(conn: &Connection, id: &str) -> rusqlite::Result<Option<Memory>> {
    db::get_memory(conn, id)
}

pub fn edit_memory(conn: &Connection, id: &str, new_content: &str) -> rusqlite::Result<bool> {
    db::edit_memory(conn, id, new_content)
}

pub fn forget_memory(conn: &Connection, id: &str) -> rusqlite::Result<bool> {
    db::forget_memory(conn, id)
}

pub fn promote_memory(conn: &Connection, id: &str) -> rusqlite::Result<Option<String>> {
    db::promote_memory(conn, id)
}

// Tap operations
pub fn tap_memory(conn: &Connection, id: &str) -> rusqlite::Result<bool> {
    db::tap_memory(conn, id)
}

pub fn tap_memories_by_match(conn: &Connection, pattern: &str) -> rusqlite::Result<Vec<String>> {
    db::tap_memories_by_match(conn, pattern)
}
