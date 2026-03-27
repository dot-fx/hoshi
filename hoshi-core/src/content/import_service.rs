use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use rusqlite::Connection;
use serde_json::{json, Value};
use tracing::{info, warn, error, debug, instrument};

use crate::config::repository::ConfigRepo;
use crate::content::{
    CacheRepository, ContentRepository, ContentType, EpisodeData, ContentMetadata,
    ContentRelation, ContentUnitRepository, RelationRepository, RelationType,
    generate_cid,
};
use crate::db::DatabaseManager;
use crate::error::{CoreError, CoreResult};
use crate::tracker::repository::{TrackerMapping, TrackerRepository};
use crate::tracker::provider::{TrackerMedia, TrackerRegistry};
use crate::tracker::provider::ContentType as TrackerContentType;
use crate::tracker::provider::simkl::SimklProvider;

use super::types::{HomeView, MediaSection};

const HOME_CACHE_KEY: &str = "home_view_v2";
const HOME_CACHE_TTL: i64  = 6 * 3600;
static HOME_REFRESHING: AtomicBool = AtomicBool::new(false);

pub struct ContentImportService;

impl ContentImportService {

    #[instrument(skip(db_manager, registry))]
    async fn refresh_home_cache(
        db_manager: Arc<DatabaseManager>,
        registry: Arc<TrackerRegistry>,
    ) -> CoreResult<()> {
        debug!("Refreshing home view cache from AniList");

        let provider = registry
            .get("anilist")
            .ok_or_else(|| {
                error!("AniList provider not found in registry");
                CoreError::Internal("error.tracker.anilist_not_registered".into())
            })?;

        let sections = provider.get_home().await?;
        let db = db_manager.connection();

        let mut enrich = |section_key: &str| -> CoreResult<Vec<Value>> {
            let items = sections.get(section_key).cloned().unwrap_or_default();
            let mut enriched = Vec::with_capacity(items.len());

            for media in &items {
                let cid = {
                    let conn = db.lock().map_err(|_| CoreError::Internal("error.system.db_lock".into()))?;
                    Self::import_media(&conn, "anilist", media)?
                };

                let mut item_json = serde_json::to_value(media).unwrap_or(json!({}));
                item_json["cid"] = json!(cid);
                enriched.push(item_json);
            }
            Ok(enriched)
        };

        let now = chrono::Utc::now().timestamp();

        let view = HomeView {
            anime: MediaSection {
                trending: enrich("trending_anime")?,
                top_rated: enrich("top_rated_anime")?,
                seasonal: Some(enrich("seasonal_anime")?),
            },
            manga: MediaSection {
                trending: enrich("trending_manga")?,
                top_rated: enrich("top_rated_manga")?,
                seasonal: None,
            },
            novel: MediaSection {
                trending: enrich("trending_novel")?,
                top_rated: enrich("top_rated_novel")?,
                seasonal: None,
            },
            cached_at: now,
        };

        let value = serde_json::to_value(&view)
            .map_err(|e| {
                error!(error = ?e, "Failed to serialize home view");
                CoreError::Internal("error.content.home_cache_error".into())
            })?;

        let conn = db.lock().map_err(|_| CoreError::Internal("error.system.db_lock".into()))?;

        let retention_period = 30 * 24 * 3600;
        CacheRepository::set(&conn, HOME_CACHE_KEY, "anilist", "home", &value, retention_period)?;

        info!("Home view cache updated successfully");
        Ok(())
    }

