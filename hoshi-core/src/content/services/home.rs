use std::sync::Arc;
use chrono::Utc;
use tracing::{error, info, warn};
use crate::content::models::FullContent;
use crate::content::repositories::cache::CacheRepository;
use crate::content::repositories::content::ContentRepository;
use crate::content::services::import::ImportService;
use crate::content::types::{HomeView, MediaSection};
use crate::content::utils::{show_adult};
use crate::error::{CoreError, CoreResult};
use crate::state::AppState;
use crate::tracker::provider::TrackerMedia;
use crate::tracker::repository::TrackerRepository;

const HOME_CACHE_KEY: &str = "home_view_v2";
const HOME_CACHE_TTL: i64  = 6 * 3600; // 6 horas

pub struct HomeService;

impl HomeService {
    async fn refresh_home_cache(state: Arc<AppState>) -> CoreResult<()> {
        let provider = state.tracker_registry.get("anilist")
            .ok_or_else(|| CoreError::Internal("error.tracker.anilist_not_registered".into()))?;

        let sections = provider.get_home().await?;

        let resolve = |medias: Vec<TrackerMedia>| {
            let state = state.clone();
            async move {
                let mut result = Vec::with_capacity(medias.len());
                for media in medias {
                    match Self::import_and_load(&state, &media).await {
                        Ok(full) => result.push(full),
                        Err(e) => warn!(error = ?e, id = %media.tracker_id, "Failed to import home entry"),
                    }
                }
                result
            }
        };

        let view = HomeView {
            anime: MediaSection {
                trending:  resolve(sections.get("trending_anime").cloned().unwrap_or_default()).await,
                top_rated: resolve(sections.get("top_rated_anime").cloned().unwrap_or_default()).await,
                seasonal:  Some(resolve(sections.get("seasonal_anime").cloned().unwrap_or_default()).await),
            },
            manga: MediaSection {
                trending:  resolve(sections.get("trending_manga").cloned().unwrap_or_default()).await,
                top_rated: resolve(sections.get("top_rated_manga").cloned().unwrap_or_default()).await,
                seasonal:  None,
            },
            novel: MediaSection {
                trending:  resolve(sections.get("trending_novel").cloned().unwrap_or_default()).await,
                top_rated: resolve(sections.get("top_rated_novel").cloned().unwrap_or_default()).await,
                seasonal:  None,
            },
            cached_at: Utc::now().timestamp(),
        };

        let value = serde_json::to_value(&view)
            .map_err(|_| CoreError::Internal("error.content.serialization".into()))?;

        CacheRepository::set(&state.pool, HOME_CACHE_KEY, "anilist", "home", &value, 30 * 24 * 3600).await?;

        info!("Home cache updated from AniList");
        Ok(())
    }

    async fn import_and_load(state: &Arc<AppState>, media: &TrackerMedia) -> CoreResult<FullContent> {
        let existing = TrackerRepository::find_cid_by_tracker(
            &state.pool, "anilist", &media.tracker_id
        ).await?;

        let cid = match existing {
            Some(cid) => cid,
            None => ImportService::import_media(&state.pool, "anilist", media).await?,
        };

        ContentRepository::get_full_content(&state.pool, &cid).await?
            .ok_or_else(|| CoreError::NotFound("error.content.not_found".into()))
    }

    pub async fn get_home_view(state: &Arc<AppState>, user_id: i32) -> CoreResult<HomeView> {
        let adult_enabled = show_adult(state, user_id).await;

        if let Some(cached_value) = CacheRepository::get(&state.pool, HOME_CACHE_KEY).await? {
            let mut view: HomeView = serde_json::from_value(cached_value)
                .map_err(|e| CoreError::Internal(format!("Cache corrupto: {}", e)))?;

            if Utc::now().timestamp() - view.cached_at > HOME_CACHE_TTL {
                let state_clone = state.clone();
                tokio::spawn(async move {
                    if let Err(e) = Self::refresh_home_cache(state_clone).await {
                        error!(error = ?e, "Background home refresh failed");
                    }
                });
            }

            if !adult_enabled {
                view.filter_nsfw();
            }
            return Ok(view);
        }

        Self::refresh_home_cache(state.clone()).await?;

        let value = CacheRepository::get(&state.pool, HOME_CACHE_KEY)
            .await?
            .ok_or_else(|| CoreError::Internal("error.content.home_cache_missing".into()))?;

        let mut view: HomeView = serde_json::from_value(value)?;

        if !adult_enabled {
            view.filter_nsfw();
        }

        Ok(view)
    }

    pub async fn get_trending(
        state: &Arc<AppState>,
        media_type: &str,
        user_id: i32
    ) -> CoreResult<Vec<FullContent>> {
        let view = Self::get_home_view(state, user_id).await?;

        let trending_list = match media_type {
            "anime" => view.anime.trending,
            "manga" => view.manga.trending,
            "novel" => view.novel.trending,
            _ => return Err(CoreError::NotFound("error.content.invalid_media_type".into())),
        };

        Ok(trending_list)
    }
}