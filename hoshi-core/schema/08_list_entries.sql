CREATE TABLE IF NOT EXISTS ListEntry (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    cid TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('CURRENT', 'PLANNING', 'COMPLETED', 'PAUSED', 'DROPPED', 'REPEATING')),
    progress INTEGER NOT NULL DEFAULT 0,
    score REAL,
    start_date DATE,
    end_date DATE,
    repeat_count INTEGER NOT NULL DEFAULT 0,
    notes TEXT,
    is_private INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (user_id, cid),
    FOREIGN KEY (user_id) REFERENCES User(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_list_user ON ListEntry(user_id);
CREATE INDEX IF NOT EXISTS idx_list_cid ON ListEntry(cid);
CREATE INDEX IF NOT EXISTS idx_list_status ON ListEntry(status);

CREATE TABLE IF NOT EXISTS ListBackup (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id      INTEGER NOT NULL,
    trigger      TEXT NOT NULL CHECK(trigger IN ('PRE_IMPORT', 'MANUAL')),
    tracker_name TEXT,
    file_path    TEXT NOT NULL,
    entry_count  INTEGER NOT NULL,
    created_at   INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES User(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_backup_user_created ON ListBackup(user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_backup_user_tracker  ON ListBackup(user_id, tracker_name);