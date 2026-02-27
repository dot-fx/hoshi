CREATE TABLE IF NOT EXISTS saved_images (
    id TEXT PRIMARY KEY,
    provider TEXT NOT NULL,
    title TEXT NOT NULL,
    artist TEXT NOT NULL,
    tags TEXT NOT NULL DEFAULT "",
    original_link TEXT NOT NULL,
    local_path TEXT,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS collections (
    id TEXT PRIMARY KEY,
    user_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    description TEXT DEFAULT "",
    is_private INTEGER NOT NULL DEFAULT 0,
    cover_id TEXT,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES User(id) ON DELETE CASCADE,
    FOREIGN KEY (cover_id) REFERENCES saved_images(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_collections_user
    ON collections(user_id);

    CREATE TABLE IF NOT EXISTS collection_images (
    collection_id TEXT NOT NULL,
    image_id TEXT NOT NULL,
    added_at INTEGER NOT NULL,
    position INTEGER DEFAULT 0,
    PRIMARY KEY (collection_id, image_id),
    FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE,
    FOREIGN KEY (image_id) REFERENCES saved_images(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_collection_position
    ON collection_images(collection_id, position);

CREATE INDEX IF NOT EXISTS idx_collection_order
    ON collection_images(collection_id, position);

CREATE INDEX IF NOT EXISTS idx_collection_images_collection
    ON collection_images(collection_id);

CREATE INDEX IF NOT EXISTS idx_collection_images_image
    ON collection_images(image_id);
