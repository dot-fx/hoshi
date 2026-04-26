use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, instrument};

use crate::content::repositories::cache::CacheRepository;
use crate::error::{CoreError, CoreResult};
use crate::list::repository::ListRepository;
use crate::schedule::types::{AiringEntryEnriched, ScheduleWindow};
use crate::state::AppState;
use crate::tracker::repository::TrackerRepository;

const SCHEDULE_CACHE_TTL: i64 = 3 * 3600;

fn cache_key(user_id: i32) -> String {
    format!("schedule:anilist:{user_id}")
}

pub struct ScheduleService;

impl ScheduleService {
    #[instrument(skip(state))]
    pub async fn get_schedule(
        state: Arc<AppState>,
        user_id: i32,
        window: ScheduleWindow,
    ) -> CoreResult<Vec<AiringEntryEnriched>> {
        let pool = state.pool();

        let now     = Utc::now().timestamp();
        let from_ts = now - window.days_back  * 86_400;
        let to_ts   = now + window.days_ahead * 86_400;

        let current  = ListRepository::get_entries(pool, user_id, Some("CURRENT")).await?;
        let planning = ListRepository::get_entries(pool, user_id, Some("PLANNING")).await?;

        let mut list_map: HashMap<String, _> = HashMap::new();
        for entry in current.into_iter().chain(planning) {
            let mappings = TrackerRepository::get_mappings_by_cid(pool, &entry.cid).await?;
            if let Some(m) = mappings.into_iter().find(|m| m.tracker_name == "anilist") {
                list_map.insert(m.tracker_id, entry);
            }
        }

        let key = cache_key(user_id);
        let raw: Vec<AiringEntryEnriched> =
            if let Some(cached) = CacheRepository::get(pool, &key).await? {
                debug!("Schedule cache hit");
                serde_json::from_value(cached).map_err(|e| {
                    CoreError::Internal(format!("Failed to deserialise schedule cache: {e}"))
                })?
            } else {
                debug!("Schedule cache miss, fetching from AniList");

                let provider = state
                    .tracker_registry
                    .get("anilist")
                    .ok_or_else(|| CoreError::Internal("AniList provider not found".into()))?;

                let episodes = provider.fetch_airing_schedule(from_ts, to_ts).await?;
                let entries: Vec<AiringEntryEnriched> = episodes
                    .into_iter()
                    .filter_map(AiringEntryEnriched::from_airing_episode)
                    .collect();

                info!(count = entries.len(), "Fetched global airing schedule from AniList");

                CacheRepository::set(
                    pool, &key, "anilist", "airing_schedule",
                    &serde_json::to_value(&entries)
                        .map_err(|e| CoreError::Internal(format!("Serialise error: {e}")))?,
                    SCHEDULE_CACHE_TTL,
                ).await?;

                entries
            };

        let enriched = raw
            .into_iter()
            .map(|mut e| {
                if let Some(le) = list_map.get(&e.tracker_id) {
                    e.user_status   = Some(le.status.clone());
                    e.user_progress = Some(le.progress);
                    e.user_score    = le.score;
                }
                e
            })
            .collect();

        Ok(enriched)
    }
}