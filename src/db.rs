use rusqlite::{Connection, Result, params};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct Memory {
    pub id: String,
    pub content: String,
    pub tap_count: u32,
    pub last_tapped_at: Option<i64>,
    pub created_at: i64,
}

pub fn get_db_path() -> PathBuf {
    // Allow override via environment variable (useful for testing)
    if let Ok(path) = std::env::var("ENGRAM_DB_PATH") {
        return PathBuf::from(path);
    }

    // Default: .engram/engram.db in current directory
    let data_dir = PathBuf::from(".engram");
    fs::create_dir_all(&data_dir).expect("Failed to create .engram directory");
    data_dir.join("engram.db")
}

pub fn open_db() -> Result<Connection> {
    let db_path = get_db_path();
    let conn = Connection::open(&db_path)?;

    // Enable WAL mode for better concurrency
    conn.pragma_update(None, "journal_mode", "WAL")?;

    // Set busy timeout to 5 seconds for retry-on-busy
    conn.busy_timeout(std::time::Duration::from_secs(5))?;

    // Initialize schema
    init_schema(&conn)?;

    Ok(conn)
}

fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS memories (
            id TEXT PRIMARY KEY,
            content TEXT NOT NULL,
            tap_count INTEGER NOT NULL DEFAULT 0,
            last_tapped_at INTEGER,
            created_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp INTEGER NOT NULL,
            action TEXT NOT NULL,
            memory_id TEXT,
            data TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events(timestamp);
        CREATE INDEX IF NOT EXISTS idx_events_action ON events(action);
        CREATE INDEX IF NOT EXISTS idx_events_memory_id ON events(memory_id);
        "
    )?;

    Ok(())
}

/// Log an event to the event log
pub fn log_event(conn: &Connection, action: &str, memory_id: Option<&str>, data: Option<&str>) -> Result<()> {
    conn.execute(
        "INSERT INTO events (timestamp, action, memory_id, data) VALUES (?1, ?2, ?3, ?4)",
        params![now_timestamp(), action, memory_id, data],
    )?;
    Ok(())
}

#[derive(Debug)]
pub struct Event {
    pub timestamp: i64,
    pub action: String,
    pub memory_id: Option<String>,
    pub data: Option<String>,
}

/// Get events from the event log
pub fn get_events(
    conn: &Connection,
    limit: u32,
    action: Option<&str>,
    memory_id: Option<&str>,
) -> Result<Vec<Event>> {
    let mut sql = String::from(
        "SELECT timestamp, action, memory_id, data FROM events WHERE 1=1"
    );
    let mut param_idx = 1;

    if action.is_some() {
        sql.push_str(&format!(" AND action = ?{}", param_idx));
        param_idx += 1;
    }
    if memory_id.is_some() {
        sql.push_str(&format!(" AND memory_id = ?{}", param_idx));
    }

    sql.push_str(&format!(" ORDER BY id DESC LIMIT {}", limit));

    let mut stmt = conn.prepare(&sql)?;

    let events = match (action, memory_id) {
        (Some(a), Some(m)) => stmt.query_map(params![a, m], row_to_event)?,
        (Some(a), None) => stmt.query_map(params![a], row_to_event)?,
        (None, Some(m)) => stmt.query_map(params![m], row_to_event)?,
        (None, None) => stmt.query_map([], row_to_event)?,
    };

    events.collect()
}

fn row_to_event(row: &rusqlite::Row) -> rusqlite::Result<Event> {
    Ok(Event {
        timestamp: row.get(0)?,
        action: row.get(1)?,
        memory_id: row.get(2)?,
        data: row.get(3)?,
    })
}

fn generate_id() -> String {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let random = RandomState::new().build_hasher().finish();
    format!("{:x}{:x}", now as u64, random)
}

fn now_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

// CRUD operations

pub fn add_memory(conn: &Connection, content: &str) -> Result<String> {
    let id = generate_id();
    let created_at = now_timestamp();

    conn.execute(
        "INSERT INTO memories (id, content, tap_count, created_at)
         VALUES (?1, ?2, 0, ?3)",
        params![id, content, created_at],
    )?;

    // Log ADD event
    let data = format!(r#"{{"content":"{}"}}"#,
        content.replace('\\', "\\\\").replace('"', "\\\""));
    log_event(conn, "ADD", Some(&id), Some(&data))?;

    Ok(id)
}

