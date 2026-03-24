CREATE TABLE IF NOT EXISTS airing_schedule (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    cid        TEXT    NOT NULL REFERENCES content(cid) ON DELETE CASCADE,
    episode    INTEGER NOT NULL,
    airing_at  INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE(cid, episode)
);

CREATE INDEX IF NOT EXISTS idx_airing_cid       ON airing_schedule(cid);
CREATE INDEX IF NOT EXISTS idx_airing_airing_at ON airing_schedule(airing_at);