use std::sync::Arc;
use std::collections::HashSet;
use tracing::{debug, info, instrument, warn};

use crate::content::models::{ContentType, FullContent, Metadata};
use crate::content::repositories::content::ContentRepository;
use crate::content::repositories::extension::ExtensionRepository;
use crate::content::services::enrichment::EnrichmentService;
use crate::content::services::extensions::ExtensionService;
use crate::content::services::resolver::ContentResolverService;
use crate::error::{CoreError, CoreResult};
use crate::extensions::types::ExtensionMetadata;
use crate::state::AppState;
use crate::tracker::repository::TrackerRepository;

const TRACKER_SOURCES: &[&str] = &["anilist", "mal", "kitsu", "anidb"];
const FUZZY_SCORE_THRESHOLD: f64 = 0.85;

pub struct ContentService;

impl ContentService {

    #[instrument(skip(state))]
    pub async fn get_content(
        state: &Arc<AppState>,
        source: &str,
        source_id: &str,
    ) -> CoreResult<FullContent> {
        if TRACKER_SOURCES.contains(&source) {
            Self::resolve_tracker_source(state, source, source_id).await
        } else {
            Self::resolve_extension_source(state, source, source_id).await
        }
    }

    #[instrument(skip(state))]
    pub async fn get_content_by_cid(
        state: &Arc<AppState>,
        cid: &str,
    ) -> CoreResult<FullContent> {
        ContentRepository::get_full_content(&state.pool, cid).await?
            .ok_or_else(|| CoreError::NotFound("error.content.not_found".into()))
    }

    async fn resolve_tracker_source(
        state: &Arc<AppState>,
        tracker: &str,
        tracker_id: &str,
    ) -> CoreResult<FullContent> {
        let maybe_cid = TrackerRepository::find_cid_by_tracker(&state.pool, tracker, tracker_id).await?;

        if let Some(cid) = maybe_cid {
            debug!(cid = %cid, tracker = %tracker, "CID found via tracker mapping");

            let mappings = TrackerRepository::get_mappings_by_cid(&state.pool, &cid).await?;
            let needs_enrich = mappings.len() == 1 && mappings[0].tracker_name == "anilist";

            if needs_enrich {
                info!(cid = %cid, "Entry has only anilist mapping, triggering enrich");
                let media = ContentResolverService::fetch_tracker_media(state, tracker, tracker_id).await?;
                return EnrichmentService::create_enriched_content(
                    state, &media.content_type, &media, tracker_id, tracker, None
                ).await;
            }

            return ContentResolverService::load_full_content(state, &cid).await;
        }

        info!(tracker = %tracker, id = %tracker_id, "No existing CID, enriching from tracker");
        let media = ContentResolverService::fetch_tracker_media(state, tracker, tracker_id).await?;
        EnrichmentService::create_enriched_content(state, &media.content_type, &media, tracker_id, tracker, None).await
    }

    async fn resolve_extension_source(
        state: &Arc<AppState>,
        ext_name: &str,
        ext_id: &str,
    ) -> CoreResult<FullContent> {
        let maybe_cid = ExtensionRepository::find_cid_by_extension(
            &state.pool, ext_name, ext_id,
        ).await?;

        if let Some(cid) = maybe_cid {
            debug!(cid = %cid, ext = %ext_name, "CID found via extension mapping");
            return ContentResolverService::load_full_content(state, &cid).await;
        }

        let ext_meta = ContentResolverService::fetch_ext_metadata(state, ext_name, ext_id).await?;
        let ext_nsfw = state.extension_manager.read().await.is_nsfw(ext_name);
        let skip = state.extension_manager.read().await.skip_default_processing(ext_name);
        let content_type = state.extension_manager.read().await.content_type(ext_name);

        if skip {
            info!(ext = %ext_name, id = %ext_id, "skip_default_processing: creating isolated entry");
            let cid = ContentResolverService::create_derived(
                state, ext_name, ext_id, &ext_meta, &content_type, ext_nsfw,
            ).await?;
            return ContentResolverService::load_full_content(state, &cid).await;
        }

        if let Some(matched) = ContentRepository::find_closest_match(
            &state.pool, &ext_meta.title, Some(content_type.clone()), ext_meta.year,
        ).await? {
            info!(cid = %matched.cid, ext = %ext_name, title = %ext_meta.title, "Found existing entry in local DB, linking");
            ContentResolverService::link(&state.pool, &matched.cid, ext_name, ext_id, ext_nsfw).await?;
            ExtensionService::save_extension_metadata(state, &matched.cid, ext_name, ext_id).await;
            return ContentResolverService::load_full_content(state, &matched.cid).await;
        }

        if let Some(full) = Self::resolve_via_tracker_ids(
            state, ext_name, ext_id, &ext_meta, &content_type, ext_nsfw,
        ).await? {
            return Ok(full);
        }

        if let Some(full) = Self::resolve_via_fuzzy(
            state, ext_name, ext_id, &ext_meta, &content_type, ext_nsfw,
        ).await? {
            return Ok(full);
        }

        warn!(ext = %ext_name, title = %ext_meta.title, "No tracker match, creating derived entry");
        let cid = ContentResolverService::create_derived(
            state, ext_name, ext_id, &ext_meta, &content_type, ext_nsfw,
        ).await?;
        ContentResolverService::load_full_content(state, &cid).await
    }

