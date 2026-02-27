CREATE TABLE IF NOT EXISTS core_metadata (
    cid TEXT PRIMARY KEY,
    type TEXT NOT NULL CHECK(type IN ('anime', 'manga', 'novel')),
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
    nsfw INTEGER NOT NULL DEFAULT 0,
    release_date TEXT,
    end_date TEXT,
    rating REAL,
    trailer_url TEXT,
    characters TEXT NOT NULL DEFAULT '[]',
    studio TEXT,
    staff TEXT NOT NULL DEFAULT '[]',
    sources TEXT,
    external_ids TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_core_type ON core_metadata(type);
CREATE INDEX IF NOT EXISTS idx_core_nsfw ON core_metadata(nsfw);
CREATE INDEX IF NOT EXISTS idx_core_status ON core_metadata(status);
CREATE INDEX IF NOT EXISTS idx_core_title ON core_metadata(title);
