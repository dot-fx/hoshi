use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};

use crate::content::{ContentRepository, ContentType, ContentUnit};
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
    #[serde(default)]
    pub title_i18n: HashMap<String, String>,
    pub cover_image: Option<String>,
    pub nsfw: bool,
    pub episode: Option<i32>,
    pub timestamp_seconds: Option<i32>,
    pub episode_duration_seconds: Option<i32>,
    pub chapter: Option<i32>,
    pub unit: Option<ContentUnit>,
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
    #[instrument(skip(state))]
    pub async fn update_anime_progress(
        state: &AppState,
        user_id: i32,
        body: UpdateAnimeProgressBody,
    ) -> CoreResult<ProgressResponse> {
        let conn = state.db.connection();
        let conn_lock = conn
            .lock()
            .map_err(|_| CoreError::Internal("error.system.db_lock".into()))?;

        ProgressRepo::upsert_anime(&conn_lock, user_id, &body)?;

        debug!(cid = %body.cid, episode = body.episode, "Anime progress saved successfully");

        Ok(ProgressResponse { success: true })
    }

    #[instrument(skip(state))]
    pub async fn update_chapter_progress(
        state: &AppState,
        user_id: i32,
        body: UpdateChapterProgressBody,
    ) -> CoreResult<ProgressResponse> {
        let conn = state.db.connection();
        let conn_lock = conn
            .lock()
            .map_err(|_| CoreError::Internal("error.system.db_lock".into()))?;

        ProgressRepo::upsert_chapter(&conn_lock, user_id, &body)?;

        debug!(cid = %body.cid, chapter = body.chapter, "Chapter progress saved successfully");

        Ok(ProgressResponse { success: true })
    }

    #[instrument(skip(state))]
    pub async fn get_continue_watching(
        state: &AppState,
        user_id: i32,
        limit: Option<i64>,
    ) -> CoreResult<ContinueWatchingResponse> {
        let limit = limit.unwrap_or(20);
        let conn = state.db.connection();
        let conn_lock = conn
            .lock()
            .map_err(|_| CoreError::Internal("error.system.db_lock".into()))?;

        let anime_rows = ProgressRepo::get_latest_anime_per_cid(&conn_lock, user_id, limit)?;
        let chapter_rows = ProgressRepo::get_latest_chapter_per_cid(&conn_lock, user_id, limit)?;

        let mut items: Vec<ContinueItem> = Vec::new();

        for row in anime_rows {
            let (title, cover_image, title_i18n, nsfw, units) = Self::fetch_enriched_data(&conn_lock, &row.cid);

            let unit = units.into_iter().find(|u| u.unit_number == row.episode as f64);

            items.push(ContinueItem {
                cid: row.cid,
                content_type: "anime".into(),
                title,
                title_i18n,
                cover_image,
                nsfw,
                episode: Some(row.episode),
                timestamp_seconds: Some(row.timestamp_seconds),
                episode_duration_seconds: row.episode_duration_seconds,
                chapter: None,
                unit,
                last_accessed: row.last_accessed,
            });
        }

        for row in chapter_rows {
            if items.iter().any(|i| i.cid == row.cid) {
                continue;
            }
            let (title, cover_image, title_i18n, nsfw, units) = Self::fetch_enriched_data(&conn_lock, &row.cid);
            let content_type = Self::fetch_content_type(&conn_lock, &row.cid);

            let unit = units.into_iter().find(|u| u.unit_number == row.chapter as f64);

            items.push(ContinueItem {
                cid: row.cid,
                content_type,
                title,
                title_i18n,
                cover_image,
                nsfw,
                episode: None,
                timestamp_seconds: None,
                episode_duration_seconds: None,
                chapter: Some(row.chapter),
                unit,
                last_accessed: row.last_accessed,
            });
        }

        items.sort_by(|a, b| b.last_accessed.cmp(&a.last_accessed));
        items.truncate(limit as usize);

        debug!(items_returned = items.len(), "Continue watching list generated");

        Ok(ContinueWatchingResponse { items })
    }

    fn fetch_enriched_data(
        conn: &rusqlite::Connection,
        cid: &str
    ) -> (String, Option<String>, HashMap<String, String>, bool, Vec<ContentUnit>) {
        ContentRepository::get_full_content(conn, cid)
            .ok()
            .flatten()
            .map(|f| {
                let primary = f.primary_metadata();

                let title = primary.map(|m| m.title.clone()).unwrap_or_else(|| "Unknown".into());
                let cover_image = primary.and_then(|m| m.cover_image.clone());
                let title_i18n = primary.map(|m| m.title_i18n.clone()).unwrap_or_default();

                (title, cover_image, title_i18n, f.content.nsfw, f.content_units)
            })
            .unwrap_or_else(|| ("Unknown".into(), None, HashMap::new(), false, vec![]))
    }

    #[instrument(skip(state))]
    pub async fn get_content_progress(
        state: &AppState,
        user_id: i32,
        cid: String,
    ) -> CoreResult<ContentProgressResponse> {
        let conn = state.db.connection();
        let conn_lock = conn
            .lock()
            .map_err(|_| CoreError::Internal("error.system.db_lock".into()))?;
        
        let (anime_progress, chapter_progress) =
            ProgressRepo::get_progress_for_cid(&conn_lock, user_id, &cid)?;

        Ok(ContentProgressResponse {
            cid,
            anime_progress,
            chapter_progress,
        })
    }

    fn fetch_content_type(conn: &rusqlite::Connection, cid: &str) -> String {
        ContentRepository::get_content_by_cid(conn, cid)
            .ok()
            .flatten()
            .map(|c| match c.content_type {
                ContentType::Anime => "anime",
                ContentType::Manga => "manga",
                ContentType::Novel => "novel",
            })
            .unwrap_or("unknown")
            .to_string()
    }
}