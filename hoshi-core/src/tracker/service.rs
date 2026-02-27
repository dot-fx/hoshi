use std::sync::Arc;
use serde::Serialize;

use crate::error::{CoreError, CoreResult};
use crate::list::repository::ListRepo;
use crate::list::service::UpsertEntryBody;
use crate::state::AppState;
use crate::tracker::repository::{TrackerIntegration, TrackerRepository};
use crate::tracker::provider::UpdateEntryParams;

pub fn normalize_list_status(s: &str) -> String {
    match s.to_uppercase().as_str() {
        "CURRENT" | "WATCHING" | "AIRING"                        => "CURRENT",
        "COMPLETED" | "FINISHED" | "WATCHED"                     => "COMPLETED",
        "PLANNING" | "PLAN_TO_WATCH" | "PTW"
        | "PLAN TO WATCH" | "WANT TO WATCH"                      => "PLANNING",
        "PAUSED" | "ON_HOLD" | "HOLD"                            => "PAUSED",
        "DROPPED" | "ABANDONED"                                   => "DROPPED",
        "REPEATING" | "REWATCHING" | "REREADING"                 => "REPEATING",
        _                                                          => "PLANNING",
    }.to_string()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationsResponse {
    pub integrations: Vec<TrackerIntegration>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncResponse {
    pub success: bool,
    pub synced: i32,
    pub errors: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SuccessResponse {
    pub success: bool,
}


pub struct IntegrationService;

impl IntegrationService {
    pub fn get_integrations(state: &AppState, user_id: i32) -> CoreResult<IntegrationsResponse> {
        let conn = state.db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;

        let integrations = TrackerRepository::get_user_integrations(&conn_lock, user_id)?;
        Ok(IntegrationsResponse { integrations })
    }

    pub fn add_integration(
        state: &AppState,
        user_id: i32,
        body: TrackerIntegration,
    ) -> CoreResult<SuccessResponse> {
        let conn = state.db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;

        TrackerRepository::save_integration(
            &conn_lock,
            user_id,
            &body.tracker_name,
            &body.tracker_user_id,
            &body.access_token,
            body.refresh_token.as_deref(),
            &body.token_type,
            body.expires_at,
        )?;
        TrackerRepository::set_sync_enabled(&conn_lock, user_id, &body.tracker_name, body.sync_enabled)?;

        Ok(SuccessResponse { success: true })
    }

    pub fn remove_integration(
        state: &AppState,
        user_id: i32,
        tracker_name: &str,
    ) -> CoreResult<SuccessResponse> {
        let conn = state.db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;

        TrackerRepository::delete_integration(&conn_lock, user_id, tracker_name)?;
        Ok(SuccessResponse { success: true })
    }
}

pub struct TrackerSyncService;

impl TrackerSyncService {
    pub async fn sync_full_account(
        state: Arc<AppState>,
        user_id: i32,
    ) -> CoreResult<SyncResponse> {
        let integrations = {
            let conn = state.db.connection();
            let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;
            TrackerRepository::get_user_integrations(&conn_lock, user_id)?
        };

        let mut synced = 0;
        let mut errors = Vec::new();

        for integration in integrations {
            if !integration.sync_enabled { continue; }

            let provider = match state.tracker_registry.get(&integration.tracker_name) {
                Some(p) => p,
                None => {
                    tracing::warn!("Tracker '{}' not in registry, skipping", integration.tracker_name);
                    continue;
                }
            };

            match Self::import_from_tracker(&state, user_id, &integration, provider).await {
                Ok(count) => synced += count,
                Err(e) => errors.push(format!("{}: {}", integration.tracker_name, e)),
            }
        }

        Ok(SyncResponse { success: true, synced, errors })
    }

    async fn import_from_tracker(
        state: &Arc<AppState>,
        user_id: i32,
        integration: &TrackerIntegration,
        provider: Arc<dyn super::provider::TrackerProvider>,
    ) -> CoreResult<i32> {
        let remote_entries = provider
            .get_user_list(&integration.access_token, &integration.tracker_user_id)
            .await?;

        let mut count = 0;

        for remote in remote_entries {
            let cid_opt = {
                let conn = state.db.connection();
                let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;
                TrackerRepository::find_cid_by_tracker(
                    &conn_lock,
                    &integration.tracker_name,
                    &remote.tracker_media_id,
                )?
            };

            let cid = match cid_opt {
                Some(c) => c,
                None => {
                    let tracker_media = match &remote.media {
                        Some(m) => m.clone(),
                        None => {
                            tracing::warn!(
                                "No media data for entry {} from {}, skipping",
                                remote.tracker_media_id, integration.tracker_name
                            );
                            continue;
                        }
                    };

                    let conn = state.db.connection();
                    let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;
                    match crate::content::service::ContentImportService::import_media(
                        &conn_lock,
                        &integration.tracker_name,
                        &tracker_media,
                    ) {
                        Ok(new_cid) => new_cid,
                        Err(e) => {
                            tracing::error!(
                                "Failed to import media {} from {}: {}",
                                remote.tracker_media_id, integration.tracker_name, e
                            );
                            continue;
                        }
                    }
                }
            };

            let status = normalize_list_status(remote.status.as_deref().unwrap_or("PLANNING"));
            let body = UpsertEntryBody {
                cid: cid.clone(),
                status: status.clone(),
                progress: Some(remote.progress),
                score: remote.score,
                start_date: remote.start_date.clone(),
                end_date: remote.end_date.clone(),
                repeat_count: Some(remote.repeat_count),
                notes: remote.notes.clone(),
                is_private: Some(remote.is_private),
            };

            {
                let conn = state.db.connection();
                let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;
                ListRepo::upsert_entry(
                    &conn_lock,
                    user_id,
                    &body,
                    &status,
                    remote.progress,
                    remote.start_date,
                    remote.end_date,
                )?;
            }

            count += 1;
        }

        Ok(count)
    }

    pub async fn sync_entry_to_all_trackers(
        state: &Arc<AppState>,
        user_id: i32,
        cid: &str,
    ) -> CoreResult<()> {
        let integrations = {
            let conn = state.db.connection();
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
            let conn = state.db.connection();
            let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;
            TrackerRepository::get_mappings_by_cid(&conn_lock, cid)?
                .into_iter()
                .find(|m| m.tracker_name == integration.tracker_name)
                .map(|m| m.tracker_id)
        };

        let remote_id = match remote_id {
            Some(id) => id,
            None => return Ok(()),
        };

        let entry = {
            let conn = state.db.connection();
            let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;
            ListRepo::get_entry(&conn_lock, user_id, cid)?
        };

        let entry = match entry {
            Some(e) => e,
            None => return Ok(()),
        };

        provider.update_entry(&integration.access_token, UpdateEntryParams {
            media_id: remote_id,
            status: Some(entry.status),
            progress: Some(entry.progress),
            score: entry.score,
            start_date: entry.start_date,
            end_date: entry.end_date,
            repeat_count: Some(entry.repeat_count),
            notes: entry.notes,
            is_private: Some(entry.is_private),
        }).await?;

        Ok(())
    }

    pub async fn delete_from_trackers(
        state: &Arc<AppState>,
        user_id: i32,
        cid: &str,
    ) -> CoreResult<()> {
        let integrations = {
            let conn = state.db.connection();
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
                let conn = state.db.connection();
                let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;
                TrackerRepository::get_mappings_by_cid(&conn_lock, cid)?
                    .into_iter()
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