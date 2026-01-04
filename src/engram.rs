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
    /// Extract clean content from JSON event data
    fn extract_content(action: &str, json_data: &str) -> String {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_data) {
            match action {
                "ADD" | "PROMOTE" => {
                    // Extract "content" field from {"content":"..."}
                    parsed.get("content")
                        .and_then(|v| v.as_str())
                        .unwrap_or(json_data)
                        .to_string()
                }
                "EDIT" => {
                    // Extract "new" field from {"old":"...","new":"..."}
                    parsed.get("new")
                        .and_then(|v| v.as_str())
                        .unwrap_or(json_data)
                        .to_string()
                }
                _ => json_data.to_string()
            }
        } else {
            // If JSON parse fails, return raw data
            json_data.to_string()
        }
    }

    /// Create from a db::Event, enriching TAP events with memory content
    fn from_event(conn: &Connection, event: db::Event) -> Self {
        let content = if let Some(data) = event.data {
            // Extract clean content from JSON data (ADD, PROMOTE, EDIT, etc.)
            Self::extract_content(&event.action, &data)
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
    /// By default, filters out TAP events for promoted memories (they're in CLAUDE.md now)
    pub fn get_enriched_events(
        &self,
        limit: u32,
        action: Option<&str>,
        memory_id: Option<&str>,
        include_promoted_taps: bool,
    ) -> rusqlite::Result<Vec<EnrichedEvent>> {
        let events = db::get_events(&self.conn, limit, action, memory_id)?;

        // Get promoted memory IDs if we need to filter
        let promoted_ids = if !include_promoted_taps {
            db::get_promoted_memory_ids(&self.conn).unwrap_or_default()
        } else {
            vec![]
        };

        let enriched: Vec<EnrichedEvent> = events
            .into_iter()
            .filter(|e| {
                // Filter out TAP events for promoted memories unless include_promoted_taps is true
                if !include_promoted_taps && e.action == "TAP" {
                    if let Some(ref mem_id) = e.memory_id {
                        return !promoted_ids.contains(mem_id);
                    }
                }
                true
            })
            .map(|e| EnrichedEvent::from_event(&self.conn, e))
            .collect();

        Ok(enriched)
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

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn create_test_engram() -> Engram {
        let conn = Connection::open_in_memory().expect("Failed to open in-memory database");
        db::init_schema(&conn).expect("Failed to init schema");
        Engram { conn }
    }

    #[test]
    fn test_enriched_event_tap_lookup() {
        let engram = create_test_engram();

        // Add a memory
        let id = engram.add_memory("Test memory content").unwrap();

        // Tap the memory
        engram.tap_memory(&id).unwrap();

        // Get enriched events - should include memory content for TAP event
        let events = engram.get_enriched_events(10, None, None, true).unwrap();

        assert_eq!(events.len(), 2); // ADD and TAP events
        let tap_event = events.iter().find(|e| e.action == "TAP").unwrap();
        assert_eq!(tap_event.content, "Test memory content");
        assert_eq!(tap_event.memory_id, Some(id));
    }

    #[test]
    fn test_multiple_taps_same_memory() {
        let engram = create_test_engram();

        // Add and tap a memory multiple times
        let id = engram.add_memory("Memory to tap").unwrap();
        engram.tap_memory(&id).unwrap();
        engram.tap_memory(&id).unwrap();
        engram.tap_memory(&id).unwrap();

        // Get all events
        let events = engram.get_enriched_events(100, None, None, true).unwrap();

        // Should have ADD + 3 TAP events
        assert_eq!(events.len(), 4);
        let tap_count = events.iter().filter(|e| e.action == "TAP").count();
        assert_eq!(tap_count, 3);

        // All TAP events should have the same content
        let tap_events: Vec<_> = events.iter().filter(|e| e.action == "TAP").collect();
        assert!(tap_events.iter().all(|e| e.content == "Memory to tap"));
    }

    #[test]
    fn test_end_to_end_memory_lifecycle() {
        let engram = create_test_engram();

        // 1. Add memory
        let id = engram.add_memory("Lifecycle test").unwrap();
        let events = engram.get_enriched_events(100, None, None, true).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].action, "ADD");

        // 2. Tap memory
        engram.tap_memory(&id).unwrap();
        let events = engram.get_enriched_events(100, None, None, true).unwrap();
        assert_eq!(events.len(), 2);

        // 3. Edit memory
        engram.edit_memory(&id, "Updated content").unwrap();
        let events = engram.get_enriched_events(100, None, None, true).unwrap();
        assert_eq!(events.len(), 3);
        let edit_event = events.iter().find(|e| e.action == "EDIT").unwrap();
        assert!(edit_event.content.contains("Updated content"));

        // 4. Promote memory
        let content = engram.promote_memory(&id).unwrap();
        assert_eq!(content, Some("Updated content".to_string()));

        let events = engram.get_enriched_events(100, None, None, true).unwrap();
        assert_eq!(events.len(), 4);

        // 5. Verify promoted memory is filtered from list
        let memories = engram.list_memories_filtered(false).unwrap();
        assert_eq!(memories.len(), 0); // promoted memory excluded

        let memories_all = engram.list_memories_filtered(true).unwrap();
        assert_eq!(memories_all.len(), 1); // included with --all
    }

    #[test]
    fn test_enriched_event_content_types() {
        let engram = create_test_engram();

        let id = engram.add_memory("Test content").unwrap();

        // ADD event has content in data field
        let events = engram.get_enriched_events(100, Some("ADD"), None, true).unwrap();
        assert_eq!(events.len(), 1);
        assert!(events[0].content.contains("Test content"));

        // TAP event looks up memory content
        engram.tap_memory(&id).unwrap();
        let events = engram.get_enriched_events(100, Some("TAP"), None, true).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].content, "Test content");

        // FORGET event has no content
        engram.forget_memory(&id).unwrap();
        let events = engram.get_enriched_events(100, Some("FORGET"), None, true).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].content, "(none)");
    }
}
