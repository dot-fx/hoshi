use std::sync::Arc;
use std::collections::HashSet;
use tracing::{debug, info, instrument, warn};
use crate::config::model::TitleLanguage;
use crate::config::repository::ConfigRepository;
use crate::content::models::{ContentType, FullContent, Metadata};
use crate::content::repositories::content::ContentRepository;
use crate::content::repositories::extension::ExtensionRepository;
use crate::content::services::chinese_title::ChineseTitleService;
use crate::content::services::content_units::SimklUnitsService;
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
        let mut full = if TRACKER_SOURCES.contains(&source) {
            Self::resolve_tracker_source(state, source, source_id).await?
        } else {
            Self::resolve_extension_source(state, source, source_id).await?
        };

        Self::maybe_inject_chinese_title(state, &mut full).await;
        Ok(full)
    }

    #[instrument(skip(state))]
    pub async fn get_content_by_cid(
        state: &Arc<AppState>,
        cid: &str,
    ) -> CoreResult<FullContent> {
        let mut full = ContentRepository::get_full_content(&state.pool, cid).await?
            .ok_or_else(|| CoreError::NotFound("error.content.not_found".into()))?;

        Self::maybe_inject_chinese_title(state, &mut full).await;
        Ok(full)
    }

    async fn resolve_tracker_source(
        state: &Arc<AppState>,
        tracker: &str,
        tracker_id: &str,
    ) -> CoreResult<FullContent> {
        let maybe_cid = TrackerRepository::find_cid_by_tracker(&state.pool, tracker, tracker_id).await?;

        if let Some(cid) = maybe_cid {
            debug!(cid = %cid, tracker = %tracker, "CID found via tracker mapping");

            let full = ContentResolverService::load_full_content(state, &cid).await?;
            Self::backfill_preferred_metadata(state, &cid, &full, tracker, tracker_id).await?;

            if let Err(e) = SimklUnitsService::sync_units_if_needed(state, &cid).await {
                warn!(cid = %cid, error = ?e, "Simkl unit sync failed, continuing");
            }

            return ContentResolverService::load_full_content(state, &cid).await;
        }

        info!(tracker = %tracker, id = %tracker_id, "No existing CID, enriching from tracker");
        let media = ContentResolverService::fetch_tracker_media(state, tracker, tracker_id).await?;
        let full = EnrichmentService::create_enriched_content(
            state, &media.content_type, &media, tracker_id, tracker, None,
        ).await?;

        if let Err(e) = SimklUnitsService::sync_units_if_needed(state, &full.content.cid).await {
            warn!(cid = %full.content.cid, error = ?e, "Simkl unit sync failed after enrichment, continuing");
        }
        Ok(full)
    }

    async fn backfill_preferred_metadata(
        state: &Arc<AppState>,
        cid: &str,
        full: &FullContent,
        current_tracker: &str,
        current_tracker_id: &str,
    ) -> CoreResult<()> {
        let config = ConfigRepository::get_config(&state.pool, 1).await?;
        let preferred = &config.content.preferred_metadata_provider;

        let already_has = full.metadata.iter().any(|m| &m.source_name == preferred);

        if already_has {
            let needs_character_refresh = full.metadata.iter()
                .find(|m| &m.source_name == preferred)
                .map(|m| m.characters.is_empty())
                .unwrap_or(false);

            if !needs_character_refresh {
                return Ok(());
            }

            info!(cid = %cid, preferred = %preferred, "Metadata missing characters, refreshing via get_by_id");

            let tid = if preferred == current_tracker {
                Some(current_tracker_id.to_string())
            } else {
                TrackerRepository::find_tracker_id_by_cid(&state.pool, cid, preferred).await?
            };

            let Some(tid) = tid else {
                warn!(cid = %cid, "No tracker mapping for preferred provider, skipping character refresh");
                return Ok(());
            };

            let media = match ContentResolverService::fetch_tracker_media(state, preferred, &tid).await {
                Ok(m) => m,
                Err(e) => {
                    warn!(error = ?e, "Failed to refresh characters, skipping");
                    return Ok(());
                }
            };

            let provider = state.tracker_registry.get(preferred)
                .ok_or_else(|| CoreError::NotFound(format!("Tracker provider '{}' not found", preferred)))?;
            let meta = provider.to_core_metadata(cid, &media);
            ContentRepository::upsert_metadata(&state.pool, &meta).await?;

            return Ok(());
        }

        info!(cid = %cid, preferred = %preferred, "Missing preferred provider metadata, backfilling");

        let preferred_tracker_id = if preferred == current_tracker {
            Some(current_tracker_id.to_string())
        } else {
            TrackerRepository::find_tracker_id_by_cid(&state.pool, cid, preferred).await?
        };

        let Some(tid) = preferred_tracker_id else {
            warn!(cid = %cid, preferred = %preferred, "No tracker mapping found for preferred provider, skipping backfill");
            return Ok(());
        };

        let media = match ContentResolverService::fetch_tracker_media(state, preferred, &tid).await {
            Ok(m) => m,
            Err(e) => {
                warn!(error = ?e, "Failed to fetch preferred provider metadata, skipping backfill");
                return Ok(());
            }
        };

        let provider = state.tracker_registry.get(preferred)
            .ok_or_else(|| CoreError::NotFound(format!("Tracker provider '{}' not found", preferred)))?;

        let meta = provider.to_core_metadata(cid, &media);
        ContentRepository::upsert_metadata(&state.pool, &meta).await?;

        Ok(())
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

    async fn maybe_inject_chinese_title(state: &Arc<AppState>, full: &mut FullContent) {
        let config = match ConfigRepository::get_config(&state.pool, 1).await {
            Ok(c) => c,
            Err(e) => {
                warn!(error = ?e, "Could not read user config for Chinese title check");
                return;
            }
        };

        if !matches!(config.ui.title_language, TitleLanguage::Chinese) {
            ChineseTitleService::evict().await;
            return;
        }

        ChineseTitleService::ensure_loaded().await;

        let anilist_id: u32 = match full
            .tracker_mappings
            .iter()
            .find(|m| m.tracker_name == "anilist")
        {
            Some(m) => match m.tracker_id.parse() {
                Ok(id) => id,
                Err(_) => {
                    warn!(tracker_id = %m.tracker_id, "AniList tracker_id is not a valid u32, skipping Chinese title");
                    return;
                }
            },
            None => return,
        };

        let Some(chinese_title) = ChineseTitleService::lookup(anilist_id).await else {
            return;
        };

        for meta in &mut full.metadata {
            meta.title_i18n.insert("chinese".into(), chinese_title.clone());
        }
    }
}