use rusqlite::{params, Connection, OptionalExtension};
use serde_json::Value;

use crate::error::CoreResult;
use super::models::{ContentRelation, ContentUnit, RelationType};

// ── CacheRepository ───────────────────────────────────────────────────────────

pub struct CacheRepository;

impl CacheRepository {
    pub fn set(
        conn: &Connection,
        key: &str,
        source: &str,
        query_type: &str,
        data: &Value,
        ttl_seconds: i64,
    ) -> CoreResult<()> {
        let now = chrono::Utc::now().timestamp();
        let expires_at = now + ttl_seconds;
        conn.execute(
            r#"
            INSERT OR REPLACE INTO cache_metadata (key, source, query_type, data, created_at, expires_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![key, source, query_type, data.to_string(), now, expires_at],
        )?;
        Ok(())
    }

    pub fn get(conn: &Connection, key: &str) -> CoreResult<Option<Value>> {
        let now = chrono::Utc::now().timestamp();
        conn.query_row(
            "SELECT data FROM cache_metadata WHERE key = ?1 AND expires_at > ?2",
            params![key, now],
            |row| {
                let data_str: String = row.get(0)?;
                Ok(serde_json::from_str(&data_str).unwrap_or(Value::Null))
            },
        )
            .optional()
            .map_err(Into::into)
    }

    pub fn cleanup(conn: &Connection) -> CoreResult<()> {
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "DELETE FROM cache_metadata WHERE expires_at <= ?1",
            params![now],
        )?;
        Ok(())
    }
}

// ── RelationRepository ────────────────────────────────────────────────────────

pub struct RelationRepository;

impl RelationRepository {
    pub fn get_by_source(conn: &Connection, source_cid: &str) -> CoreResult<Vec<ContentRelation>> {
        let mut stmt = conn.prepare(
            "SELECT id, source_cid, target_cid, relation_type, source_name, created_at \
             FROM content_relations WHERE source_cid = ?1",
        )?;
        let rows = stmt.query_map(params![source_cid], |row| {
            Ok(ContentRelation {
                id:            Some(row.get(0)?),
                source_cid:    row.get(1)?,
                target_cid:    row.get(2)?,
                relation_type: serde_json::from_str(
                    &format!("\"{}\"", row.get::<_, String>(3)?)
                ).unwrap(),
                source_name:   row.get(4)?,
                created_at:    row.get(5)?,
            })
        })?;
        let mut results = Vec::new();
        for row in rows { results.push(row?); }
        Ok(results)
    }

    pub fn upsert(conn: &Connection, relation: &ContentRelation) -> CoreResult<()> {
        conn.execute(
            r#"
            INSERT INTO content_relations (source_cid, target_cid, relation_type, source_name, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(source_cid, target_cid, relation_type, source_name) DO NOTHING
            "#,
            params![
                relation.source_cid,
                relation.target_cid,
                relation.relation_type.as_str(),
                relation.source_name,
                relation.created_at,
            ],
        )?;
        Ok(())
    }
}

// ── UnitRepository ────────────────────────────────────────────────────────────

pub struct UnitRepository;

impl UnitRepository {
    pub fn upsert(conn: &Connection, unit: &ContentUnit) -> CoreResult<()> {
        conn.execute(
            r#"
            INSERT INTO content_units (
                cid, unit_number, type, title, description,
                thumbnail_url, released_at, duration, absolute_number, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            ON CONFLICT(cid, type, unit_number) DO UPDATE SET
                title         = excluded.title,
                description   = excluded.description,
                thumbnail_url = excluded.thumbnail_url,
                released_at   = excluded.released_at
            "#,
            params![
                unit.cid, unit.unit_number, unit.content_type, unit.title,
                unit.description, unit.thumbnail_url, unit.released_at,
                unit.duration, unit.absolute_number, unit.created_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_by_cid(conn: &Connection, cid: &str) -> CoreResult<Vec<ContentUnit>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM content_units WHERE cid = ?1 \
             ORDER BY CASE WHEN type = 'episode' THEN 1 ELSE 2 END, unit_number ASC",
        )?;
        let rows = stmt.query_map(params![cid], |row| {
            Ok(ContentUnit {
                id:            Some(row.get(0)?),
                cid:           row.get(1)?,
                unit_number:   row.get(2)?,
                content_type:  row.get(3)?,
                title:         row.get(4)?,
                description:   row.get(5)?,
                thumbnail_url: row.get(6)?,
                released_at:   row.get(7)?,
                duration:      row.get(8)?,
                absolute_number: row.get(9)?,
                created_at:    row.get(10)?,
            })
        })?;
        let mut units = Vec::new();
        for unit in rows { units.push(unit?); }
        Ok(units)
    }
}