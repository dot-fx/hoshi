use crate::error::CoreResult;
use rusqlite::{params, Connection, OptionalExtension};
use crate::list::types::{ListEntry, ScoreDistribution, UpsertEntryBody, UserStats};

pub struct ListRepo;

impl ListRepo {
    pub fn get_entry(conn: &Connection, user_id: i32, cid: &str) -> CoreResult<Option<ListEntry>> {
        let entry = conn.query_row(
            "SELECT * FROM ListEntry WHERE user_id = ? AND cid = ?",
            params![user_id, cid],
            Self::map_row
        ).optional()?;
        Ok(entry)
    }

    pub fn get_entries(
        conn: &Connection,
        user_id: i32,
        status: Option<&str>
    ) -> CoreResult<Vec<ListEntry>> {
        let mut sql = "SELECT * FROM ListEntry WHERE user_id = ?".to_string();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(user_id)];

        if let Some(s) = status {
            sql.push_str(" AND status = ?");
            params.push(Box::new(s.to_string()));
        }

        sql.push_str(" ORDER BY updated_at DESC");

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(
            rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
            Self::map_row,
        )?;

        let mut entries = Vec::new();
        for r in rows {
            entries.push(r?);
        }
        Ok(entries)
    }

    pub fn upsert_entry(conn: &Connection, user_id: i32, body: &UpsertEntryBody, final_status: &str, new_progress: i32, start_date: Option<String>, end_date: Option<String>) -> CoreResult<usize> {
        let changes = conn.execute(
            r#"
            INSERT INTO ListEntry (user_id, cid, status, progress, score, start_date, end_date, repeat_count, notes, is_private, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, CURRENT_TIMESTAMP)
            ON CONFLICT(user_id, cid) DO UPDATE SET
                status = excluded.status,
                progress = excluded.progress,
                score = excluded.score,
                start_date = excluded.start_date,
                end_date = excluded.end_date,
                repeat_count = excluded.repeat_count,
                notes = excluded.notes,
                is_private = excluded.is_private,
                updated_at = CURRENT_TIMESTAMP
            "#,
            params![
                user_id,
                body.cid,
                final_status,
                new_progress,
                body.score,
                start_date,
                end_date,
                body.repeat_count.unwrap_or(0),
                body.notes,
                body.is_private.unwrap_or(false),
            ],
        )?;
        Ok(changes)
    }

    pub fn delete_entry(conn: &Connection, user_id: i32, cid: &str) -> CoreResult<bool> {
        let rows = conn.execute(
            "DELETE FROM ListEntry WHERE user_id = ? AND cid = ?",
            params![user_id, cid],
        )?;
        Ok(rows > 0)
    }

    pub fn get_user_stats(conn: &Connection, user_id: i32) -> CoreResult<UserStats> {
        let total_entries: i32 = conn
            .query_row("SELECT COUNT(*) FROM ListEntry WHERE user_id = ?", [user_id], |r| r.get(0))
            .unwrap_or(0);
        let watching: i32 = conn
            .query_row("SELECT COUNT(*) FROM ListEntry WHERE user_id = ? AND status = 'CURRENT'", [user_id], |r| r.get(0))
            .unwrap_or(0);
        let completed: i32 = conn
            .query_row("SELECT COUNT(*) FROM ListEntry WHERE user_id = ? AND status = 'COMPLETED'", [user_id], |r| r.get(0))
            .unwrap_or(0);
        let planning: i32 = conn
            .query_row("SELECT COUNT(*) FROM ListEntry WHERE user_id = ? AND status = 'PLANNING'", [user_id], |r| r.get(0))
            .unwrap_or(0);
        let paused: i32 = conn
            .query_row("SELECT COUNT(*) FROM ListEntry WHERE user_id = ? AND status = 'PAUSED'", [user_id], |r| r.get(0))
            .unwrap_or(0);
        let dropped: i32 = conn
            .query_row("SELECT COUNT(*) FROM ListEntry WHERE user_id = ? AND status = 'DROPPED'", [user_id], |r| r.get(0))
            .unwrap_or(0);
        let repeating: i32 = conn
            .query_row("SELECT COUNT(*) FROM ListEntry WHERE user_id = ? AND status = 'REPEATING'", [user_id], |r| r.get(0))
            .unwrap_or(0);

        let mean_score: Option<f64> = conn
            .query_row(
                "SELECT AVG(score) FROM ListEntry WHERE user_id = ? AND score IS NOT NULL",
                [user_id],
                |r| r.get(0),
            )
            .ok()
            .flatten();

        let score_distribution = {
            let mut stmt = conn.prepare(
                "SELECT CAST(ROUND(score) AS INTEGER) AS bucket, COUNT(*) AS cnt
                 FROM ListEntry
                 WHERE user_id = ? AND score IS NOT NULL AND score >= 1 AND score <= 10
                 GROUP BY bucket
                 ORDER BY bucket ASC",
            )?;
            let rows = stmt.query_map([user_id], |r| {
                Ok(ScoreDistribution {
                    score: r.get(0)?,
                    count: r.get(1)?,
                })
            })?;
            let mut dist = Vec::new();
            for r in rows {
                dist.push(r?);
            }
            dist
        };

        let days_since_last_activity: Option<i64> = conn
            .query_row(
                "SELECT CAST(julianday('now') - julianday(MAX(updated_at)) AS INTEGER)
                 FROM ListEntry WHERE user_id = ?",
                [user_id],
                |r| r.get(0),
            )
            .ok()
            .flatten();

        let started = total_entries - planning;
        let completion_rate = if started > 0 {
            Some(completed as f64 / started as f64)
        } else {
            None
        };

        let total_rewatches: i32 = conn
            .query_row(
                "SELECT COALESCE(SUM(repeat_count), 0) FROM ListEntry WHERE user_id = ?",
                [user_id],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let entries_with_notes: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM ListEntry WHERE user_id = ? AND notes IS NOT NULL AND notes != ''",
                [user_id],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let private_entries: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM ListEntry WHERE user_id = ? AND is_private = 1",
                [user_id],
                |r| r.get(0),
            )
            .unwrap_or(0);

        Ok(UserStats {
            total_entries,
            watching,
            completed,
            planning,
            paused,
            dropped,
            repeating,
            total_episodes: 0,
            total_chapters: 0,
            mean_score,
            score_distribution,
            days_since_last_activity,
            completion_rate,
            total_rewatches,
            entries_with_notes,
            private_entries,
        })
    }

    pub fn get_completed_entries_progress(conn: &Connection, user_id: i32) -> CoreResult<Vec<(String, i32)>> {
        let mut stmt = conn.prepare(
            "SELECT cid, progress FROM ListEntry WHERE user_id = ? AND status = 'COMPLETED'"
        )?;
        let rows = stmt.query_map([user_id], |row| Ok((row.get(0)?, row.get(1)?)))?;

        let mut result = Vec::new();
        for r in rows { result.push(r?); }
        Ok(result)
    }

    fn map_row(row: &rusqlite::Row) -> rusqlite::Result<ListEntry> {
        Ok(ListEntry {
            id: row.get("id").ok(),
            user_id: row.get("user_id")?,
            cid: row.get("cid")?,
            status: row.get("status")?,
            progress: row.get("progress")?,
            score: row.get("score").ok(),
            start_date: row.get("start_date").ok(),
            end_date: row.get("end_date").ok(),
            repeat_count: row.get("repeat_count")?,
            notes: row.get("notes").ok(),
            is_private: row.get::<_, i32>("is_private")? == 1,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}