    #[instrument(skip(db_manager, registry))]
    pub async fn get_home_view(
        db_manager: Arc<DatabaseManager>,
        registry: Arc<TrackerRegistry>,
        user_id: i32,
    ) -> CoreResult<Value> {
        let show_adult = {
            let db_arc = db_manager.connection();
            let Ok(conn) = db_arc.lock() else { return Ok(json!({})) };
            ConfigRepo::get_config(&conn, user_id)
                .map(|c| c.general.show_adult_content)
                .unwrap_or(false)
        };

        {
            let db_arc = db_manager.connection();
            let conn = db_arc.lock().map_err(|_| CoreError::Internal("error.system.db_lock".into()))?;

            if let Some(cached) = CacheRepository::get(&conn, HOME_CACHE_KEY)? {
                debug!("Home view cache hit");

                let cached_at = cached["cached_at"].as_i64().unwrap_or(0);
                let expired = chrono::Utc::now().timestamp() - cached_at > HOME_CACHE_TTL;

                if expired && !HOME_REFRESHING.swap(true, Ordering::SeqCst) {
                    info!("Home cache expired, triggering background refresh");
                    let db_clone  = db_manager.clone();
                    let reg_clone = registry.clone();

                    tokio::spawn(async move {
                        if let Err(e) = Self::refresh_home_cache(db_clone, reg_clone).await {
                            warn!(error = ?e, "Background home refresh failed");
                        }
                        HOME_REFRESHING.store(false, Ordering::SeqCst);
                    });
                }

                return Ok(if show_adult { cached } else { Self::filter_home_nsfw(cached) });
            }
        }

        info!("No home cache found, performing initial fetch");
        Self::refresh_home_cache(db_manager.clone(), registry.clone()).await?;

        let db_arc = db_manager.connection();
        let conn = db_arc.lock().map_err(|_| CoreError::Internal("error.system.db_lock".into()))?;

        let value = CacheRepository::get(&conn, HOME_CACHE_KEY)?
            .ok_or_else(|| {
                error!("Home cache missing immediately after refresh");
                CoreError::Internal("error.content.home_cache_missing".into())
            })?;

        Ok(if show_adult { value } else { Self::filter_home_nsfw(value) })
    }

    #[instrument(skip(db_manager, registry))]
    pub async fn get_trending(
        db_manager: Arc<DatabaseManager>,
        registry: Arc<TrackerRegistry>,
        media_type: &str,
        user_id: i32,
    ) -> CoreResult<Value> {
        let cache_key = format!("trending_{}_v2", media_type);

        let show_adult = {
            let db_arc = db_manager.connection();
            let Ok(conn) = db_arc.lock() else { return Ok(json!([])) };
            ConfigRepo::get_config(&conn, user_id)
                .map(|c| c.general.show_adult_content)
                .unwrap_or(false)
        };

        {
            let db_arc = db_manager.connection();
            let conn = db_arc.lock().map_err(|_| CoreError::Internal("error.system.db_lock".into()))?;
            if let Some(cached) = CacheRepository::get(&conn, &cache_key)? {
                return Ok(if show_adult { cached } else { Self::filter_array_nsfw(cached) });
            }
        }

        {
            let db_arc = db_manager.connection();
            let conn = db_arc.lock().map_err(|_| CoreError::Internal("error.system.db_lock".into()))?;
            if let Some(home) = CacheRepository::get(&conn, HOME_CACHE_KEY)? {
                if let Some(section) = home.get(media_type) {
                    if let Some(trending) = section.get("trending") {
                        let val = trending.clone();
                        return Ok(if show_adult { val } else { Self::filter_array_nsfw(val) });
                    }
                }
            }
        }

        debug!(media_type = %media_type, "Trending not in cache, fetching from AniList");
        let provider = registry.get("anilist")
            .ok_or_else(|| CoreError::Internal("error.tracker.anilist_not_registered".into()))?;

        let sections = provider.get_home().await?;
        let db = db_manager.connection();

        let section_key = format!("trending_{}", media_type);
        let items = sections.get(&section_key).cloned().unwrap_or_default();

        let mut enriched = Vec::with_capacity(items.len());
        for media in &items {
            let cid = {
                let conn = db.lock().map_err(|_| CoreError::Internal("error.system.db_lock".into()))?;
                Self::import_media(&conn, "anilist", media)?
            };
            let mut item_json = serde_json::to_value(media).unwrap_or(json!({}));
            item_json["cid"] = json!(cid);
            enriched.push(item_json);
        }

        let value = json!(enriched);

        {
            let db_arc = db_manager.connection();
            let conn = db_arc.lock().map_err(|_| CoreError::Internal("error.system.db_lock".into()))?;
            let _ = CacheRepository::set(&conn, &cache_key, "anilist", "trending", &value, HOME_CACHE_TTL);
        }

        Ok(if show_adult { value } else { Self::filter_array_nsfw(value) })
    }

