use chrono::Utc;
use std::sync::Arc;

use crate::content::repository::ContentRepository;
use crate::content::service::ContentImportService;
use crate::error::{CoreError, CoreResult};
use crate::list::repository::ListRepo;
use crate::schedule::repository::{AiringEntryEnriched, ScheduleRepository, ScheduleWindow};
use crate::state::AppState;
use crate::tracker::repository::TrackerRepository;

const SCHEDULE_SYNC_TTL: i64 = 6 * 3600; // 6 hours

fn sync_cache_key(cid: &str) -> String {
    format!("schedule:sync:{}", cid)
}

pub struct ScheduleService;

impl ScheduleService {

    pub async fn get_schedule(
        state: Arc<AppState>,
        user_id: i32,
        window: ScheduleWindow,
    ) -> CoreResult<Vec<AiringEntryEnriched>> {
        let list_cids: Vec<String> = {
            let conn = state.db.connection();
            let lock = conn.lock().map_err(|_| CoreError::Internal("DB lock".into()))?;

            let current  = ListRepo::get_entries(&lock, user_id, Some("CURRENT"))?;
            let planning = ListRepo::get_entries(&lock, user_id, Some("PLANNING"))?;

            current.into_iter()
                .chain(planning)
                .map(|e| e.cid)
                .collect()
        };

        if list_cids.is_empty() {
            return Ok(vec![]);
        }

        for cid in &list_cids {
            if let Err(e) = Self::maybe_sync_cid(&state, cid).await {
                tracing::warn!("Schedule sync failed for cid {}: {}", cid, e);
            }
        }

        let now = Utc::now().timestamp();
        let from_ts = now - window.days_back  * 86_400;
        let to_ts   = now + window.days_ahead * 86_400;

        let entries = {
            let conn = state.db.connection();
            let lock = conn.lock().map_err(|_| CoreError::Internal("DB lock".into()))?;
            ScheduleRepository::get_by_cids_in_window(&lock, &list_cids, from_ts, to_ts)?
        };

        let mut enriched = Vec::with_capacity(entries.len());
        for entry in entries {
            let conn = state.db.connection();
            let lock = conn.lock().map_err(|_| CoreError::Internal("DB lock".into()))?;

            let meta = ContentRepository::get_by_cid(&lock, &entry.cid)?;
            let list_entry = ListRepo::get_entry(&lock, user_id, &entry.cid)?;

            let (title, subtype, cover_image, banner_image, synopsis, status,
                genres, tags, nsfw, rating, release_date, end_date, trailer_url, studio) = match meta {
                Some(m) => (
                    m.title,
                    m.subtype,
                    m.cover_image,
                    m.banner_image,
                    m.synopsis,
                    m.status.map(|s| format!("{:?}", s).to_lowercase()),
                    m.genres,
                    m.tags,
                    m.nsfw,
                    m.rating,
                    m.release_date,
                    m.end_date,
                    m.trailer_url,
                    m.studio,
                ),
                None => (
                    entry.cid.clone(), None, None, None, None, None,
                    vec![], vec![], false, None, None, None, None, None,
                ),
            };

            enriched.push(AiringEntryEnriched {
                id:           entry.id,
                cid:          entry.cid,
                episode:      entry.episode,
                airing_at:    entry.airing_at,
                title,
                subtype,
                cover_image,
                banner_image,
                synopsis,
                status,
                genres,
                tags,
                nsfw,
                rating,
                release_date,
                end_date,
                trailer_url,
                studio,
                user_status:   list_entry.as_ref().map(|e| e.status.clone()),
                user_progress: list_entry.as_ref().map(|e| e.progress),
                user_score:    list_entry.and_then(|e| e.score),
            });
        }

        Ok(enriched)
    }


    async fn maybe_sync_cid(state: &Arc<AppState>, cid: &str) -> CoreResult<()> {
        let anilist_id: i64 = {
            let conn = state.db.connection();
            let lock = conn.lock().map_err(|_| CoreError::Internal("DB lock".into()))?;

            let mappings = TrackerRepository::get_mappings_by_cid(&lock, cid)?;
            match mappings.into_iter().find(|m| m.tracker_name == "anilist") {
                Some(m) => match m.tracker_id.parse::<i64>() {
                    Ok(id) => id,
                    Err(_) => {
                        tracing::debug!("cid {} has non-numeric anilist_id, skipping schedule sync", cid);
                        return Ok(());
                    }
                },
                None => {
                    return Ok(());
                }
            }
        };

        let needs_sync = {
            let conn = state.db.connection();
            let lock = conn.lock().map_err(|_| CoreError::Internal("DB lock".into()))?;

            let has_data = ScheduleRepository::has_any(&lock, cid)?;
            if !has_data {
                true
            } else {
                use crate::content::repository::CacheRepository;
                CacheRepository::get(&lock, &sync_cache_key(cid))?.is_none()
            }
        };

        if !needs_sync {
            return Ok(());
        }

        tracing::debug!("Syncing airing schedule for cid {} (anilist_id {})", cid, anilist_id);
        Self::sync_from_anilist(state, cid, anilist_id).await
    }

    async fn sync_from_anilist(
        state: &Arc<AppState>,
        cid: &str,
        anilist_id: i64,
    ) -> CoreResult<()> {
        let anilist_provider = crate::tracker::provider::anilist::AniListProvider::new();

        let episodes = crate::tracker::provider::anilist::fetch_airing_schedule(
            &anilist_provider,
            anilist_id,
        ).await?;

        if episodes.is_empty() {
            tracing::debug!("No airing schedule entries returned for anilist_id {}", anilist_id);
            Self::mark_synced(state, cid)?;
            return Ok(());
        }

        if let Some(media) = episodes.iter().find_map(|e| e.media.as_ref()) {
            let conn = state.db.connection();
            let lock = conn.lock().map_err(|_| CoreError::Internal("DB lock".into()))?;
            if let Err(e) = ContentImportService::import_media(&lock, "anilist", media) {
                tracing::warn!("Failed to import media for cid {} during schedule sync: {}", cid, e);
            }
        }

        {
            let conn = state.db.connection();
            let lock = conn.lock().map_err(|_| CoreError::Internal("DB lock".into()))?;
            for ep in &episodes {
                if let Err(e) = ScheduleRepository::upsert(&lock, cid, ep.episode, ep.airing_at) {
                    tracing::warn!(
                        "Failed to upsert episode {} for cid {}: {}",
                        ep.episode, cid, e
                    );
                }
            }
        }

        Self::mark_synced(state, cid)?;
        tracing::info!(
            "Schedule synced for cid {} ({} episodes)",
            cid,
            episodes.len()
        );
        Ok(())
    }

    fn mark_synced(state: &Arc<AppState>, cid: &str) -> CoreResult<()> {
        use crate::content::repository::CacheRepository;
        let conn = state.db.connection();
        let lock = conn.lock().map_err(|_| CoreError::Internal("DB lock".into()))?;
        CacheRepository::set(
            &lock,
            &sync_cache_key(cid),
            "anilist",
            "schedule_sync",
            &serde_json::json!({ "synced": true }),
            SCHEDULE_SYNC_TTL,
        )?;
        Ok(())
    }
}