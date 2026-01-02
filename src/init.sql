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
