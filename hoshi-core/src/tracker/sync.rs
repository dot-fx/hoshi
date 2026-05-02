// tracker/sync.rs
use std::sync::Arc;
use tracing::{error, info, warn};
use crate::content::repositories::content::ContentRepository;
use crate::error::CoreResult;
use crate::list::repository::ListRepository;
use crate::list::types::UpsertEntryBody;
use crate::list::service::ListService;
use crate::state::AppState;
use crate::tracker::provider::UserListEntry;
use crate::tracker::repository::TrackerRepository;
use crate::users::repository::UserRepo;
use crate::content::services::enrichment::EnrichmentService;

pub struct StartupSyncService;

impl StartupSyncService {
    pub fn run(state: Arc<AppState>) {
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;

            let users = match UserRepo::get_all_users(&state.pool).await {
                Ok(u) => u,
                Err(e) => { error!(error = ?e, "Startup sync: failed to fetch users"); return; }
            };

            info!(users = users.len(), "Starting tracker startup sync");

            for user in users {
                let integrations = match TrackerRepository::get_user_integrations(&state.pool, user.id).await {
                    Ok(i) => i,
                    Err(e) => { warn!(error = ?e, user_id = user.id, "Failed to fetch integrations"); continue; }
                };

                for integration in integrations {
                    if !integration.sync_enabled { continue; }

                    let state = state.clone();
                    tokio::spawn(async move {
                        let provider = match state.tracker_registry.get(&integration.tracker_name) {
                            Some(p) => p,
                            None => { warn!(tracker = %integration.tracker_name, "Not in registry, skipping"); return; }
                        };

                        let entries = match provider.get_user_list(
                            &integration.access_token,
                            &integration.tracker_user_id,
                        ).await {
                            Ok(e) => e,
                            Err(e) => { warn!(error = ?e, tracker = %integration.tracker_name, user_id = integration.user_id, "Failed to fetch remote list"); return; }
                        };

                        info!(
                            tracker = %integration.tracker_name,
                            user_id = integration.user_id,
                            count = entries.len(),
                            "Syncing entries from tracker"
                        );

                        let mut imported = 0usize;
                        let mut skipped  = 0usize;

                        for entry in entries {
                            match Self::merge_entry(&state, integration.user_id, &integration.tracker_name, &entry).await {
                                Ok(true)  => imported += 1,
                                Ok(false) => skipped  += 1,
                                Err(e)    => warn!(error = ?e, tracker_id = %entry.tracker_media_id, "Failed to merge entry"),
                            }
                        }

                        info!(
                            tracker = %integration.tracker_name,
                            user_id = integration.user_id,
                            imported, skipped,
                            "Tracker sync complete"
                        );
                    });
                }
            }
        });
    }

    async fn merge_entry(
        state: &Arc<AppState>,
        user_id: i32,
        tracker_name: &str,
        entry: &UserListEntry,
    ) -> CoreResult<bool> {
        let cid = match TrackerRepository::find_cid_by_tracker(
            &state.pool, tracker_name, &entry.tracker_media_id,
        ).await? {
            Some(cid) => cid,
            None => {
                let media = match &entry.media {
                    Some(m) => m,
                    None => return Ok(false),
                };
                let full = EnrichmentService::create_enriched_content(
                    state,
                    &entry.content_type,
                    media,
                    &entry.tracker_media_id,
                    tracker_name,
                    None,
                ).await?;
                full.content.cid
            }
        };

        let local = ListRepository::get_entry(&state.pool, user_id, &cid).await?;
        if let Some(ref local_entry) = local {
            if local_entry.progress >= entry.progress {
                return Ok(false);
            }
        }

        let body = UpsertEntryBody {
            cid:          cid.clone(),
            status:       entry.status.clone().unwrap_or_else(|| "PLANNING".into()),
            progress:     Some(entry.progress),
            score:        entry.score,
            start_date:   entry.start_date.clone(),
            end_date:     entry.end_date.clone(),
            repeat_count: Some(entry.repeat_count),
            notes:        entry.notes.clone(),
            is_private:   Some(entry.is_private),
        };

        ListService::upsert_entry(state.clone(), user_id, body).await?;
        Ok(true)
    }
}