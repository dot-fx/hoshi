use rusqlite::{params, Connection, OptionalExtension};
use serde_json::Value;
use tracing::{debug, instrument};

use crate::error::CoreResult;
use super::models::{ExtensionSource};

pub struct ExtensionRepository;

impl ExtensionRepository {
    #[instrument(skip(conn, source))]
    pub fn add_source(conn: &Connection, source: &ExtensionSource) -> CoreResult<i64> {
        let now = chrono::Utc::now().timestamp();
        debug!(cid = %source.cid, ext = %source.extension_name, "Adding or updating extension source mapping");

        conn.execute(
            r#"
            INSERT INTO extension_sources
            (cid, extension_name, extension_id, nsfw, language, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(extension_name, extension_id) DO UPDATE SET
                nsfw       = excluded.nsfw,
                language   = excluded.language,
                updated_at = excluded.updated_at
            "#,
            params![
                source.cid, source.extension_name, source.extension_id,
                if source.nsfw { 1 } else { 0 }, source.language, now, now,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    #[instrument(skip(conn))]
    pub fn update_source(conn: &Connection, id: i64, ext_id: &str) -> CoreResult<()> {
        let now = chrono::Utc::now().timestamp();
        debug!(mapping_id = id, new_ext_id = %ext_id, "Updating extension ID in existing mapping");

        conn.execute(
            "UPDATE extension_sources SET extension_id = ?1, updated_at = ?2 WHERE id = ?3",
            params![ext_id, now, id],
        )?;
        Ok(())
    }

    pub fn find_cid_by_extension(
        conn: &Connection,
        extension_name: &str,
        extension_id: &str,
    ) -> CoreResult<Option<String>> {
        conn.query_row(
            "SELECT cid FROM extension_sources WHERE extension_name = ?1 AND extension_id = ?2",
            params![extension_name, extension_id],
            |row| row.get(0),
        )
            .optional()
            .map_err(Into::into)
    }

    #[instrument(skip(conn))]
    pub fn get_by_cid(conn: &Connection, cid: &str) -> CoreResult<Vec<ExtensionSource>> {
        debug!(cid = %cid, "Fetching all extension sources for content");
        let mut stmt = conn.prepare(
            "SELECT id, cid, extension_name, extension_id, nsfw, language, created_at, updated_at \
             FROM extension_sources WHERE cid = ?1",
        )?;

        let rows = stmt.query_map(params![cid], |row| {
            Ok(ExtensionSource {
                id:             Some(row.get(0)?),
                cid:            row.get(1)?,
                extension_name: row.get(2)?,
                extension_id:   row.get(3)?,
                nsfw:           row.get::<_, i32>(4)? == 1,
                language:       row.get(5)?,
                created_at:     row.get(6)?,
                updated_at:     row.get(7)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows { results.push(row?); }
        Ok(results)
    }

    pub fn find_mapping_id(conn: &Connection, cid: &str, ext_name: &str) -> CoreResult<Option<i64>> {
        conn.query_row(
            "SELECT id FROM extension_sources WHERE cid = ?1 AND extension_name = ?2",
            params![cid, ext_name],
            |row| row.get(0),
        )
            .optional()
            .map_err(Into::into)
    }

    pub fn get_extension_id_and_type(
        conn: &Connection,
        cid: &str,
        ext_name: &str,
    ) -> CoreResult<Option<(String, String)>> {
        conn.query_row(
            r#"
            SELECT c.type, es.extension_id
            FROM content c
            JOIN extension_sources es ON c.cid = es.cid
            WHERE c.cid = ?1 AND es.extension_name = ?2
            "#,
            params![cid, ext_name],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
            .optional()
            .map_err(Into::into)
    }
}

pub struct ContentUnitRepository;

impl ContentUnitRepository {
    #[instrument(skip(conn, unit))]
    pub fn upsert(conn: &Connection, cid: &str, unit: &Value) -> CoreResult<()> { // ✅ Cambiado a CoreResult
        let unit_type    = unit.get("type").and_then(|v| v.as_str()).unwrap_or("episode");
        let unit_number  = unit.get("episode").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let title        = unit.get("title").and_then(|v| v.as_str());
        let description  = unit.get("description").and_then(|v| v.as_str());
        let released_at  = unit.get("date").and_then(|v| v.as_str());
        let thumbnail_url = unit.get("img").and_then(|v| v.as_str())
            .map(|img| format!("https://simkl.in/episodes/{}_m.jpg", img));
        let now = chrono::Utc::now().timestamp();

        debug!(cid = %cid, type = %unit_type, num = unit_number, "Upserting content unit (episode/chapter)");

        conn.execute(
            "INSERT INTO content_units (\
                cid, unit_number, type, title, description, thumbnail_url, released_at, created_at\
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(cid, type, unit_number) DO UPDATE SET
                title         = excluded.title,
                description   = excluded.description,
                thumbnail_url = excluded.thumbnail_url,
                released_at   = excluded.released_at",
            rusqlite::params![cid, unit_number, unit_type, title, description, thumbnail_url, released_at, now],
        )?;
        Ok(())
    }
}