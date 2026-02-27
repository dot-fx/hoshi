CREATE TABLE IF NOT EXISTS content_units (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cid TEXT NOT NULL,
    unit_number REAL NOT NULL,
    type TEXT NOT NULL,
    title TEXT,
    description TEXT,
    thumbnail_url TEXT,
    released_at TEXT,
    duration INTEGER,
    absolute_number INTEGER,
    created_at INTEGER NOT NULL,
    FOREIGN KEY(cid) REFERENCES core_metadata(cid) ON DELETE CASCADE,
    UNIQUE(cid, type, unit_number)
);

CREATE INDEX IF NOT EXISTS idx_units_cid ON content_units(cid);
CREATE INDEX IF NOT EXISTS idx_units_type ON content_units(type);
