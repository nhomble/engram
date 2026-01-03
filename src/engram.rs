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
    fn from_event(conn: &Connection, event: db::Event) -> Self {
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

/// Main service layer struct - encapsulates database connection
pub struct Engram {
    conn: Connection,
}

impl Engram {
    /// Create a new Engram instance with the given config
    pub fn new(config: &Config) -> rusqlite::Result<Self> {
        let conn = db::open_db(config)?;
        Ok(Self { conn })
    }

    /// Create a new Engram instance from environment variables
    pub fn from_env() -> rusqlite::Result<Self> {
        let config = Config::from_env();
        Self::new(&config)
    }

    /// Get events with enriched content (TAP events include memory content)
    pub fn get_enriched_events(
        &self,
        limit: u32,
        action: Option<&str>,
        memory_id: Option<&str>,
    ) -> rusqlite::Result<Vec<EnrichedEvent>> {
        let events = db::get_events(&self.conn, limit, action, memory_id)?;
        Ok(events.into_iter().map(|e| EnrichedEvent::from_event(&self.conn, e)).collect())
    }

    /// Add a new memory
    pub fn add_memory(&self, content: &str) -> rusqlite::Result<String> {
        db::add_memory(&self.conn, content)
    }

    /// List memories, optionally including terminal states (promoted/forgotten)
    pub fn list_memories_filtered(&self, include_terminal: bool) -> rusqlite::Result<Vec<Memory>> {
        db::list_memories_filtered(&self.conn, include_terminal)
    }

    /// Get a specific memory by ID
    pub fn get_memory(&self, id: &str) -> rusqlite::Result<Option<Memory>> {
        db::get_memory(&self.conn, id)
    }

    /// Edit a memory's content
    pub fn edit_memory(&self, id: &str, new_content: &str) -> rusqlite::Result<bool> {
        db::edit_memory(&self.conn, id, new_content)
    }

    /// Forget a memory (mark as discarded)
    pub fn forget_memory(&self, id: &str) -> rusqlite::Result<bool> {
        db::forget_memory(&self.conn, id)
    }

    /// Promote a memory to permanent storage
    pub fn promote_memory(&self, id: &str) -> rusqlite::Result<Option<String>> {
        db::promote_memory(&self.conn, id)
    }

    /// Record a memory tap (usage)
    pub fn tap_memory(&self, id: &str) -> rusqlite::Result<bool> {
        db::tap_memory(&self.conn, id)
    }

    /// Tap memories matching a pattern
    pub fn tap_memories_by_match(&self, pattern: &str) -> rusqlite::Result<Vec<String>> {
        db::tap_memories_by_match(&self.conn, pattern)
    }
}
