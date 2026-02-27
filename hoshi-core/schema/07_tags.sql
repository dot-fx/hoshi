CREATE TABLE IF NOT EXISTS content_tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cid TEXT NOT NULL,
    tag TEXT NOT NULL,
    tag_type TEXT NOT NULL CHECK(tag_type IN ('genre', 'theme', 'demographic', 'custom')),
    spoiler INTEGER NOT NULL DEFAULT 0,
    votes INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    UNIQUE(cid, tag, tag_type),
    FOREIGN KEY (cid) REFERENCES core_metadata(cid) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_tags_cid ON content_tags(cid);
CREATE INDEX IF NOT EXISTS idx_tags_tag ON content_tags(tag);
CREATE INDEX IF NOT EXISTS idx_tags_type ON content_tags(tag_type);
