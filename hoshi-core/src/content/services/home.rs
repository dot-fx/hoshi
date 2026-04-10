use std::sync::Arc;
use tracing::{debug, error, info, instrument};
use serde_json::{json, Value};
use crate::content::repositories::cache::CacheRepository;
use crate::content::types::{HomeView, MediaSection};
use crate::content::utils::{filter_array_nsfw, filter_home_nsfw, show_adult};
use crate::error::{CoreError, CoreResult};
use crate::state::AppState;

const HOME_CACHE_KEY: &str = "home_view_v2";
const HOME_CACHE_TTL: i64  = 6 * 3600; // 6 horas

pub struct HomeService;

impl HomeService {
    #[instrument(skip(state))]
    async fn refresh_home_cache(state: Arc<AppState>) -> CoreResult<()> {
        debug!("Refreshing home cache from AniList");

        let provider = state.tracker_registry.get("anilist")
            .ok_or_else(|| CoreError::Internal("error.tracker.anilist_not_registered".into()))?;

        let sections = provider.get_home().await?;

        let now = chrono::Utc::now().timestamp();

        let view = HomeView {
            anime: MediaSection {
                trending:  sections.get("trending_anime").cloned().unwrap_or_default(),
                top_rated: sections.get("top_rated_anime").cloned().unwrap_or_default(),
                seasonal:  Some(sections.get("seasonal_anime").cloned().unwrap_or_default()),
            },
            manga: MediaSection {
                trending:  sections.get("trending_manga").cloned().unwrap_or_default(),
                top_rated: sections.get("top_rated_manga").cloned().unwrap_or_default(),
                seasonal:  None,
            },
            novel: MediaSection {
                trending:  sections.get("trending_novel").cloned().unwrap_or_default(),
                top_rated: sections.get("top_rated_novel").cloned().unwrap_or_default(),
                seasonal:  None,
            },
            cached_at: now,
        };

        let value = serde_json::to_value(&view)
            .map_err(|_| CoreError::Internal("error.content.serialization".into()))?;

        CacheRepository::set(&state.pool, HOME_CACHE_KEY, "anilist", "home", &value, 30 * 24 * 3600).await?;

        info!("Home cache updated from AniList");
        Ok(())
    }

    pub async fn get_home_view(state: &Arc<AppState>, user_id: i32) -> CoreResult<Value> {
        let adult_enabled = show_adult(state, user_id).await;

        let cached = CacheRepository::get(&state.pool, HOME_CACHE_KEY).await?;

        if let Some(cached) = cached {
            let cached_at = cached["cachedAt"].as_i64().unwrap_or(0);
            if chrono::Utc::now().timestamp() - cached_at > HOME_CACHE_TTL {
                let state_clone = state.clone();
                tokio::spawn(async move {
                    if let Err(e) = Self::refresh_home_cache(state_clone).await {
                        error!(error = ?e, "Background home refresh failed");
                    }
                });
            }
            return Ok(if adult_enabled { cached } else { filter_home_nsfw(cached) });
        }

        Self::refresh_home_cache(state.clone()).await?;

        let value = CacheRepository::get(&state.pool, HOME_CACHE_KEY)
            .await?
            .ok_or_else(|| CoreError::Internal("error.content.home_cache_missing".into()))?;

        Ok(if adult_enabled { value } else { filter_home_nsfw(value) })
    }

    pub async fn get_trending(state: &Arc<AppState>, media_type: &str, user_id: i32) -> CoreResult<Value> {
        let adult_enabled = show_adult(state, user_id).await;

        let mut home_cache = CacheRepository::get(&state.pool, HOME_CACHE_KEY).await?;

        if home_cache.is_none() {
            Self::refresh_home_cache(state.clone()).await?;
            home_cache = CacheRepository::get(&state.pool, HOME_CACHE_KEY).await?;
        }

        let home_val = home_cache
            .ok_or_else(|| CoreError::Internal("error.content.home_cache_missing".into()))?;

        let trending_list = home_val.get(media_type)
            .and_then(|m| m.get("trending"))
            .cloned()
            .unwrap_or(json!([]));

        Ok(if adult_enabled { trending_list } else { filter_array_nsfw(trending_list) })
    }
}