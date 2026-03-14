use serde::{Deserialize, Serialize};

use crate::content::{ContentRepository, ContentType};
use crate::error::{CoreError, CoreResult};
use crate::progress::repository::ProgressRepo;
use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AnimeProgress {
    pub id: i64,
    pub user_id: i32,
    pub cid: String,
    pub episode: i32,
    pub timestamp_seconds: i32,
    pub episode_duration_seconds: Option<i32>,
    pub completed: bool,
    pub last_accessed: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChapterProgress {
    pub id: i64,
    pub user_id: i32,
    pub cid: String,
    pub chapter: i32,
    pub completed: bool,
    pub last_accessed: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAnimeProgressBody {
    pub cid: String,
    pub episode: i32,
    pub timestamp_seconds: i32,
    pub episode_duration_seconds: Option<i32>,
    pub completed: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateChapterProgressBody {
    pub cid: String,
    pub chapter: i32,
    pub completed: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinueItem {
    pub cid: String,
    pub content_type: String,
    pub title: String,
    pub cover_image: Option<String>,
    pub episode: Option<i32>,
    pub timestamp_seconds: Option<i32>,
    pub episode_duration_seconds: Option<i32>,
    pub chapter: Option<i32>,
    pub last_accessed: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinueWatchingResponse {
    pub items: Vec<ContinueItem>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressResponse {
    pub success: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentProgressResponse {
    pub cid: String,
    pub anime_progress: Vec<AnimeProgress>,
    pub chapter_progress: Vec<ChapterProgress>,
}

pub struct ProgressService;

impl ProgressService {
    pub async fn update_anime_progress(
        state: &AppState,
        user_id: i32,
        body: UpdateAnimeProgressBody,
    ) -> CoreResult<ProgressResponse> {
        let conn = state.db.connection();
        let conn_lock = conn
            .lock()
            .map_err(|_| CoreError::Internal("DB Lock error".into()))?;

        ProgressRepo::upsert_anime(&conn_lock, user_id, &body)?;
        Ok(ProgressResponse { success: true })
    }
    pub async fn update_chapter_progress(
        state: &AppState,
        user_id: i32,
        body: UpdateChapterProgressBody,
    ) -> CoreResult<ProgressResponse> {
        let conn = state.db.connection();
        let conn_lock = conn
            .lock()
            .map_err(|_| CoreError::Internal("DB Lock error".into()))?;

        ProgressRepo::upsert_chapter(&conn_lock, user_id, &body)?;
        Ok(ProgressResponse { success: true })
    }

    pub async fn get_continue_watching(
        state: &AppState,
        user_id: i32,
        limit: Option<i64>,
    ) -> CoreResult<ContinueWatchingResponse> {
        let limit = limit.unwrap_or(20);
        let conn = state.db.connection();
        let conn_lock = conn
            .lock()
            .map_err(|_| CoreError::Internal("DB Lock error".into()))?;

        let anime_rows = ProgressRepo::get_latest_anime_per_cid(&conn_lock, user_id, limit)?;
        let chapter_rows = ProgressRepo::get_latest_chapter_per_cid(&conn_lock, user_id, limit)?;

        let mut items: Vec<ContinueItem> = Vec::new();

        for row in anime_rows {
            let (title, cover_image) = Self::fetch_meta(&conn_lock, &row.cid);
            items.push(ContinueItem {
                cid: row.cid,
                content_type: "anime".into(),
                title,
                cover_image,
                episode: Some(row.episode),
                timestamp_seconds: Some(row.timestamp_seconds),
                episode_duration_seconds: row.episode_duration_seconds,
                chapter: None,
                last_accessed: row.last_accessed,
            });
        }

        for row in chapter_rows {
            if items.iter().any(|i| i.cid == row.cid) {
                continue;
            }
            let (title, cover_image) = Self::fetch_meta(&conn_lock, &row.cid);
            let content_type = Self::fetch_content_type(&conn_lock, &row.cid);
            items.push(ContinueItem {
                cid: row.cid,
                content_type,
                title,
                cover_image,
                episode: None,
                timestamp_seconds: None,
                episode_duration_seconds: None,
                chapter: Some(row.chapter),
                last_accessed: row.last_accessed,
            });
        }

        items.sort_by(|a, b| b.last_accessed.cmp(&a.last_accessed));
        items.truncate(limit as usize);

        Ok(ContinueWatchingResponse { items })
    }

    pub async fn get_content_progress(
        state: &AppState,
        user_id: i32,
        cid: String,
    ) -> CoreResult<ContentProgressResponse> {
        let conn = state.db.connection();
        let conn_lock = conn
            .lock()
            .map_err(|_| CoreError::Internal("DB Lock error".into()))?;

        let (anime_progress, chapter_progress) =
            ProgressRepo::get_progress_for_cid(&conn_lock, user_id, &cid)?;

        Ok(ContentProgressResponse {
            cid,
            anime_progress,
            chapter_progress,
        })
    }

    fn fetch_meta(conn: &rusqlite::Connection, cid: &str) -> (String, Option<String>) {
        ContentRepository::get_full_content(conn, cid)
            .ok()
            .flatten()
            .and_then(|f| {
                f.primary_metadata()
                    .map(|m| (m.title.clone(), m.cover_image.clone()))
            })
            .unwrap_or_else(|| ("Unknown".into(), None))
    }

    fn fetch_content_type(conn: &rusqlite::Connection, cid: &str) -> String {
        ContentRepository::get_content_by_cid(conn, cid)
            .ok()
            .flatten()
            .map(|c| match c.content_type {
                ContentType::Anime => "anime",
                ContentType::Manga => "manga",
                ContentType::Novel => "novel",
                ContentType::Booru => "novel",
            })
            .unwrap_or("unknown")
            .to_string()
    }
}