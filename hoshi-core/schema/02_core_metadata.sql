CREATE TABLE IF NOT EXISTS content (
    cid TEXT PRIMARY KEY,
    type TEXT NOT NULL CHECK(type IN ('anime', 'manga', 'novel')),
    nsfw INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS metadata (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cid TEXT NOT NULL,
    source_name TEXT NOT NULL,
    source_id TEXT,
    subtype TEXT,
    title TEXT NOT NULL,
    alt_titles TEXT NOT NULL DEFAULT '[]',
    synopsis TEXT,
    cover_image TEXT,
    banner_image TEXT,
    eps_or_chapters TEXT,
    status TEXT,
    tags TEXT NOT NULL DEFAULT '[]',
    genres TEXT NOT NULL DEFAULT '[]',
    release_date TEXT,
    end_date TEXT,
    rating REAL,
    trailer_url TEXT,
    characters TEXT NOT NULL DEFAULT '[]',
    studio TEXT,
    staff TEXT NOT NULL DEFAULT '[]',
    external_ids TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE(cid, source_name),
    FOREIGN KEY (cid) REFERENCES content(cid) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_metadata_cid ON metadata(cid);
CREATE INDEX IF NOT EXISTS idx_metadata_source ON metadata(source_name);
CREATE INDEX IF NOT EXISTS idx_metadata_title ON metadata(title);

CREATE TABLE IF NOT EXISTS AnimeProgress (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    cid TEXT NOT NULL,
    episode INTEGER NOT NULL,
    timestamp_seconds INTEGER NOT NULL DEFAULT 0,
    episode_duration_seconds INTEGER,
    completed INTEGER NOT NULL DEFAULT 0,
    last_accessed INTEGER NOT NULL DEFAULT (unixepoch()),
    UNIQUE(user_id, cid, episode),
    FOREIGN KEY (user_id) REFERENCES User(id) ON DELETE CASCADE,
    FOREIGN KEY (cid) REFERENCES content(cid) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS ChapterProgress (
   id INTEGER PRIMARY KEY AUTOINCREMENT,
   user_id INTEGER NOT NULL,
   cid TEXT NOT NULL,
   chapter INTEGER NOT NULL,
   completed INTEGER NOT NULL DEFAULT 0,
   last_accessed INTEGER NOT NULL DEFAULT (unixepoch()),
    UNIQUE(user_id, cid, chapter),
    FOREIGN KEY (user_id) REFERENCES User(id) ON DELETE CASCADE,
    FOREIGN KEY (cid) REFERENCES content(cid) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_anime_progress_user_accessed ON AnimeProgress(user_id, last_accessed DESC);
CREATE INDEX IF NOT EXISTS idx_anime_progress_user_cid ON AnimeProgress(user_id, cid);
CREATE INDEX IF NOT EXISTS idx_chapter_progress_user_accessed ON ChapterProgress(user_id, last_accessed DESC);
CREATE INDEX IF NOT EXISTS idx_chapter_progress_user_cid ON ChapterProgress(user_id, cid);