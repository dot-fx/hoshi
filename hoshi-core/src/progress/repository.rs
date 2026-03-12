use crate::error::{CoreError, CoreResult};
use crate::progress::service::{
    AnimeProgress, ChapterProgress, UpdateAnimeProgressBody, UpdateChapterProgressBody,
};

pub struct ProgressRepo;

impl ProgressRepo {
    pub fn upsert_anime(
        conn: &rusqlite::Connection,
        user_id: i32,
        body: &UpdateAnimeProgressBody,
    ) -> CoreResult<()> {
        conn.execute(
            r#"
            INSERT INTO AnimeProgress
                (user_id, cid, episode, timestamp_seconds, episode_duration_seconds, completed, last_accessed)
            VALUES
                (?1, ?2, ?3, ?4, ?5, ?6, unixepoch())
            ON CONFLICT(user_id, cid, episode) DO UPDATE SET
                timestamp_seconds        = excluded.timestamp_seconds,
                episode_duration_seconds = excluded.episode_duration_seconds,
                completed                = excluded.completed,
                last_accessed            = unixepoch()
            "#,
            rusqlite::params![
                user_id,
                body.cid,
                body.episode,
                body.timestamp_seconds,
                body.episode_duration_seconds,
                body.completed.unwrap_or(false) as i32,
            ],
        )
            .map_err(|e| CoreError::Internal(e.to_string()))?;
        Ok(())
    }

    pub fn upsert_chapter(
        conn: &rusqlite::Connection,
        user_id: i32,
        body: &UpdateChapterProgressBody,
    ) -> CoreResult<()> {
        conn.execute(
            r#"
            INSERT INTO ChapterProgress
                (user_id, cid, chapter, completed, last_accessed)
            VALUES
                (?1, ?2, ?3, ?4, unixepoch())
            ON CONFLICT(user_id, cid, chapter) DO UPDATE SET
                completed     = excluded.completed,
                last_accessed = unixepoch()
            "#,
            rusqlite::params![
                user_id,
                body.cid,
                body.chapter,
                body.completed.unwrap_or(false) as i32,
            ],
        )
            .map_err(|e| CoreError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Último episodio no completado por cid (para el home)
    pub fn get_latest_anime_per_cid(
        conn: &rusqlite::Connection,
        user_id: i32,
        limit: i64,
    ) -> CoreResult<Vec<AnimeProgress>> {
        let mut stmt = conn
            .prepare(
                r#"
                SELECT ap.*
                FROM AnimeProgress ap
                INNER JOIN (
                    SELECT cid, MAX(last_accessed) AS max_accessed
                    FROM AnimeProgress
                    WHERE user_id = ?1 AND completed = 0
                    GROUP BY cid
                ) latest ON ap.cid = latest.cid AND ap.last_accessed = latest.max_accessed
                WHERE ap.user_id = ?1
                ORDER BY ap.last_accessed DESC
                LIMIT ?2
                "#,
            )
            .map_err(|e| CoreError::Internal(e.to_string()))?;

        let rows: Vec<AnimeProgress> = stmt
            .query_map(rusqlite::params![user_id, limit], Self::map_anime_row)
            .map_err(|e| CoreError::Internal(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| CoreError::Internal(e.to_string()))?;

        Ok(rows)
    }

    /// Último capítulo no completado por cid (para el home)
    pub fn get_latest_chapter_per_cid(
        conn: &rusqlite::Connection,
        user_id: i32,
        limit: i64,
    ) -> CoreResult<Vec<ChapterProgress>> {
        let mut stmt = conn
            .prepare(
                r#"
                SELECT cp.*
                FROM ChapterProgress cp
                INNER JOIN (
                    SELECT cid, MAX(last_accessed) AS max_accessed
                    FROM ChapterProgress
                    WHERE user_id = ?1 AND completed = 0
                    GROUP BY cid
                ) latest ON cp.cid = latest.cid AND cp.last_accessed = latest.max_accessed
                WHERE cp.user_id = ?1
                ORDER BY cp.last_accessed DESC
                LIMIT ?2
                "#,
            )
            .map_err(|e| CoreError::Internal(e.to_string()))?;

        let rows: Vec<ChapterProgress> = stmt
            .query_map(rusqlite::params![user_id, limit], Self::map_chapter_row)
            .map_err(|e| CoreError::Internal(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| CoreError::Internal(e.to_string()))?;

        Ok(rows)
    }

    /// Todo el progreso de un cid concreto (para el player/reader al abrirlo)
    pub fn get_progress_for_cid(
        conn: &rusqlite::Connection,
        user_id: i32,
        cid: &str,
    ) -> CoreResult<(Vec<AnimeProgress>, Vec<ChapterProgress>)> {
        let mut anime_stmt = conn
            .prepare(
                "SELECT * FROM AnimeProgress WHERE user_id = ?1 AND cid = ?2 ORDER BY episode ASC",
            )
            .map_err(|e| CoreError::Internal(e.to_string()))?;

        let anime: Vec<AnimeProgress> = anime_stmt
            .query_map(rusqlite::params![user_id, cid], Self::map_anime_row)
            .map_err(|e| CoreError::Internal(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| CoreError::Internal(e.to_string()))?;

        drop(anime_stmt);

        let mut chapter_stmt = conn
            .prepare(
                "SELECT * FROM ChapterProgress WHERE user_id = ?1 AND cid = ?2 ORDER BY chapter ASC",
            )
            .map_err(|e| CoreError::Internal(e.to_string()))?;

        let chapters: Vec<ChapterProgress> = chapter_stmt
            .query_map(rusqlite::params![user_id, cid], Self::map_chapter_row)
            .map_err(|e| CoreError::Internal(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| CoreError::Internal(e.to_string()))?;

        drop(chapter_stmt);

        Ok((anime, chapters))
    }

    // ── Mappers ───────────────────────────────────────────────────────────────

    fn map_anime_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<AnimeProgress> {
        Ok(AnimeProgress {
            id: row.get(0)?,
            user_id: row.get(1)?,
            cid: row.get(2)?,
            episode: row.get(3)?,
            timestamp_seconds: row.get(4)?,
            episode_duration_seconds: row.get(5)?,
            completed: row.get::<_, i32>(6)? != 0,
            last_accessed: row.get(7)?,
        })
    }

    fn map_chapter_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ChapterProgress> {
        Ok(ChapterProgress {
            id: row.get(0)?,
            user_id: row.get(1)?,
            cid: row.get(2)?,
            chapter: row.get(3)?,
            completed: row.get::<_, i32>(4)? != 0,
            last_accessed: row.get(5)?,
        })
    }
}