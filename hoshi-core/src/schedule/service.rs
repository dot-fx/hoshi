use chrono::Utc;
use std::sync::Arc;
use tracing::{debug, info, instrument, warn};

use crate::content::repositories::content::ContentRepository;
use crate::content::services::import::ImportService;
use crate::error::{CoreError, CoreResult};
use crate::list::repository::ListRepository;
use crate::schedule::repository::ScheduleRepository;
use crate::schedule::types::{AiringEntryEnriched, ScheduleWindow};
use crate::state::AppState;
use crate::tracker::repository::TrackerRepository;

const SCHEDULE_SYNC_TTL: i64 = 48 * 3600;

fn sync_cache_key(cid: &str) -> String {
    format!("schedule:sync:{}", cid)
}

pub struct ScheduleService;

impl ScheduleService {
    #[instrument(skip(state))]
    pub async fn get_schedule(
        state: Arc<AppState>,
        user_id: i32,
        window: ScheduleWindow,
    ) -> CoreResult<Vec<AiringEntryEnriched>> {
        debug!("Generating airing schedule for user library");

        let pool = state.pool();

        let current  = ListRepository::get_entries(pool, user_id, Some("CURRENT")).await?;
        let planning = ListRepository::get_entries(pool, user_id, Some("PLANNING")).await?;

        let list_cids: Vec<String> = current.into_iter()
            .chain(planning)
            .map(|e| e.cid)
            .collect();

        if list_cids.is_empty() {
            debug!("User library is empty, returning empty schedule");
            return Ok(vec![]);
        }

        for cid in &list_cids {
            if let Err(e) = Self::maybe_sync_cid(&state, cid).await {
                warn!(cid = %cid, error = ?e, "Schedule sync failed for content");
            }
        }

        let now     = Utc::now().timestamp();
        let from_ts = now - window.days_back  * 86_400;
        let to_ts   = now + window.days_ahead * 86_400;

        let entries = ScheduleRepository::get_by_cids_in_window(pool, &list_cids, from_ts, to_ts).await?;

        let entry_cids: Vec<String> = entries.iter().map(|e| e.cid.clone()).collect();

        let all_content    = ContentRepository::get_contents_by_cids(pool, &entry_cids).await?;
        let all_meta       = ContentRepository::get_metas_by_cids(pool, &entry_cids).await?;
        let all_list       = ListRepository::get_entries_by_cids(pool, user_id, &entry_cids).await?;

        let content_map: std::collections::HashMap<_, _> =
            all_content.into_iter().map(|c| (c.cid.clone(), c)).collect();
        let meta_map: std::collections::HashMap<_, _> =
            all_meta.into_iter().map(|m| (m.cid.clone(), m)).collect();
        let list_map: std::collections::HashMap<_, _> =
            all_list.into_iter().map(|e| (e.cid.clone(), e)).collect();

        let mut enriched = Vec::with_capacity(entries.len());
        for entry in entries {
            let content    = content_map.get(&entry.cid);
            let meta       = meta_map.get(&entry.cid);
            let list_entry = list_map.get(&entry.cid);

            let nsfw = content.map(|c| c.nsfw).unwrap_or(false);
            let (title, title_i18n, subtype, cover_image, banner_image, synopsis, status,
                genres, rating, release_date, end_date, trailer_url, studio) = match meta {
                Some(m) => (
                    m.title.clone(), m.title_i18n.clone(), m.subtype.clone(),
                    m.cover_image.clone(), m.banner_image.clone(), m.synopsis.clone(),
                    m.status.as_ref().map(|s| format!("{:?}", s).to_lowercase()),
                    m.genres.clone(), m.rating, m.release_date.clone(),
                    m.end_date.clone(), m.trailer_url.clone(), m.studio.clone(),
                ),
                None => (
                    entry.cid.clone(), std::collections::HashMap::new(), None,
                    None, None, None, None, vec![], None, None, None, None, None,
                ),
            };

            enriched.push(AiringEntryEnriched {
                id: entry.id, cid: entry.cid, episode: entry.episode, airing_at: entry.airing_at,
                title, title_i18n, subtype, cover_image, banner_image, synopsis, status,
                genres, nsfw, rating, release_date, end_date, trailer_url, studio,
                user_status:   list_entry.map(|e| e.status.clone()),
                user_progress: list_entry.map(|e| e.progress),
                user_score:    list_entry.and_then(|e| e.score),
            });
        }

        debug!(count = enriched.len(), "Returning enriched schedule entries");
        Ok(enriched)
    }

    #[instrument(skip(state))]
    async fn maybe_sync_cid(state: &Arc<AppState>, cid: &str) -> CoreResult<()> {
        let pool = state.pool();

        use crate::content::repositories::cache::CacheRepository;
        if CacheRepository::get(pool, &sync_cache_key(cid)).await?.is_some() {
            return Ok(());
        }

        let mappings = TrackerRepository::get_mappings_by_cid(pool, cid).await?;
        let anilist_id = match mappings.into_iter().find(|m| m.tracker_name == "anilist") {
            Some(m) => match m.tracker_id.parse::<i64>() {
                Ok(id) => id,
                Err(_) => {
                    debug!(cid = %cid, "CID has non-numeric AniList ID, skipping schedule sync");
                    return Ok(());
                }
            },
            None => return Ok(()),
        };

        info!(cid = %cid, anilist_id = anilist_id, "Syncing airing schedule from AniList");
        Self::sync_from_anilist(state, cid, anilist_id).await
    }

    #[instrument(skip(state))]
    async fn sync_from_anilist(
        state: &Arc<AppState>,
        cid: &str,
        anilist_id: i64,
    ) -> CoreResult<()> {
        let pool = state.pool();
        let anilist_provider = state.tracker_registry.get("anilist")
            .ok_or_else(|| CoreError::Internal("AniList provider not found in registry".into()))?;

        let episodes = anilist_provider
            .fetch_airing_schedule(anilist_id)
            .await?;

        if episodes.is_empty() {
            debug!(anilist_id = anilist_id, "No airing schedule entries returned from provider");
            Self::mark_synced(state, cid).await?;
            return Ok(());
        }

        if let Some(media) = episodes.iter().find_map(|e| e.media.as_ref()) {
            if let Err(e) = ImportService::import_media(pool, "anilist", media).await {
                warn!(cid = %cid, error = ?e, "Failed to import media metadata during schedule sync");
            }
        }

        for ep in &episodes {
            if let Err(e) = ScheduleRepository::upsert(pool, cid, ep.episode, ep.airing_at).await {
                warn!(cid = %cid, episode = ep.episode, error = ?e, "Failed to upsert episode schedule");
            }
        }

        Self::mark_synced(state, cid).await?;
        info!(cid = %cid, count = episodes.len(), "Airing schedule synced successfully");
        Ok(())
    }

    async fn mark_synced(state: &Arc<AppState>, cid: &str) -> CoreResult<()> {
        use crate::content::repositories::cache::CacheRepository;
        CacheRepository::set(
            state.pool(),
            &sync_cache_key(cid),
            "anilist",
            "schedule_sync",
            &serde_json::json!({ "synced": true }),
            SCHEDULE_SYNC_TTL,
        ).await?;
        Ok(())
    }
}