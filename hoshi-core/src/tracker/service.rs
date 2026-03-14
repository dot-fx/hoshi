use std::sync::Arc;
use serde::Serialize;

use crate::error::{CoreError, CoreResult};
use crate::list::repository::ListRepo;
use crate::list::service::UpsertEntryBody;
use crate::state::AppState;
use crate::tracker::repository::{AddIntegrationRequest, TrackerIntegration, TrackerRepository};
use crate::tracker::provider::TrackerAuthConfig;
use crate::tracker::provider::UpdateEntryParams;
use crate::content::import_service::ContentImportService;
use crate::backup::repository::{BackupRepository, BackupTrigger};

pub fn normalize_list_status(s: &str) -> String {
    match s.to_uppercase().as_str() {
        "CURRENT" | "WATCHING" | "AIRING"                    => "CURRENT",
        "COMPLETED" | "FINISHED" | "WATCHED"                 => "COMPLETED",
        "PLANNING" | "PLAN_TO_WATCH" | "PTW"
        | "PLAN TO WATCH" | "WANT TO WATCH"                  => "PLANNING",
        "PAUSED" | "ON_HOLD" | "HOLD"                        => "PAUSED",
        "DROPPED" | "ABANDONED"                              => "DROPPED",
        "REPEATING" | "REWATCHING" | "REREADING"             => "REPEATING",
        _                                                     => "PLANNING",
    }.to_string()
}