pub fn get_memory(conn: &Connection, id: &str) -> Result<Option<Memory>> {
    let mut stmt = conn.prepare(
        "SELECT id, content, tap_count, last_tapped_at, created_at
         FROM memories WHERE id = ?1"
    )?;

    let mut rows = stmt.query(params![id])?;

    if let Some(row) = rows.next()? {
        Ok(Some(Memory {
            id: row.get(0)?,
            content: row.get(1)?,
            tap_count: row.get(2)?,
            last_tapped_at: row.get(3)?,
            created_at: row.get(4)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn list_memories(conn: &Connection) -> Result<Vec<Memory>> {
    let mut stmt = conn.prepare(
        "SELECT id, content, tap_count, last_tapped_at, created_at
         FROM memories
         ORDER BY tap_count DESC, created_at DESC"
    )?;

    let memories = stmt.query_map([], |row| {
        Ok(Memory {
            id: row.get(0)?,
            content: row.get(1)?,
            tap_count: row.get(2)?,
            last_tapped_at: row.get(3)?,
            created_at: row.get(4)?,
        })
    })?.collect::<Result<Vec<_>>>()?;

    Ok(memories)
}

pub fn remove_memory(conn: &Connection, id: &str) -> Result<bool> {
    let rows_affected = conn.execute("DELETE FROM memories WHERE id = ?1", params![id])?;
    if rows_affected > 0 {
        log_event(conn, "REMOVE", Some(id), None)?;
    }
    Ok(rows_affected > 0)
}

/// Edit a memory's content
pub fn edit_memory(conn: &Connection, id: &str, new_content: &str) -> Result<bool> {
    // Get old content for event log
    let old_content: Option<String> = conn.query_row(
        "SELECT content FROM memories WHERE id = ?1",
        params![id],
        |row| row.get(0),
    ).ok();

    let rows_affected = conn.execute(
        "UPDATE memories SET content = ?1 WHERE id = ?2",
        params![new_content, id],
    )?;

    if rows_affected > 0 {
        let data = format!(r#"{{"old":"{}","new":"{}"}}"#,
            old_content.unwrap_or_default().replace('\\', "\\\\").replace('"', "\\\""),
            new_content.replace('\\', "\\\\").replace('"', "\\\""));
        log_event(conn, "EDIT", Some(id), Some(&data))?;
    }
    Ok(rows_affected > 0)
}

/// Tap a memory by ID - increments tap_count and updates last_tapped_at
pub fn tap_memory(conn: &Connection, id: &str) -> Result<bool> {
    let rows_affected = conn.execute(
        "UPDATE memories SET tap_count = tap_count + 1, last_tapped_at = ?1 WHERE id = ?2",
        params![now_timestamp(), id],
    )?;
    if rows_affected > 0 {
        log_event(conn, "TAP", Some(id), None)?;
    }
    Ok(rows_affected > 0)
}

/// Tap memories matching a substring - returns list of tapped IDs
pub fn tap_memories_by_match(conn: &Connection, pattern: &str) -> Result<Vec<String>> {
    let search = format!("%{}%", pattern);
    let timestamp = now_timestamp();

    // First get the IDs that will be tapped
    let mut stmt = conn.prepare("SELECT id FROM memories WHERE content LIKE ?1")?;
    let ids: Vec<String> = stmt.query_map(params![search], |row| row.get(0))?
        .collect::<Result<Vec<_>>>()?;

    // Then update them
    conn.execute(
        "UPDATE memories SET tap_count = tap_count + 1, last_tapped_at = ?1 WHERE content LIKE ?2",
        params![timestamp, search],
    )?;

    // Log TAP events for each matched memory
    for id in &ids {
        log_event(conn, "TAP", Some(id), None)?;
    }

    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_test_db() -> Connection {
        let conn = Connection::open_in_memory().expect("Failed to open in-memory database");
        init_schema(&conn).expect("Failed to init schema");
        conn
    }

    #[test]
    fn test_schema_init() {
        let conn = open_test_db();
        // Verify table exists by querying it
        let count: u32 = conn
            .query_row("SELECT COUNT(*) FROM memories", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_add_and_get_memory() {
        let conn = open_test_db();

        let id = add_memory(&conn, "test content").expect("Failed to add memory");
        assert!(!id.is_empty());

        let memory = get_memory(&conn, &id).expect("Failed to get memory");
        assert!(memory.is_some());

        let m = memory.unwrap();
        assert_eq!(m.content, "test content");
    }

    #[test]
    fn test_tap_memory() {
        let conn = open_test_db();

        let id = add_memory(&conn, "tap test").expect("Failed to add memory");

        // Initial state
        let m = get_memory(&conn, &id).unwrap().unwrap();
        assert_eq!(m.tap_count, 0);

        // Tap it
        let tapped = tap_memory(&conn, &id).expect("Failed to tap");
        assert!(tapped);

        // Verify tap count increased
        let m = get_memory(&conn, &id).unwrap().unwrap();
        assert_eq!(m.tap_count, 1);
        assert!(m.last_tapped_at.is_some());
    }

    #[test]
    fn test_remove_memory() {
        let conn = open_test_db();

        let id = add_memory(&conn, "to remove").expect("Failed to add memory");

        let removed = remove_memory(&conn, &id).expect("Failed to remove");
        assert!(removed);

        let memory = get_memory(&conn, &id).expect("Failed to get");
        assert!(memory.is_none());
    }

    #[test]
    fn test_edit_memory() {
        let conn = open_test_db();

        let id = add_memory(&conn, "original content").expect("Failed to add memory");

        // Verify original
        let m = get_memory(&conn, &id).unwrap().unwrap();
        assert_eq!(m.content, "original content");

        // Edit it
        let edited = edit_memory(&conn, &id, "updated content").expect("Failed to edit");
        assert!(edited);

        // Verify update
        let m = get_memory(&conn, &id).unwrap().unwrap();
        assert_eq!(m.content, "updated content");

        // Edit non-existent returns false
        let edited = edit_memory(&conn, "nonexistent", "new").expect("Failed to edit");
        assert!(!edited);
    }

    #[test]
    fn test_list_memories() {
        let conn = open_test_db();

        add_memory(&conn, "first memory").unwrap();
        add_memory(&conn, "second memory").unwrap();

        let all = list_memories(&conn).unwrap();
        assert_eq!(all.len(), 2);
    }
}