    #[instrument(skip(db, registry, params))]
    pub async fn search_and_import(
        db: Arc<std::sync::Mutex<Connection>>,
        registry: Arc<TrackerRegistry>,
        params: &super::types::SearchParams,
    ) -> CoreResult<Vec<String>> {
        let content_type = super::types::parse_content_type(
            params.r#type.as_deref().unwrap_or("anime")
        );

        let tracker_name = match params.tracker.as_deref().unwrap_or("anilist") {
            "mal"   => "mal",
            "kitsu" => "kitsu",
            _       => "anilist",
        };

        let provider = registry.get(tracker_name)
            .or_else(|| registry.get("anilist"))
            .ok_or_else(|| {
                error!(tracker = %tracker_name, "No suitable search provider found");
                CoreError::Internal("error.tracker.no_provider_available".into())
            })?;

        let actual_tracker = provider.name();
        debug!(tracker = %actual_tracker, query = ?params.query, "Executing search and import");

        let results = provider.search(
            params.query.as_deref(),
            content_type,
            params.limit.min(50) as usize,
            params.sort.as_deref(),
            params.genre.as_deref(),
            params.format.as_deref(),
            params.nsfw,
        ).await?;

        let mut imported = Vec::new();
        for media in &results {
            let conn = db.lock().map_err(|_| CoreError::Internal("error.system.db_lock".into()))?;
            match Self::import_media(&conn, actual_tracker, media) {
                Ok(cid) => {
                    if !imported.contains(&cid) { imported.push(cid); }
                },
                Err(e)  => error!(id = %media.tracker_id, error = ?e, "Failed to import search result"),
            }
        }

        Ok(imported)
    }

