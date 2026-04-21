use chrono::Utc;
use sqlx::SqlitePool;
use tracing::{debug, instrument};

use crate::error::CoreResult;
use crate::schedule::types::AiringEntry;

pub struct ScheduleRepository;

impl ScheduleRepository {
    #[instrument(skip(pool))]
    pub async fn upsert(pool: &SqlitePool, cid: &str, episode: i32, airing_at: i64) -> CoreResult<()> {
        let now = Utc::now().timestamp();
        debug!(cid = %cid, episode = episode, "Upserting airing schedule entry");

        sqlx::query(
            r#"
            INSERT INTO airing_schedule (cid, episode, airing_at, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(cid, episode) DO UPDATE SET
                airing_at  = excluded.airing_at,
                updated_at = excluded.updated_at
            "#,
        )
            .bind(cid)
            .bind(episode)
            .bind(airing_at)
            .bind(now)
            .bind(now)
            .execute(pool)
            .await?;

        Ok(())
    }

    #[instrument(skip(pool, cids))]
    pub async fn get_by_cids_in_window(
        pool: &SqlitePool,
        cids: &[String],
        from_ts: i64,
        to_ts: i64,
    ) -> CoreResult<Vec<AiringEntry>> {
        if cids.is_empty() {
            return Ok(vec![]);
        }

        debug!(count = cids.len(), "Fetching schedule for multiple CIDs in time window");

        let placeholders = cids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let sql = format!(
            r#"
            SELECT id, cid, episode, airing_at, created_at, updated_at
            FROM airing_schedule
            WHERE airing_at BETWEEN ? AND ?
              AND cid IN ({})
            ORDER BY airing_at ASC
            "#,
            placeholders
        );

        let mut query = sqlx::query_as(&sql).bind(from_ts).bind(to_ts);
        for cid in cids {
            query = query.bind(cid);
        }

        let rows = query.fetch_all(pool).await?;
        Ok(rows)
    }

    pub async fn has_any(pool: &SqlitePool, cid: &str) -> CoreResult<bool> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM airing_schedule WHERE cid = ?",
        )
            .bind(cid)
            .fetch_one(pool)
            .await?;

        Ok(row.0 > 0)
    }

    #[instrument(skip(pool, cids))]
    pub async fn get_content_by_cids(
        pool: &SqlitePool,
        cids: &[String],
    ) -> CoreResult<Vec<AiringEntry>> {
        if cids.is_empty() {
            return Ok(vec![]);
        }
        let placeholders = cids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let sql = format!(
            "SELECT id, cid, episode, airing_at, created_at, updated_at FROM airing_schedule WHERE cid IN ({})",
            placeholders
        );
        let mut query = sqlx::query_as(&sql);
        for cid in cids {
            query = query.bind(cid);
        }
        Ok(query.fetch_all(pool).await?)
    }
}