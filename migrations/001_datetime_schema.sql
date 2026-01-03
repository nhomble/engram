-- Migration: Convert INTEGER timestamps to TEXT (ISO 8601 datetime)
-- Run with: sqlite3 .engram/engram.db < migrations/001_datetime_schema.sql

BEGIN TRANSACTION;

-- Step 1: Convert data from Unix timestamps to ISO 8601 datetime strings
UPDATE memories SET created_at = datetime(CAST(created_at AS INTEGER), 'unixepoch', 'localtime') || '+00:00' WHERE typeof(created_at) = 'integer';
UPDATE memories SET last_tapped_at = datetime(CAST(last_tapped_at AS INTEGER), 'unixepoch', 'localtime') || '+00:00' WHERE typeof(last_tapped_at) = 'integer' AND last_tapped_at IS NOT NULL;
UPDATE events SET timestamp = datetime(CAST(timestamp AS INTEGER), 'unixepoch', 'localtime') || '+00:00' WHERE typeof(timestamp) = 'integer';

-- Step 2: Recreate memories table with TEXT schema
CREATE TABLE memories_new (
    id TEXT PRIMARY KEY,
    content TEXT NOT NULL,
    tap_count INTEGER NOT NULL DEFAULT 0,
    last_tapped_at TEXT,
    created_at TEXT NOT NULL
);

INSERT INTO memories_new SELECT * FROM memories;
DROP TABLE memories;
ALTER TABLE memories_new RENAME TO memories;

-- Step 3: Recreate events table with TEXT schema
CREATE TABLE events_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    action TEXT NOT NULL,
    memory_id TEXT,
    data TEXT
);

INSERT INTO events_new (id, timestamp, action, memory_id, data)
SELECT id, timestamp, action, memory_id, data FROM events;

DROP TABLE events;
ALTER TABLE events_new RENAME TO events;

-- Step 4: Recreate indexes
CREATE INDEX idx_events_timestamp ON events(timestamp);
CREATE INDEX idx_events_action ON events(action);
CREATE INDEX idx_events_memory_id ON events(memory_id);

COMMIT;
