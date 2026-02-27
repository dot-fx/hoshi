CREATE TABLE IF NOT EXISTS tracker_mappings (
    cid TEXT NOT NULL,
    tracker_name TEXT NOT NULL,
    tracker_id TEXT NOT NULL,
    tracker_url TEXT,
    sync_enabled INTEGER NOT NULL DEFAULT 1,
    last_synced INTEGER,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (cid, tracker_name),
    FOREIGN KEY (cid) REFERENCES core_metadata(cid) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_tracker_name ON tracker_mappings(tracker_name);
CREATE INDEX IF NOT EXISTS idx_tracker_id ON tracker_mappings(tracker_id);
