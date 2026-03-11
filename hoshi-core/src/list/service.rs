use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use chrono::Utc;
use futures::future::join_all;

use crate::error::{CoreError, CoreResult};
use crate::list::repository::ListRepo;
use crate::tracker::repository::{TrackerRepository, TrackerIntegration};
use crate::content::repository::{ContentRepository, EpisodeData};
use crate::tracker::provider::UpdateEntryParams;
use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListEntry {
    pub id: Option<i64>,
    pub user_id: i32,
    pub cid: String,
    pub status: String,
    pub progress: i32,
    pub score: Option<f64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub repeat_count: i32,
    pub notes: Option<String>,
    pub is_private: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnrichedListEntry {
    #[serde(flatten)]
    pub entry: ListEntry,
    pub title: String,
    pub cover_image: Option<String>,
    pub content_type: String,
    pub nsfw: bool,
    pub total_units: Option<i32>,
    pub tracker_ids: Value,
    pub external_ids: Value,
    pub has_extension_source: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserStats {
    pub total_entries: i32,
    pub watching: i32,
    pub completed: i32,
    pub planning: i32,
    pub paused: i32,
    pub dropped: i32,
    pub total_episodes: i32,
    pub total_chapters: i32,
    pub mean_score: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertEntryBody {
    pub cid: String,
    pub status: String,
    pub progress: Option<i32>,
    pub score: Option<f64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub repeat_count: Option<i32>,
    pub notes: Option<String>,
    pub is_private: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterQuery {
    pub status: Option<String>,
    pub content_type: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListResponse {
    pub results: Vec<EnrichedListEntry>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SingleEntryResponse {
    pub found: bool,
    pub entry: Option<EnrichedListEntry>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertEntryResponse {
    pub success: bool,
    pub changes: usize,
    pub is_new: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SuccessResponse {
    pub success: bool,
}

pub struct ListService;

impl ListService {
    pub async fn get_list(
        state: &AppState,
        user_id: i32,
        filter: FilterQuery,
    ) -> CoreResult<ListResponse> {
        let db = state.db.connection();

        let entries = {
            let conn = db.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;
            ListRepo::get_entries(&conn, user_id, filter.status.as_deref())?
        };

        let mut enriched = Self::enrich_entries(&db, entries).await?;

        if let Some(ct) = filter.content_type {
            enriched.retain(|e| e.content_type == ct.to_lowercase());
        }

        Ok(ListResponse { results: enriched })
    }

    pub async fn get_single_entry(
        state: &AppState,
        user_id: i32,
        cid: String,
    ) -> CoreResult<SingleEntryResponse> {
        let db = state.db.connection();

        let entry = {
            let conn = db.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;
            ListRepo::get_entry(&conn, user_id, &cid)?
        };

        if let Some(e) = entry {
            let enriched = Self::enrich_entries(&db, vec![e]).await?;
            Ok(SingleEntryResponse {
                found: true,
                entry: enriched.into_iter().next(),
            })
        } else {
            Ok(SingleEntryResponse { found: false, entry: None })
        }
    }

    pub async fn upsert_entry(
        state: Arc<AppState>,
        user_id: i32,
        body: UpsertEntryBody,
    ) -> CoreResult<UpsertEntryResponse> {
        let total_units = {
            let conn = state.db.connection();
            let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;

            ContentRepository::get_content_by_cid(&conn_lock, &body.cid)?
                .ok_or_else(|| CoreError::NotFound(format!("Content CID {} not found", body.cid)))?;

            ContentRepository::get_by_cid(&conn_lock, &body.cid)?
                .map(|m| match m.eps_or_chapters {
                    EpisodeData::Count(n) => n,
                    EpisodeData::List(l)  => l.len() as i32,
                })
        };

        let prev_entry = {
            let conn = state.db.connection();
            let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;
            ListRepo::get_entry(&conn_lock, user_id, &body.cid)?
        };

        let is_new        = prev_entry.is_none();
        let prev_progress = prev_entry.as_ref().map(|e| e.progress).unwrap_or(0);
        let new_progress  = body.progress.unwrap_or(prev_progress);

        if !is_new && body.progress.is_some() && new_progress < prev_progress {
            tracing::warn!("Ignoring progress downgrade for user {} on CID {}", user_id, body.cid);
            return Ok(UpsertEntryResponse { success: true, changes: 0, is_new: false });
        }

        let today = Utc::now().format("%Y-%m-%d").to_string();
        let mut final_start_date = body.start_date.clone();
        let mut final_end_date   = body.end_date.clone();
        let mut final_status     = body.status.clone();

        if let Some(ref prev) = prev_entry {
            if final_start_date.is_none() && prev.start_date.is_some() {
                final_start_date = prev.start_date.clone();
            }
        }

        if final_start_date.is_none() && new_progress == 1 {
            final_start_date = Some(today.clone());
        }

        if let Some(total) = total_units {
            if new_progress >= total && total > 0 {
                final_status = "COMPLETED".to_string();
                if final_end_date.is_none() {
                    final_end_date = Some(today);
                }
            }
        }

        let changes = {
            let conn = state.db.connection();
            let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;
            ListRepo::upsert_entry(
                &conn_lock,
                user_id,
                &body,
                &final_status,
                new_progress,
                final_start_date,
                final_end_date,
            )?
        };

        let cid_clone   = body.cid.clone();
        let state_clone = state.clone();
        tokio::task::spawn(async move {
            if let Err(e) = Self::sync_entry_to_all_trackers(&state_clone, user_id, &cid_clone).await {
                tracing::error!("Background sync failed for {}: {}", cid_clone, e);
            }
        });

        Ok(UpsertEntryResponse { success: true, changes, is_new })
    }

    pub async fn delete_entry(
        state: Arc<AppState>,
        user_id: i32,
        cid: String,
    ) -> CoreResult<SuccessResponse> {
        let state_clone = state.clone();
        let cid_clone   = cid.clone();
        tokio::task::spawn(async move {
            let _ = Self::delete_from_trackers(&state_clone, user_id, &cid_clone).await;
        });

        let conn      = state.db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;
        let deleted   = ListRepo::delete_entry(&conn_lock, user_id, &cid)?;

        if deleted {
            Ok(SuccessResponse { success: true })
        } else {
            Err(CoreError::NotFound("Entry not found".into()))
        }
    }

    async fn enrich_entries(
        db_meta: &Arc<std::sync::Mutex<rusqlite::Connection>>,
        entries: Vec<ListEntry>,
    ) -> CoreResult<Vec<EnrichedListEntry>> {
        let futures = entries.into_iter().map(|entry| {
            let db = db_meta.clone();
            async move {
                let full_content = {
                    let conn = db.lock().unwrap();
                    ContentRepository::get_full_content(&conn, &entry.cid).ok().flatten()
                };

                match full_content {
                    Some(full) => {
                        let content_type = full.content.content_type.as_str().to_string();
                        let nsfw         = full.content.nsfw;

                        let meta = full.primary_metadata();

                        let title       = meta.map(|m| m.title.clone()).unwrap_or_else(|| "Unknown".into());
                        let cover_image = meta.and_then(|m| m.cover_image.clone());
                        let external_ids = meta.map(|m| m.external_ids.clone()).unwrap_or(json!({}));

                        let total = meta.map(|m| match &m.eps_or_chapters {
                            EpisodeData::Count(n) => *n,
                            EpisodeData::List(l)  => l.len() as i32,
                        });

                        let mut tracker_ids = serde_json::Map::new();
                        for mapping in &full.tracker_mappings {
                            tracker_ids.insert(
                                mapping.tracker_name.clone(),
                                Value::String(mapping.tracker_id.clone()),
                            );
                        }

                        EnrichedListEntry {
                            entry,
                            title,
                            cover_image,
                            content_type,
                            nsfw,
                            total_units: total,
                            tracker_ids: Value::Object(tracker_ids),
                            external_ids,
                            has_extension_source: !full.extension_sources.is_empty(),
                        }
                    }
                    None => EnrichedListEntry {
                        entry,
                        title: "Unknown Content".into(),
                        cover_image: None,
                        content_type: "unknown".into(),
                        nsfw: false,
                        total_units: None,
                        tracker_ids: json!({}),
                        external_ids: json!({}),
                        has_extension_source: false,
                    },
                }
            }
        });

        Ok(join_all(futures).await)
    }

    async fn sync_entry_to_all_trackers(
        state: &Arc<AppState>,
        user_id: i32,
        cid: &str,
    ) -> CoreResult<()> {
        let integrations = {
            let conn      = state.db.connection();
            let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;
            TrackerRepository::get_user_integrations(&conn_lock, user_id)?
        };

        for integration in integrations {
            if !integration.sync_enabled { continue; }
            if let Err(e) = Self::sync_entry_to_single_tracker(state, user_id, cid, &integration).await {
                tracing::error!("Sync error for tracker {}: {}", integration.tracker_name, e);
            }
        }
        Ok(())
    }

    async fn sync_entry_to_single_tracker(
        state: &Arc<AppState>,
        user_id: i32,
        cid: &str,
        integration: &TrackerIntegration,
    ) -> CoreResult<()> {
        let provider = match state.tracker_registry.get(&integration.tracker_name) {
            Some(p) => p,
            None => {
                tracing::warn!("Tracker '{}' not found in registry, skipping sync", integration.tracker_name);
                return Ok(());
            }
        };

        let remote_id = {
            let conn      = state.db.connection();
            let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;
            let mappings  = TrackerRepository::get_mappings_by_cid(&conn_lock, cid)?;
            mappings.into_iter()
                .find(|m| m.tracker_name == integration.tracker_name)
                .map(|m| m.tracker_id)
        };

        let remote_id = match remote_id {
            Some(id) => id,
            None => return Ok(()),
        };

        let entry = {
            let conn      = state.db.connection();
            let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;
            ListRepo::get_entry(&conn_lock, user_id, cid)?
        };

        let entry = match entry {
            Some(e) => e,
            None => return Ok(()),
        };

        let params = UpdateEntryParams {
            media_id:     remote_id,
            status:       Some(entry.status),
            progress:     Some(entry.progress),
            score:        entry.score,
            start_date:   entry.start_date,
            end_date:     entry.end_date,
            repeat_count: Some(entry.repeat_count),
            notes:        entry.notes,
            is_private:   Some(entry.is_private),
        };

        provider.update_entry(&integration.access_token, params).await?;
        Ok(())
    }

    async fn delete_from_trackers(
        state: &Arc<AppState>,
        user_id: i32,
        cid: &str,
    ) -> CoreResult<()> {
        let integrations = {
            let conn      = state.db.connection();
            let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;
            TrackerRepository::get_user_integrations(&conn_lock, user_id)?
        };

        for integration in integrations {
            if !integration.sync_enabled { continue; }

            let provider = match state.tracker_registry.get(&integration.tracker_name) {
                Some(p) => p,
                None => {
                    tracing::warn!("Tracker '{}' not found in registry, skipping delete", integration.tracker_name);
                    continue;
                }
            };

            let remote_id = {
                let conn      = state.db.connection();
                let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;
                let mappings  = TrackerRepository::get_mappings_by_cid(&conn_lock, cid)?;
                mappings.into_iter()
                    .find(|m| m.tracker_name == integration.tracker_name)
                    .map(|m| m.tracker_id)
            };

            if let Some(id) = remote_id {
                if let Err(e) = provider.delete_entry(&integration.access_token, &id).await {
                    tracing::error!("Failed to delete from tracker '{}': {}", integration.tracker_name, e);
                }
            }
        }

        Ok(())
    }
}