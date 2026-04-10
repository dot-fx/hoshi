use chrono::Utc;
use serde_json::Value;
use sqlx::SqlitePool;
use tracing::{debug, instrument};

use crate::error::CoreResult;

pub struct ContentUnitRepository;

impl ContentUnitRepository {
    #[instrument(skip(pool, unit))]
    pub async fn upsert(pool: &SqlitePool, cid: &str, unit: &Value) -> CoreResult<()> {
        let unit_type     = unit.get("type").and_then(|v| v.as_str()).unwrap_or("episode");
        let unit_number   = unit.get("episode").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let title         = unit.get("title").and_then(|v| v.as_str());
        let description   = unit.get("description").and_then(|v| v.as_str());
        let released_at   = unit.get("date").and_then(|v| v.as_str());
        let thumbnail_url = unit.get("img").and_then(|v| v.as_str())
            .map(|img| format!("https://simkl.in/episodes/{}_m.jpg", img));
        let now = Utc::now().timestamp();

        debug!(cid = %cid, type = %unit_type, num = unit_number, "Upserting content unit (episode/chapter)");

        sqlx::query(
            r#"
            INSERT INTO content_units (
                cid, unit_number, type, title, description, thumbnail_url, released_at, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(cid, type, unit_number) DO UPDATE SET
                title         = excluded.title,
                description   = excluded.description,
                thumbnail_url = excluded.thumbnail_url,
                released_at   = excluded.released_at
            "#,
        )
            .bind(cid)
            .bind(unit_number)
            .bind(unit_type)
            .bind(title)
            .bind(description)
            .bind(thumbnail_url)
            .bind(released_at)
            .bind(now)
            .execute(pool)
            .await?;

        Ok(())
    }
}