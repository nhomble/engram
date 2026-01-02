use rusqlite::{Connection, Result, params};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct Memory {
    pub id: String,
    pub content: String,
    pub scope: String,
    pub generation: u8,
    pub tap_count: u32,
    pub last_tapped_at: Option<i64>,
    pub created_at: i64,
}

pub fn get_db_path() -> PathBuf {
    // Allow override via environment variable (useful for local dev/testing)
    if let Ok(path) = std::env::var("ENGRAM_DB_PATH") {
        return PathBuf::from(path);
    }

    // Default: ~/.engram/engram.db
    let home = std::env::var("HOME").expect("HOME not set");
    let data_dir = PathBuf::from(home).join(".engram");
    fs::create_dir_all(&data_dir).expect("Failed to create data directory");
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
            scope TEXT NOT NULL DEFAULT 'global',
            generation INTEGER NOT NULL DEFAULT 0,
            tap_count INTEGER NOT NULL DEFAULT 0,
            last_tapped_at INTEGER,
            created_at INTEGER NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_memories_scope ON memories(scope);
        CREATE INDEX IF NOT EXISTS idx_memories_generation ON memories(generation);

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

pub fn add_memory(conn: &Connection, content: &str, scope: &str) -> Result<String> {
    let id = generate_id();
    let created_at = now_timestamp();

    conn.execute(
        "INSERT INTO memories (id, content, scope, generation, tap_count, created_at)
         VALUES (?1, ?2, ?3, 0, 0, ?4)",
        params![id, content, scope, created_at],
    )?;

    // Log ADD event
    let data = format!(r#"{{"content":"{}","scope":"{}"}}"#,
        content.replace('\\', "\\\\").replace('"', "\\\""),
        scope.replace('\\', "\\\\").replace('"', "\\\""));
    log_event(conn, "ADD", Some(&id), Some(&data))?;

    Ok(id)
}

pub fn get_memory(conn: &Connection, id: &str) -> Result<Option<Memory>> {
    let mut stmt = conn.prepare(
        "SELECT id, content, scope, generation, tap_count, last_tapped_at, created_at
         FROM memories WHERE id = ?1"
    )?;

    let mut rows = stmt.query(params![id])?;

    if let Some(row) = rows.next()? {
        Ok(Some(Memory {
            id: row.get(0)?,
            content: row.get(1)?,
            scope: row.get(2)?,
            generation: row.get(3)?,
            tap_count: row.get(4)?,
            last_tapped_at: row.get(5)?,
            created_at: row.get(6)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn list_memories(conn: &Connection, scope: Option<&str>, gen: Option<u8>) -> Result<Vec<Memory>> {
    let mut sql = String::from(
        "SELECT id, content, scope, generation, tap_count, last_tapped_at, created_at
         FROM memories WHERE 1=1"
    );

    if scope.is_some() {
        sql.push_str(" AND scope = ?1");
    }
    if gen.is_some() {
        sql.push_str(if scope.is_some() { " AND generation = ?2" } else { " AND generation = ?1" });
    }

    sql.push_str(" ORDER BY created_at DESC");

    let mut stmt = conn.prepare(&sql)?;

    let rows = match (scope, gen) {
        (Some(s), Some(g)) => stmt.query(params![s, g])?,
        (Some(s), None) => stmt.query(params![s])?,
        (None, Some(g)) => stmt.query(params![g])?,
        (None, None) => stmt.query([])?,
    };

    let memories = rows.mapped(|row| {
        Ok(Memory {
            id: row.get(0)?,
            content: row.get(1)?,
            scope: row.get(2)?,
            generation: row.get(3)?,
            tap_count: row.get(4)?,
            last_tapped_at: row.get(5)?,
            created_at: row.get(6)?,
        })
    }).collect::<Result<Vec<_>>>()?;

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

/// Get memory statistics
pub fn get_stats(conn: &Connection) -> Result<MemoryStats> {
    let total: u32 = conn.query_row("SELECT COUNT(*) FROM memories", [], |row| row.get(0))?;

    let gen0: u32 = conn.query_row("SELECT COUNT(*) FROM memories WHERE generation = 0", [], |row| row.get(0))?;
    let gen1: u32 = conn.query_row("SELECT COUNT(*) FROM memories WHERE generation = 1", [], |row| row.get(0))?;
    let gen2: u32 = conn.query_row("SELECT COUNT(*) FROM memories WHERE generation = 2", [], |row| row.get(0))?;

    let total_taps: u32 = conn.query_row("SELECT COALESCE(SUM(tap_count), 0) FROM memories", [], |row| row.get(0))?;

    let never_tapped: u32 = conn.query_row("SELECT COUNT(*) FROM memories WHERE tap_count = 0", [], |row| row.get(0))?;

    // Scopes breakdown
    let mut stmt = conn.prepare("SELECT scope, COUNT(*) FROM memories GROUP BY scope ORDER BY COUNT(*) DESC")?;
    let scopes: Vec<(String, u32)> = stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?))
    })?.collect::<Result<Vec<_>>>()?;

    Ok(MemoryStats {
        total,
        by_generation: [gen0, gen1, gen2],
        total_taps,
        never_tapped,
        scopes,
    })
}

#[derive(Debug)]
pub struct MemoryStats {
    pub total: u32,
    pub by_generation: [u32; 3],
    pub total_taps: u32,
    pub never_tapped: u32,
    pub scopes: Vec<(String, u32)>,
}

/// Run garbage collection - expire memories with low engagement
/// Returns (expired_ids, promoted_ids)
pub fn run_gc(
    conn: &Connection,
    min_taps: u32,
    promote_threshold: u32,
    dry_run: bool,
) -> Result<GcResult> {
    let mut expired = Vec::new();
    let mut promoted = Vec::new();

    // Expire gen0 memories with fewer than min_taps
    let mut stmt = conn.prepare(
        "SELECT id, content, tap_count FROM memories
         WHERE generation = 0 AND tap_count < ?1"
    )?;

    let to_expire: Vec<(String, String, u32)> = stmt.query_map(params![min_taps], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    })?.collect::<Result<Vec<_>>>()?;

    for (id, content, taps) in to_expire {
        expired.push((id.clone(), content, taps));
        if !dry_run {
            conn.execute("DELETE FROM memories WHERE id = ?1", params![id])?;
            log_event(conn, "EXPIRE", Some(&id), Some(&format!(r#"{{"reason":"low_taps","taps":{}}}"#, taps)))?;
        }
    }

    // Promote gen0 memories with enough taps to gen1
    let mut stmt = conn.prepare(
        "SELECT id, content, tap_count FROM memories
         WHERE generation = 0 AND tap_count >= ?1"
    )?;

    let to_promote: Vec<(String, String, u32)> = stmt.query_map(params![promote_threshold], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    })?.collect::<Result<Vec<_>>>()?;

    for (id, content, taps) in to_promote {
        promoted.push((id.clone(), content, taps));
        if !dry_run {
            conn.execute("UPDATE memories SET generation = 1 WHERE id = ?1", params![id])?;
            log_event(conn, "PROMOTE", Some(&id), Some(r#"{"from":0,"to":1}"#))?;
        }
    }

    // Promote gen1 to gen2 if they have enough taps
    let mut stmt = conn.prepare(
        "SELECT id, content, tap_count FROM memories
         WHERE generation = 1 AND tap_count >= ?1"
    )?;

    let gen1_promote: Vec<(String, String, u32)> = stmt.query_map(params![promote_threshold * 2], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    })?.collect::<Result<Vec<_>>>()?;

    for (id, content, taps) in gen1_promote {
        promoted.push((id.clone(), content, taps));
        if !dry_run {
            conn.execute("UPDATE memories SET generation = 2 WHERE id = ?1", params![id])?;
            log_event(conn, "PROMOTE", Some(&id), Some(r#"{"from":1,"to":2}"#))?;
        }
    }

    Ok(GcResult { expired, promoted })
}

#[derive(Debug)]
pub struct GcResult {
    pub expired: Vec<(String, String, u32)>,  // id, content, taps
    pub promoted: Vec<(String, String, u32)>,
}

/// Hot memories - most tapped in recent time window
#[derive(Debug)]
pub struct HotMemory {
    pub id: String,
    pub content: String,
    pub recent_taps: u32,
    pub total_taps: u32,
}

/// Get memories with most TAP events in the last N seconds
pub fn get_hot_memories(conn: &Connection, window_secs: i64, limit: u32) -> Result<Vec<HotMemory>> {
    let cutoff = now_timestamp() - window_secs;

    let mut stmt = conn.prepare(
        "SELECT m.id, m.content, COUNT(e.id) as recent_taps, m.tap_count
         FROM memories m
         JOIN events e ON e.memory_id = m.id AND e.action = 'TAP' AND e.timestamp >= ?1
         GROUP BY m.id
         ORDER BY recent_taps DESC, m.tap_count DESC
         LIMIT ?2"
    )?;

    let rows = stmt.query_map(params![cutoff, limit], |row| {
        Ok(HotMemory {
            id: row.get(0)?,
            content: row.get(1)?,
            recent_taps: row.get(2)?,
            total_taps: row.get(3)?,
        })
    })?;

    rows.collect()
}

/// Activity summary for a time period
#[derive(Debug)]
pub struct ActivitySummary {
    pub period: String,
    pub adds: u32,
    pub taps: u32,
    pub removes: u32,
    pub reviews: u32,
}

/// Get activity summary grouped by day
pub fn get_activity_by_day(conn: &Connection, days: u32) -> Result<Vec<ActivitySummary>> {
    let cutoff = now_timestamp() - (days as i64 * 86400);

    let mut stmt = conn.prepare(
        "SELECT date(timestamp, 'unixepoch', 'localtime') as day,
                SUM(CASE WHEN action = 'ADD' THEN 1 ELSE 0 END) as adds,
                SUM(CASE WHEN action = 'TAP' THEN 1 ELSE 0 END) as taps,
                SUM(CASE WHEN action = 'REMOVE' THEN 1 ELSE 0 END) as removes,
                SUM(CASE WHEN action = 'REVIEW' THEN 1 ELSE 0 END) as reviews
         FROM events
         WHERE timestamp >= ?1
         GROUP BY day
         ORDER BY day DESC"
    )?;

    let rows = stmt.query_map(params![cutoff], |row| {
        Ok(ActivitySummary {
            period: row.get(0)?,
            adds: row.get(1)?,
            taps: row.get(2)?,
            removes: row.get(3)?,
            reviews: row.get(4)?,
        })
    })?;

    rows.collect()
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

        let id = add_memory(&conn, "test content", "global").expect("Failed to add memory");
        assert!(!id.is_empty());

        let memory = get_memory(&conn, &id).expect("Failed to get memory");
        assert!(memory.is_some());

        let m = memory.unwrap();
        assert_eq!(m.content, "test content");
        assert_eq!(m.scope, "global");
        assert_eq!(m.generation, 0);
    }

    #[test]
    fn test_tap_memory() {
        let conn = open_test_db();

        let id = add_memory(&conn, "tap test", "global").expect("Failed to add memory");

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

        let id = add_memory(&conn, "to remove", "global").expect("Failed to add memory");

        let removed = remove_memory(&conn, &id).expect("Failed to remove");
        assert!(removed);

        let memory = get_memory(&conn, &id).expect("Failed to get");
        assert!(memory.is_none());
    }

    #[test]
    fn test_edit_memory() {
        let conn = open_test_db();

        let id = add_memory(&conn, "original content", "global").expect("Failed to add memory");

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

        // Add memories with different scopes
        add_memory(&conn, "global memory", "global").unwrap();
        add_memory(&conn, "project memory", "project:/test").unwrap();

        // List all
        let all = list_memories(&conn, None, None).unwrap();
        assert_eq!(all.len(), 2);

        // Filter by scope
        let global = list_memories(&conn, Some("global"), None).unwrap();
        assert_eq!(global.len(), 1);
        assert_eq!(global[0].content, "global memory");
    }

    #[test]
    fn test_gc_expires_untapped() {
        let conn = open_test_db();

        // Add memory with 0 taps
        let id = add_memory(&conn, "untapped memory", "global").unwrap();

        // GC with min_taps=1 should expire it
        let result = run_gc(&conn, 1, 3, false).unwrap();
        assert_eq!(result.expired.len(), 1);
        assert_eq!(result.expired[0].0, id);

        // Memory should be gone
        assert!(get_memory(&conn, &id).unwrap().is_none());
    }

    #[test]
    fn test_gc_promotes_tapped() {
        let conn = open_test_db();

        // Add memory and tap it 3 times
        let id = add_memory(&conn, "tapped memory", "global").unwrap();
        tap_memory(&conn, &id).unwrap();
        tap_memory(&conn, &id).unwrap();
        tap_memory(&conn, &id).unwrap();

        // GC with promote_threshold=3 should promote to gen1
        let result = run_gc(&conn, 1, 3, false).unwrap();
        assert_eq!(result.promoted.len(), 1);

        // Memory should be gen1 now
        let m = get_memory(&conn, &id).unwrap().unwrap();
        assert_eq!(m.generation, 1);
    }
}
