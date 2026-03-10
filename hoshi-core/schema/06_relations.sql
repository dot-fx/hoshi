CREATE TABLE IF NOT EXISTS content_relations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_cid TEXT NOT NULL,
    target_cid TEXT NOT NULL,
    relation_type TEXT NOT NULL CHECK(relation_type IN (
        'sequel', 'prequel', 'side_story', 'spinoff',
        'adaptation', 'alternative', 'parent', 'summary'
    )),
    source_name TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    UNIQUE(source_cid, target_cid, relation_type, source_name),
    FOREIGN KEY (source_cid) REFERENCES content(cid) ON DELETE CASCADE,
    FOREIGN KEY (target_cid) REFERENCES content(cid) ON DELETE CASCADE
    );

CREATE INDEX IF NOT EXISTS idx_rel_source ON content_relations(source_cid);
CREATE INDEX IF NOT EXISTS idx_rel_target ON content_relations(target_cid);