    #[instrument(skip(db, registry))]
    pub async fn enrich_with_simkl(
        db: Arc<std::sync::Mutex<Connection>>,
        registry: Arc<TrackerRegistry>,
        cid: &str,
    ) -> CoreResult<()> {
        let simkl = match registry.get("simkl") {
            Some(p) => p,
            None => {
                debug!("Simkl provider not available, skipping enrichment");
                return Ok(());
            }
        };

        let (existing_mappings, mut meta) = {
            let conn = db.lock().map_err(|_| CoreError::Internal("error.system.db_lock".into()))?;
            let mappings = TrackerRepository::get_mappings_by_cid(&conn, cid)?;
            let meta = ContentRepository::get_by_cid(&conn, cid)?
                .ok_or_else(|| CoreError::NotFound("error.content.metadata_not_found".into()))?;
            (mappings, meta)
        };

        let mut simkl_id = existing_mappings.iter()
            .find(|m| m.tracker_name == "simkl")
            .map(|m| m.tracker_id.clone());

        if simkl_id.is_none() {
            let lookup = ["anilist", "myanimelist", "kitsu"].iter().find_map(|&t| {
                existing_mappings.iter()
                    .find(|m| m.tracker_name == t)
                    .map(|m| (if t == "myanimelist" { "mal" } else { t }, m.tracker_id.clone()))
            });

            let Some((id_type, id_val)) = lookup else { return Ok(()) };

            debug!(type = %id_type, id = %id_val, "Searching for Simkl ID via cross-reference");
            let simkl_provider = SimklProvider::new();
            if let Ok(Some(found)) = simkl_provider
                .find_by_cross_id(id_type, &id_val, TrackerContentType::Anime)
                .await
            {
                simkl_id = Some(found.tracker_id);
            }
        }

        let Some(id) = simkl_id else { return Ok(()) };

        let simkl_media = match simkl.get_by_id(&id).await {
            Ok(Some(m)) => m,
            _ => {
                warn!(simkl_id = %id, "Simkl ID found but data could not be retrieved");
                return Ok(());
            }
        };

        let mut external_ids_map = match meta.external_ids.clone() {
            Value::Object(map) => map,
            _ => serde_json::Map::new(),
        };

        let allowed_trackers = ["myanimelist", "anilist", "kitsu", "anidb", "imdb", "livechart", "trakt", "animeplanet"];
        let now = chrono::Utc::now().timestamp();
        let mut new_mappings: Vec<TrackerMapping> = Vec::new();

        if !existing_mappings.iter().any(|m| m.tracker_name == "simkl") {
            new_mappings.push(TrackerMapping {
                cid: cid.to_string(), tracker_name: "simkl".to_string(), tracker_id: id.clone(),
                tracker_url: Some(format!("https://simkl.com/anime/{}", id)),
                sync_enabled: true, last_synced: Some(now), created_at: now, updated_at: now,
            });
        }

        for (key, val) in &simkl_media.cross_ids {
            let tracker_name = match key.as_str() {
                "mal" => "myanimelist",
                "ann" => "animenewsnetwork",
                "trakttv" | "trakttvslug" => "trakt",
                other => other,
            };

            if allowed_trackers.contains(&tracker_name) || tracker_name == "simkl" {
                if !existing_mappings.iter().any(|m| m.tracker_name == tracker_name) {
                    let url = match tracker_name {
                        "anidb"     => Some(format!("https://anidb.net/anime/{}", val)),
                        "kitsu"     => Some(format!("https://kitsu.io/anime/{}", val)),
                        "imdb"      => Some(format!("https://www.imdb.com/title/{}/", val)),
                        "livechart" => Some(format!("https://www.livechart.me/anime/{}", val)),
                        "trakt"     => Some(format!("https://trakt.tv/shows/{}", val)),
                        _           => None,
                    };
                    new_mappings.push(TrackerMapping {
                        cid: cid.to_string(), tracker_name: tracker_name.to_string(), tracker_id: val.to_string(),
                        tracker_url: url, sync_enabled: false, last_synced: None, created_at: now, updated_at: now,
                    });
                }
                external_ids_map.remove(tracker_name);
                external_ids_map.remove(key);
            } else {
                external_ids_map.insert(key.clone(), Value::String(val.clone()));
            }
        }

        meta.external_ids = Value::Object(external_ids_map);

        if meta.synopsis.is_none()    { meta.synopsis    = simkl_media.synopsis; }
        if meta.trailer_url.is_none() { meta.trailer_url = simkl_media.trailer_url; }
        if meta.cover_image.is_none() { meta.cover_image = simkl_media.cover_image; }
        if meta.rating.is_none()      { meta.rating      = simkl_media.rating; }

        let episodes = SimklProvider::new().get_episodes(&id).await.ok();

        {
            let conn = db.lock().map_err(|_| CoreError::Internal("error.system.db_lock".into()))?;
            for mapping in &new_mappings { let _ = TrackerRepository::add_mapping(&conn, mapping); }
            if let Some(eps) = episodes {
                for ep_json in eps {
                    if let Err(e) = ContentUnitRepository::upsert(&conn, cid, &ep_json) {
                        error!(cid = %cid, error = ?e, "Failed to upsert Simkl episode unit");
                    }
                }
            }
            let _ = ContentRepository::upsert_metadata(&conn, &meta);
        }

        info!(cid = %cid, "Simkl metadata enrichment completed");
        Ok(())
    }

