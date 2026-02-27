use crate::error::CoreResult;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SavedImage {
    pub id: String,
    pub provider: String,
    pub title: String,
    pub artist: String,
    pub tags: String,
    pub original_link: String,
    pub local_path: Option<String>,
    pub created_at: i64,
}

pub struct BooruRepo;

impl BooruRepo {
    pub fn create_image(conn: &Connection, img: &SavedImage) -> CoreResult<()> {
        conn.execute(
            "INSERT OR IGNORE INTO saved_images
            (id, provider, title, artist, tags, original_link, local_path, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                img.id,
                img.provider,
                img.title,
                img.artist,
                img.tags,
                img.original_link,
                img.local_path,
                img.created_at
            ],
        )?;
        Ok(())
    }

    pub fn get_image(conn: &Connection, id: &str) -> CoreResult<Option<SavedImage>> {
        conn.query_row(
            "SELECT * FROM saved_images WHERE id = ?",
            [id],
            |row| {
                Ok(SavedImage {
                    id: row.get("id")?,
                    provider: row.get("provider")?,
                    title: row.get("title")?,
                    artist: row.get("artist")?,
                    tags: row.get("tags")?,
                    original_link: row.get("original_link")?,
                    local_path: row.get("local_path")?,
                    created_at: row.get("created_at")?,
                })
            },
        )
            .optional()
            .map_err(Into::into)
    }

    pub fn delete_image(conn: &Connection, id: &str) -> CoreResult<()> {
        conn.execute("DELETE FROM saved_images WHERE id = ?", [id])?;
        Ok(())
    }

    pub fn get_collection_images(
        conn: &Connection,
        collection_id: &str,
    ) -> CoreResult<Vec<SavedImage>> {
        let mut stmt = conn.prepare(
            "SELECT si.*
             FROM saved_images si
             JOIN collection_images ci ON si.id = ci.image_id
             WHERE ci.collection_id = ?
             ORDER BY ci.position ASC",
        )?;

        let rows = stmt.query_map([collection_id], |row| {
            Ok(SavedImage {
                id: row.get("id")?,
                provider: row.get("provider")?,
                title: row.get("title")?,
                artist: row.get("artist")?,
                tags: row.get("tags")?,
                original_link: row.get("original_link")?,
                local_path: row.get("local_path")?,
                created_at: row.get("created_at")?,
            })
        })?;

        Ok(rows.filter_map(Result::ok).collect())
    }
}