fn status_priority(s: &str) -> u8 {
    match s {
        "COMPLETED"  => 6,
        "REPEATING"  => 5,
        "CURRENT"    => 4,
        "PAUSED"     => 3,
        "DROPPED"    => 2,
        "PLANNING"   => 1,
        _            => 0,
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationsResponse {
    pub integrations: Vec<TrackerIntegration>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackerInfoResponse {
    pub name: String,
    pub display_name: String,
    pub icon_url: String,
    pub supported_types: Vec<String>,
    pub auth: TrackerAuthConfig,
    pub connected: bool,
    pub tracker_user_id: Option<String>,
    pub sync_enabled: Option<bool>,
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

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetSyncEnabledRequest {
    pub enabled: bool,
}

pub struct IntegrationService;

impl IntegrationService {

    pub fn set_sync_enabled(
        state: &AppState,
        user_id: i32,
        tracker_name: &str,
        enabled: bool,
    ) -> CoreResult<SuccessResponse> {
        let conn = state.db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;
        TrackerRepository::set_sync_enabled(&conn_lock, user_id, tracker_name, enabled)?;
        Ok(SuccessResponse { success: true })
    }

    pub fn get_integrations(state: &AppState, user_id: i32) -> CoreResult<IntegrationsResponse> {
        let conn = state.db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;
        let integrations = TrackerRepository::get_user_integrations(&conn_lock, user_id)?;
        Ok(IntegrationsResponse { integrations })
    }

    pub fn list_trackers(state: &AppState, user_id: i32) -> CoreResult<Vec<TrackerInfoResponse>> {
        let conn = state.db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;
        let integrations = TrackerRepository::get_user_integrations(&conn_lock, user_id)?;
        drop(conn_lock);

        let result = state
            .tracker_registry
            .all()
            .into_iter()
            .map(|provider| {
                let integration = integrations
                    .iter()
                    .find(|i| i.tracker_name == provider.name());

                TrackerInfoResponse {
                    name: provider.name().to_string(),
                    display_name: provider.display_name().to_string(),
                    icon_url: provider.icon_url().to_string(),
                    supported_types: provider
                        .supported_types()
                        .iter()
                        .map(|t| t.as_str().to_string())
                        .collect(),
                    auth: provider.auth_config(),
                    connected: integration.is_some(),
                    tracker_user_id: integration.map(|i| i.tracker_user_id.clone()),
                    sync_enabled: integration.map(|i| i.sync_enabled),
                }
            })
            .collect();

        Ok(result)
    }

    pub async fn add_integration(
        state: Arc<AppState>,
        user_id: i32,
        body: AddIntegrationRequest,
    ) -> CoreResult<SuccessResponse> {
        let provider = state
            .tracker_registry
            .get(&body.tracker_name)
            .ok_or_else(|| CoreError::Internal(format!("Unknown tracker: {}", body.tracker_name)))?;

        let token_data = provider
            .validate_and_store_token(&body.access_token, "Bearer")
            .await?;

        let expires_at = chrono::DateTime::parse_from_rfc3339(&token_data.expires_at)
            .map(|dt| dt.timestamp())
            .unwrap_or_else(|_| chrono::Utc::now().timestamp() + 31_536_000);

        {
            let conn = state.db.connection();
            let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;

            TrackerRepository::save_integration(
                &conn_lock,
                user_id,
                &body.tracker_name,
                &token_data.tracker_user_id,
                &token_data.access_token,
                token_data.refresh_token.as_deref(),
                &token_data.token_type,
                expires_at,
            )?;

            TrackerRepository::set_sync_enabled(&conn_lock, user_id, &body.tracker_name, false)?;
        }

        // Lanzar el import inicial en background — no bloqueamos la respuesta al cliente
        let state_clone = state.clone();
        let tracker_name = body.tracker_name.clone();
        tokio::spawn(async move {
            tracing::info!(
                "Starting initial import from '{}' for user {}",
                tracker_name, user_id
            );
            match TrackerSyncService::import_from_tracker_by_name(
                &state_clone,
                user_id,
                &tracker_name,
            ).await {
                Ok(count) => tracing::info!(
                    "Initial import from '{}' for user {}: {} entries",
                    tracker_name, user_id, count
                ),
                Err(e) => tracing::error!(
                    "Initial import from '{}' for user {} failed: {}",
                    tracker_name, user_id, e
                ),
            }
        });

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

    pub async fn import_from_tracker_by_name(
        state: &Arc<AppState>,
        user_id: i32,
        tracker_name: &str,
    ) -> CoreResult<i32> {
        let integration = {
            let conn = state.db.connection();
            let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;
            TrackerRepository::get_user_integrations(&conn_lock, user_id)?
                .into_iter()
                .find(|i| i.tracker_name == tracker_name)
                .ok_or_else(|| CoreError::NotFound(format!(
                    "Integration '{}' not found for user {}", tracker_name, user_id
                )))?
        };

        let provider = state.tracker_registry.get(tracker_name)
            .ok_or_else(|| CoreError::Internal(format!("Tracker '{}' not in registry", tracker_name)))?;

        Self::import_from_tracker(state, user_id, &integration, provider).await
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

        {
            let conn = state.db.connection();
            let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;
            if let Err(e) = BackupRepository::create_snapshot(
                &conn_lock,
                &state.paths,
                user_id,
                BackupTrigger::PreImport,
                Some(&integration.tracker_name),
            ) {
                tracing::warn!(
                    "Could not create pre-import backup for tracker '{}': {}",
                    integration.tracker_name, e
                );
            }
        }

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
                Some(existing_cid) => {
                    existing_cid
                }
                None => {
                    // Prefer inline media if present, but if it's a shallow stub
                    // (no synopsis + no characters) try to fetch full metadata from
                    // the provider first so import_media gets rich data to store.
                    let tracker_media = {
                        let inline = remote.media.clone();
                        let needs_fetch = inline.as_ref()
                            .map(|m| m.synopsis.is_none() && m.characters.is_empty())
                            .unwrap_or(true); // no inline at all → must fetch

                        if needs_fetch {
                            tracing::debug!(
                                "Fetching full metadata for {} from {}",
                                remote.tracker_media_id, integration.tracker_name
                            );
                            match provider.get_by_id(&remote.tracker_media_id).await {
                                Ok(Some(full)) => full,
                                Ok(None) => {
                                    // Provider doesn't know this id — fall back to
                                    // inline stub if we have one, otherwise skip.
                                    match inline {
                                        Some(m) => m,
                                        None => {
                                            tracing::warn!(
                                                "No media found for {} in {}, skipping",
                                                remote.tracker_media_id, integration.tracker_name
                                            );
                                            continue;
                                        }
                                    }
                                }
                                Err(e) => {
                                    // Network/API error — fall back to inline stub if
                                    // available so we don't lose the list entry entirely.
                                    tracing::warn!(
                                        "get_by_id failed for {} in {}: {} — falling back to inline stub",
                                        remote.tracker_media_id, integration.tracker_name, e
                                    );
                                    match inline {
                                        Some(m) => m,
                                        None => continue,
                                    }
                                }
                            }
                        } else {
                            // Inline stub looks complete enough, use it directly.
                            inline.unwrap()
                        }
                    };

                    let conn = state.db.connection();
                    let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;
                    match ContentImportService::import_media(
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

            let remote_status = normalize_list_status(
                remote.status.as_deref().unwrap_or("PLANNING")
            );

            let (final_status, final_progress, final_score, final_start, final_end) = {
                let conn = state.db.connection();
                let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;
                let local = ListRepo::get_entry(&conn_lock, user_id, &cid)?;

                match local {
                    None => {
                        (
                            remote_status,
                            remote.progress,
                            remote.score,
                            remote.start_date.clone(),
                            remote.end_date.clone(),
                        )
                    }
                    Some(local_entry) => {
                        let progress = remote.progress.max(local_entry.progress);

                        let status = if status_priority(&remote_status)
                            >= status_priority(&local_entry.status)
                        {
                            remote_status.clone()
                        } else {
                            local_entry.status.clone()
                        };

                        let score = remote.score.or(local_entry.score);

                        let start = local_entry.start_date.or(remote.start_date.clone());
                        let end = local_entry.end_date.or(remote.end_date.clone());

                        tracing::debug!(
                            "Merge conflict for cid={}: local progress={} status={}, \
                             remote progress={} status={} → resolved: progress={} status={}",
                            cid, local_entry.progress, local_entry.status,
                            remote.progress, remote_status,
                            progress, status
                        );

                        (status, progress, score, start, end)
                    }
                }
            };

            let body = UpsertEntryBody {
                cid: cid.clone(),
                status: final_status.clone(),
                progress: Some(final_progress),
                score: final_score,
                start_date: final_start.clone(),
                end_date: final_end.clone(),
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
                    &final_status,
                    final_progress,
                    final_start,
                    final_end,
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