    #[instrument(skip(conn, media))]
    pub fn import_media(
        conn: &Connection,
        tracker_name: &str,
        media: &TrackerMedia,
    ) -> CoreResult<String> {
        let is_full = media.synopsis.is_some() || !media.characters.is_empty();

        let cid = if let Some(cid) = TrackerRepository::find_cid_by_tracker(conn, tracker_name, &media.tracker_id)? {
            if is_full {
                let meta = Self::to_content_metadata(&cid, tracker_name, media);
                ContentRepository::upsert_metadata(conn, &meta)?;
            }
            cid
        } else {
            let mut found_cross = None;
            for (cross_tracker, cross_id) in &media.cross_ids {
                if let Some(cid) = TrackerRepository::find_cid_by_tracker(conn, cross_tracker, cross_id)? {
                    match ContentRepository::get_content_by_cid(conn, &cid)? {
                        Some(existing) if existing.content_type == media.content_type => {
                            found_cross = Some(cid);
                            break;
                        }
                        Some(existing) => {
                            warn!(tracker = %cross_tracker, id = %cross_id, cid = %cid,
                                "Cross-id type mismatch (DB: {:?}, Import: {:?})",
                                existing.content_type, media.content_type);
                        }
                        None => {}
                    }
                }
            }

            if let Some(cid) = found_cross {
                debug!(cid = %cid, tracker = %tracker_name, "Linking existing CID via cross-id");
                Self::add_mapping(conn, &cid, tracker_name, &media.tracker_id,
                                  &Self::tracker_url(tracker_name, &media.tracker_id, &media.content_type))?;
                Self::add_cross_mappings(conn, &cid, &media.cross_ids, tracker_name, &media.content_type)?;
                if is_full {
                    let meta = Self::to_content_metadata(&cid, tracker_name, media);
                    ContentRepository::upsert_metadata(conn, &meta)?;
                }
                cid
            } else {
                let new_cid = generate_cid();
                info!(cid = %new_cid, title = %media.title, "Creating new canonical content entry");
                let meta = Self::to_content_metadata(&new_cid, tracker_name, media);
                ContentRepository::create_with_type(conn, &media.content_type, media.nsfw, meta)?;
                Self::add_mapping(conn, &new_cid, tracker_name, &media.tracker_id,
                                  &Self::tracker_url(tracker_name, &media.tracker_id, &media.content_type))?;
                Self::add_cross_mappings(conn, &new_cid, &media.cross_ids, tracker_name, &media.content_type)?;
                new_cid
            }
        };

        if is_full {
            for rel in &media.relations {
                match Self::import_media(conn, tracker_name, &rel.media) {
                    Ok(target_cid) => {
                        let rel_enum = match rel.relation_type.as_str() {
                            "SEQUEL"     => RelationType::Sequel,
                            "PREQUEL"    => RelationType::Prequel,
                            "SIDE_STORY" => RelationType::SideStory,
                            "SPIN_OFF"   => RelationType::Spinoff,
                            "ADAPTATION" => RelationType::Adaptation,
                            "PARENT"     => RelationType::Parent,
                            "SUMMARY"    => RelationType::Summary,
                            _            => RelationType::Alternative,
                        };
                        let _ = RelationRepository::upsert(conn, &ContentRelation {
                            id: None, source_cid: cid.clone(), target_cid,
                            relation_type: rel_enum, source_name: tracker_name.to_string(),
                            created_at: chrono::Utc::now().timestamp(),
                        });
                    }
                    Err(e) => warn!(error = ?e, "Failed to import related media"),
                }
            }
        }

        Ok(cid)
    }

    pub fn add_mapping(conn: &Connection, cid: &str, tracker: &str, id: &str, url: &str) -> CoreResult<()> {
        let now = chrono::Utc::now().timestamp();
        TrackerRepository::add_mapping(conn, &TrackerMapping {
            cid:          cid.to_string(),
            tracker_name: tracker.to_string(),
            tracker_id:   id.to_string(),
            tracker_url:  Some(url.to_string()),
            sync_enabled: true,
            last_synced:  Some(now),
            created_at:   now,
            updated_at:   now,
        })
    }