    async fn resolve_via_tracker_ids(
        state: &Arc<AppState>,
        ext_name: &str,
        ext_id: &str,
        ext_meta: &ExtensionMetadata,
        content_type: &ContentType,
        ext_nsfw: bool,
    ) -> CoreResult<Option<FullContent>> {
        if let Some(ref raw) = ext_meta.anilist_id {
            let id_str = raw.to_string();
            if let Some(full) = ContentResolverService::link_or_enrich_tracker(
                state, ext_name, ext_id, ext_nsfw, "anilist", &id_str, content_type,
            ).await? {
                return Ok(Some(full));
            }
        }

        if let Some(ref raw) = ext_meta.mal_id {
            let prefix = match content_type { ContentType::Anime => "anime", _ => "manga" };
            let id_str = format!("{}:{}", prefix, raw);
            if let Some(full) = ContentResolverService::link_or_enrich_tracker(
                state, ext_name, ext_id, ext_nsfw, "mal", &id_str, content_type,
            ).await? {
                return Ok(Some(full));
            }
        }

        Ok(None)
    }

    async fn resolve_via_fuzzy(
        state: &Arc<AppState>,
        ext_name: &str,
        ext_id: &str,
        ext_meta: &ExtensionMetadata,
        content_type: &ContentType,
        ext_nsfw: bool,
    ) -> CoreResult<Option<FullContent>> {
        let title = &ext_meta.title;
        let mut candidates: Vec<(String, String)> = Vec::new();
        let mut seen_mal_ids: HashSet<String> = HashSet::new();

        // AniList — also yields MAL IDs so we can deduplicate
        if let Some(provider) = state.tracker_registry.get("anilist") {
            match provider.search(Some(title.as_str()), content_type.clone(), 10, 1, None, None, None, None, None).await {
                Ok(results) => {
                    for item in results {
                        if crate::content::utils::similarity(title, &item.title) < FUZZY_SCORE_THRESHOLD {
                            continue;
                        }
                        if let Some(mal_id) = item.cross_ids.get("mal") {
                            let key = mal_id.clone();
                            if seen_mal_ids.insert(key.clone()) {
                                let prefix = match content_type { ContentType::Anime => "anime", _ => "manga" };
                                candidates.push(("mal".into(), format!("{}:{}", prefix, key)));
                            }
                        }
                        candidates.push(("anilist".into(), item.tracker_id.clone()));
                    }
                }
                Err(e) => warn!(error = ?e, "AniList fuzzy search failed"),
            }
        }

        // MAL — skip IDs already seen via AniList
        if let Some(provider) = state.tracker_registry.get("mal") {
            match provider.search(Some(title.as_str()), content_type.clone(), 10, 1, None, None, None, None, None).await {
                Ok(results) => {
                    for item in results {
                        if crate::content::utils::similarity(title, &item.title) < FUZZY_SCORE_THRESHOLD {
                            continue;
                        }
                        if !seen_mal_ids.contains(&item.tracker_id) {
                            let prefix = match content_type { ContentType::Anime => "anime", _ => "manga" };
                            candidates.push(("mal".into(), format!("{}:{}", prefix, &item.tracker_id)));
                        }
                    }
                }
                Err(e) => warn!(error = ?e, "MAL fuzzy search failed"),
            }
        }

        for (tracker, tracker_id) in candidates {
            if let Some(full) = ContentResolverService::link_or_enrich_tracker(
                state, ext_name, ext_id, ext_nsfw, &tracker, &tracker_id, content_type,
            ).await? {
                return Ok(Some(full));
            }
        }

        Ok(None)
    }

    #[instrument(skip(state, meta))]
    pub async fn update_content(
        state: &Arc<AppState>,
        cid: &str,
        meta: Metadata,
    ) -> CoreResult<FullContent> {
        info!(cid = %cid, source = %meta.source_name, "Updating content metadata");
        ContentRepository::upsert_metadata(&state.pool, &meta).await?;

        ContentRepository::get_full_content(&state.pool, cid).await?
            .ok_or_else(|| {
                warn!(cid = %cid, "Content not found after metadata update");
                CoreError::NotFound("error.content.not_found".into())
            })
    }
}