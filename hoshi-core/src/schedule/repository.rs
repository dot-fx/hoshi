use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};

use crate::error::CoreResult;
use crate::tracker::provider::TrackerMedia;


#[derive(Debug, Clone)]
pub struct AiringEpisode {
    pub episode:   i32,
    pub airing_at: i64,
    pub media:     Option<TrackerMedia>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiringEntry {
    pub id:         i64,
    pub cid:        String,
    pub episode:    i32,
    pub airing_at:  i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiringEntryEnriched {
    pub id:         i64,
    pub cid:        String,
    pub episode:    i32,
    pub airing_at:  i64,
    pub title:         String,
    pub subtype:       Option<String>,
    pub cover_image:   Option<String>,
    pub banner_image:  Option<String>,
    pub synopsis:      Option<String>,
    #[serde(default)]
    pub title_i18n: std::collections::HashMap<String, String>,
    pub status:        Option<String>,
    pub genres:        Vec<String>,
    pub tags:          Vec<String>,
    pub nsfw:          bool,
    pub rating:        Option<f32>,
    pub release_date:  Option<String>,
    pub end_date:      Option<String>,
    pub trailer_url:   Option<String>,
    pub studio:        Option<String>,
    pub user_status:   Option<String>,
    pub user_progress: Option<i32>,
    pub user_score:    Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleWindow {
    #[serde(default = "default_days_back")]
    pub days_back: i64,
    #[serde(default = "default_days_ahead")]
    pub days_ahead: i64,
}

fn default_days_back()  -> i64 { 1 }
fn default_days_ahead() -> i64 { 7 }

pub struct ScheduleRepository;

impl ScheduleRepository {
    #[instrument(skip(conn))]
    pub fn upsert(conn: &Connection, cid: &str, episode: i32, airing_at: i64) -> CoreResult<()> {
        let now = Utc::now().timestamp();
        debug!(cid = %cid, episode = episode, "Upserting airing schedule entry");

        conn.execute(
            r#"
            INSERT INTO airing_schedule (cid, episode, airing_at, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?4)
            ON CONFLICT(cid, episode) DO UPDATE SET
                airing_at  = excluded.airing_at,
                updated_at = excluded.updated_at
            "#,
            params![cid, episode, airing_at, now],
        )?;
        Ok(())
    }

    #[instrument(skip(conn, cids))]
    pub fn get_by_cids_in_window(
        conn: &Connection,
        cids: &[String],
        from_ts: i64,
        to_ts: i64,
    ) -> CoreResult<Vec<AiringEntry>> {
        if cids.is_empty() {
            return Ok(vec![]);
        }

        debug!(count = cids.len(), "Fetching schedule for multiple CIDs in time window");

        let placeholders = cids
            .iter()
            .enumerate()
            .map(|(i, _)| format!("?{}", i + 3))
            .collect::<Vec<_>>()
            .join(", ");

        let sql = format!(
            r#"
            SELECT id, cid, episode, airing_at, created_at, updated_at
            FROM airing_schedule
            WHERE airing_at BETWEEN ?1 AND ?2
              AND cid IN ({})
            ORDER BY airing_at ASC
            "#,
            placeholders
        );

        let mut stmt = conn.prepare(&sql)?;

        let mut param_values: Vec<Box<dyn rusqlite::ToSql>> = vec![
            Box::new(from_ts),
            Box::new(to_ts),
        ];
        for cid in cids {
            param_values.push(Box::new(cid.clone()));
        }

        let rows = stmt.query_map(
            rusqlite::params_from_iter(param_values.iter().map(|p| p.as_ref())),
            |row| {
                Ok(AiringEntry {
                    id:         row.get(0)?,
                    cid:        row.get(1)?,
                    episode:    row.get(2)?,
                    airing_at:  row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            },
        )?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn has_any(conn: &Connection, cid: &str) -> CoreResult<bool> {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM airing_schedule WHERE cid = ?1",
            params![cid],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }
}