    pub fn add_cross_mappings(
        conn: &Connection,
        cid: &str,
        cross_ids: &std::collections::HashMap<String, String>,
        skip_tracker: &str,
        content_type: &ContentType,
    ) -> CoreResult<()> {
        for (tracker, id) in cross_ids {
            if tracker == skip_tracker { continue; }
            if TrackerRepository::find_cid_by_tracker(conn, tracker, id)?.is_none() {
                let _ = Self::add_mapping(conn, cid, tracker, id, &Self::tracker_url(tracker, id, content_type));
            }
        }
        Ok(())
    }

    pub fn tracker_url(tracker: &str, id: &str, content_type: &ContentType) -> String {
        let type_path = match content_type {
            ContentType::Manga | ContentType::Novel => "manga",
            _ => "anime",
        };
        match tracker {
            "anilist"     => format!("https://anilist.co/{}/{}", type_path, id),
            "myanimelist" => format!("https://myanimelist.net/{}/{}", type_path, id),
            "simkl"       => format!("https://simkl.com/{}/{}", type_path, id),
            _             => String::new(),
        }
    }

    pub fn to_content_metadata(cid: &str, tracker_name: &str, media: &TrackerMedia) -> ContentMetadata {
        use crate::content::ContentStatus;
        let now   = chrono::Utc::now().timestamp();
        let count = match media.content_type {
            TrackerContentType::Anime => media.episode_count.unwrap_or(0),
            _                         => media.chapter_count.unwrap_or(0),
        };

        let status = media.status.as_deref().map(|s| match s {
            "FINISHED" | "ended" | "completed"   => ContentStatus::Completed,
            "RELEASING" | "ongoing" | "airing"   => ContentStatus::Ongoing,
            "NOT_YET_RELEASED" | "planned"       => ContentStatus::Planned,
            "CANCELLED" | "canceled"             => ContentStatus::Cancelled,
            "HIATUS"                             => ContentStatus::Hiatus,
            _                                    => ContentStatus::Ongoing,
        });

        ContentMetadata {
            id: None, cid: cid.to_string(), source_name: tracker_name.to_string(),
            source_id: Some(media.tracker_id.clone()), subtype: media.format.clone(),
            title: media.title.clone(), alt_titles: media.alt_titles.clone(),
            title_i18n: media.title_i18n.clone(), synopsis: media.synopsis.clone(),
            cover_image: media.cover_image.clone(), banner_image: media.banner_image.clone(),
            eps_or_chapters: EpisodeData::Count(count), status, tags: media.tags.clone(),
            genres: media.genres.clone(), release_date: media.release_date.clone(),
            end_date: media.end_date.clone(), rating: media.rating,
            trailer_url: media.trailer_url.clone(), characters: media.characters.clone(),
            studio: media.studio.clone(), staff: media.staff.clone(),
            external_ids: json!({}), created_at: now, updated_at: now,
        }
    }

    fn filter_home_nsfw(mut view: Value) -> Value {
        if let Some(obj) = view.as_object_mut() {
            for section_key in ["anime", "manga", "novel"] {
                if let Some(section) = obj.get_mut(section_key).and_then(|s| s.as_object_mut()) {
                    for list_key in ["trending", "topRated", "seasonal"] {
                        if let Some(arr) = section.get_mut(list_key).and_then(|v| v.as_array_mut()) {
                            arr.retain(|item| !item.get("nsfw").and_then(|v| v.as_bool()).unwrap_or(false));
                        }
                    }
                }
            }
        }
        view
    }

    fn filter_array_nsfw(value: Value) -> Value {
        if let Value::Array(mut arr) = value {
            arr.retain(|item| !item.get("nsfw").and_then(|v| v.as_bool()).unwrap_or(false));
            Value::Array(arr)
        } else {
            value
        }
    }
}