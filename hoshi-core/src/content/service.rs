use std::sync::Arc;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::content::repository::{ContentRepository, ContentStatus, ContentType, ContentWithMappings, CoreMetadata, EpisodeData, ExtensionRepository, ExtensionSource, CacheRepository, generate_cid, RelationType, ContentRelation, RelationRepository};
use crate::tracker::repository::{TrackerMapping, TrackerRepository};
use crate::content::resolver::ContentResolverService;
use crate::db::DatabaseManager;
use crate::error::{CoreError, CoreResult};
use crate::extensions::ExtensionType;
use crate::state::AppState;
use crate::tracker::provider::{ContentType as TrackerContentType, TrackerMedia, TrackerRegistry};

#[derive(Debug, Clone)]
pub struct SearchParams {
    pub r#type: Option<String>,
    pub nsfw: Option<bool>,
    pub status: Option<String>,
    pub query: Option<String>,
    pub limit: i32,
    pub offset: i32,
    pub extension: Option<String>,
    pub sort: Option<String>,
    pub genre: Option<String>,
    pub format: Option<String>,
    pub extension_filters: Option<String>,
}

#[derive(Debug)]
pub struct ContentListResult {
    pub data: Vec<ContentWithMappings>,
    pub total: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentResponse {
    pub success: bool,
    pub data: ContentWithMappings,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentListResponse {
    pub success: bool,
    pub data: Vec<ContentWithMappings>,
    pub total: usize,
    pub limit: i32,
    pub offset: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HomeResponse {
    pub success: bool,
    pub data: Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemsResponse {
    pub success: bool,
    pub data: Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayResponse {
    pub success: bool,
    #[serde(rename = "type")]
    pub play_type: Value,
    pub data: Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SuccessResponse {
    pub success: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SuccessWithIdResponse {
    pub success: bool,
    pub id: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionSearchResponse {
    pub success: bool,
    pub results: Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateContentRequest {
    pub content: CoreMetadata,
    pub tracker_mappings: Option<Vec<TrackerMapping>>,
    pub extension_sources: Option<Vec<ExtensionSource>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchQuery {
    pub r#type: Option<String>,
    pub nsfw: Option<bool>,
    pub status: Option<String>,
    pub query: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub extension: Option<String>,
    pub sort: Option<String>,
    pub genre: Option<String>,
    pub format: Option<String>,
    pub extension_filters: Option<String>,
}

impl SearchQuery {
    pub fn into_params(self) -> SearchParams {
        SearchParams {
            r#type: self.r#type,
            nsfw: self.nsfw,
            status: self.status,
            query: self.query,
            limit: self.limit.unwrap_or(20),
            offset: self.offset.unwrap_or(0),
            extension: self.extension,
            sort: self.sort,
            genre: self.genre,
            format: self.format,
            extension_filters: self.extension_filters,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateExtensionMappingRequest {
    pub extension_name: String,
    pub extension_id: String,
    pub metadata: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTrackerMappingRequest {
    pub tracker_name: String,
    pub tracker_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceQuery {
    pub server: Option<String>,
    pub category: Option<String>,
}

pub fn parse_content_type(t: &str) -> TrackerContentType {
    match t {
        "manga"  => TrackerContentType::Manga,
        "novel"  => TrackerContentType::Novel,
        "booru"  => TrackerContentType::Booru,
        _        => TrackerContentType::Anime,
    }
}

pub struct ContentImportService;

impl ContentImportService {
    pub async fn get_home_view(
        db_manager: Arc<DatabaseManager>,
        registry: Arc<TrackerRegistry>,
    ) -> CoreResult<Value> {
        let provider = registry.get("anilist")
            .ok_or_else(|| CoreError::Internal("AniList provider not registered".into()))?;

        let sections = provider.get_home().await?;
        let db = db_manager.connection();
        let mut result = serde_json::Map::new();

        for (section_key, items) in sections {
            let mut enriched = Vec::new();
            for media in &items {
                let cid = {
                    let conn = db.lock().map_err(|_| CoreError::Internal("DB Lock".into()))?;
                    Self::import_media(&conn, "anilist", media)?
                };
                let mut item_json = serde_json::to_value(media).unwrap_or(json!({}));
                item_json["cid"] = json!(cid);
                enriched.push(item_json);
            }
            result.insert(section_key, json!(enriched));
        }

        Ok(Value::Object(result))
    }
    
    pub async fn search_and_import(
        db: Arc<std::sync::Mutex<Connection>>,
        registry: Arc<TrackerRegistry>,
        params: &SearchParams,
    ) -> CoreResult<Vec<String>> {
        if params.r#type.as_deref() == Some("booru") {
            return Ok(vec![]);
        }

        let content_type = parse_content_type(params.r#type.as_deref().unwrap_or("anime"));

        let provider = registry.get("anilist")
            .ok_or_else(|| CoreError::Internal("No search provider available".into()))?;

        let results = provider.search(
            params.query.as_deref(),
            content_type,
            params.limit.min(50) as usize,
            params.sort.as_deref(),
            params.genre.as_deref(),
            params.format.as_deref(),
        ).await?;

        let mut imported = Vec::new();
        for media in &results {
            let conn = db.lock().map_err(|_| CoreError::Internal("DB Lock".into()))?;
            match Self::import_media(&conn, "anilist", media) {
                Ok(cid) => imported.push(cid),
                Err(e) => tracing::error!("Failed to import media {}: {}", media.tracker_id, e),
            }
        }

        Ok(imported)
    }

    pub async fn enrich_with_simkl(
        db: Arc<std::sync::Mutex<Connection>>,
        registry: Arc<TrackerRegistry>,
        cid: &str,
    ) -> CoreResult<()> {
        let simkl = match registry.get("simkl") {
            Some(p) => p,
            None => return Ok(()),
        };

        let (mut existing_mappings, mut meta) = {
            let conn = db.lock().unwrap();
            let mappings = TrackerRepository::get_mappings_by_cid(&conn, cid)?;
            let meta = ContentRepository::get_by_cid(&conn, cid)?
                .ok_or_else(|| CoreError::NotFound("Content not found".into()))?;
            (mappings, meta)
        };

        let mut simkl_id = existing_mappings.iter()
            .find(|m| m.tracker_name == "simkl")
            .map(|m| m.tracker_id.clone());

        if simkl_id.is_none() {
            let al_id = existing_mappings.iter().find(|m| m.tracker_name == "anilist").map(|m| m.tracker_id.clone());
            let mal_id = existing_mappings.iter().find(|m| m.tracker_name == "myanimelist").map(|m| m.tracker_id.clone());

            if al_id.is_none() && mal_id.is_none() {
                return Ok(());
            }

            let id_type = if al_id.is_some() { "anilist" } else { "mal" };
            let id_val = al_id.as_deref().or(mal_id.as_deref()).unwrap();

            if let Ok(mut results) = simkl.search(
                Some(&format!("id:{}:{}", id_type, id_val)),
                TrackerContentType::Anime,
                1, None, None, None
            ).await {
                if !results.is_empty() {
                    simkl_id = Some(results.remove(0).tracker_id);
                }
            }
        }

        let simkl_id = match simkl_id {
            Some(id) => id,
            None => return Ok(()),
        };

        let simkl_media = match simkl.get_by_id(&simkl_id).await {
            Ok(Some(m)) => m,
            _ => return Ok(()),
        };

        let now = chrono::Utc::now().timestamp();
        let conn = db.lock().map_err(|_| CoreError::Internal("DB Lock".into()))?;

        if !existing_mappings.iter().any(|m| m.tracker_name == "simkl") {
            let _ = TrackerRepository::add_mapping(&conn, &TrackerMapping {
                cid: cid.to_string(),
                tracker_name: "simkl".to_string(),
                tracker_id: simkl_id.clone(),
                tracker_url: Some(format!("https://simkl.com/anime/{}", simkl_id)),
                sync_enabled: true,
                last_synced: Some(now),
                created_at: now,
                updated_at: now,
            });
        }

        let mut external_ids_map = match meta.external_ids.clone() {
            Value::Object(map) => map,
            _ => serde_json::Map::new(),
        };

        let allowed_trackers = ["myanimelist", "anilist", "kitsu", "anidb", "imdb", "livechart", "trakt", "animeplanet"];

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
                        "anidb" => Some(format!("https://anidb.net/anime/{}", val)),
                        "kitsu" => Some(format!("https://kitsu.io/anime/{}", val)),
                        "imdb" => Some(format!("https://www.imdb.com/title/{}/", val)),
                        "livechart" => Some(format!("https://www.livechart.me/anime/{}", val)),
                        "trakt" => Some(format!("https://trakt.tv/shows/{}", val)),
                        _ => None
                    };

                    let _ = TrackerRepository::add_mapping(&conn, &TrackerMapping {
                        cid: cid.to_string(),
                        tracker_name: tracker_name.to_string(),
                        tracker_id: val.to_string(),
                        tracker_url: url,
                        sync_enabled: false,
                        last_synced: None,
                        created_at: now,
                        updated_at: now,
                    });

                    existing_mappings.push(TrackerMapping {
                        cid: cid.to_string(), tracker_name: tracker_name.to_string(), tracker_id: val.to_string(),
                        tracker_url: None, sync_enabled: false, last_synced: None, created_at: now, updated_at: now,
                    });
                }

                external_ids_map.remove(tracker_name);
                external_ids_map.remove(key);
            } else {
                external_ids_map.insert(key.clone(), Value::String(val.clone()));
            }
        }

        meta.external_ids = Value::Object(external_ids_map);

        if meta.synopsis.is_none() && simkl_media.synopsis.is_some() {
            meta.synopsis = simkl_media.synopsis;
        }
        if meta.trailer_url.is_none() && simkl_media.trailer_url.is_some() {
            meta.trailer_url = simkl_media.trailer_url;
        }
        if meta.cover_image.is_none() && simkl_media.cover_image.is_some() {
            meta.cover_image = simkl_media.cover_image;
        }
        if meta.rating.is_none() && simkl_media.rating.is_some() {
            meta.rating = simkl_media.rating;
        }

        let _ = ContentRepository::update(&conn, cid, &meta);

        Ok(())
    }

    pub fn import_media(
        conn: &Connection,
        tracker_name: &str,
        media: &TrackerMedia,
    ) -> CoreResult<String> {
        let is_full = media.synopsis.is_some() || !media.characters.is_empty();

        let cid = if let Some(cid) = TrackerRepository::find_cid_by_tracker(conn, tracker_name, &media.tracker_id)? {
            if is_full {
                let meta = Self::to_core_metadata(&cid, tracker_name, media);
                let _ = ContentRepository::update(conn, &cid, &meta);
            }
            cid
        } else {
            let mut found_cross = None;
            for (cross_tracker, cross_id) in &media.cross_ids {
                if let Some(cid) = TrackerRepository::find_cid_by_tracker(conn, cross_tracker, cross_id)? {
                    found_cross = Some(cid);
                    break;
                }
            }

            if let Some(cid) = found_cross {
                Self::add_mapping(conn, &cid, tracker_name, &media.tracker_id, &Self::tracker_url(tracker_name, &media.tracker_id))?;
                Self::add_cross_mappings(conn, &cid, &media.cross_ids, tracker_name)?;
                if is_full {
                    let meta = Self::to_core_metadata(&cid, tracker_name, media);
                    let _ = ContentRepository::update(conn, &cid, &meta);
                }
                cid
            } else {
                let cid = generate_cid();
                ContentRepository::create(conn, Self::to_core_metadata(&cid, tracker_name, media))?;
                Self::add_mapping(conn, &cid, tracker_name, &media.tracker_id, &Self::tracker_url(tracker_name, &media.tracker_id))?;
                Self::add_cross_mappings(conn, &cid, &media.cross_ids, tracker_name)?;
                cid
            }
        };

        if is_full {
            for rel in &media.relations {
                let target_cid = match Self::import_media(conn, tracker_name, &rel.media) {
                    Ok(id) => id,
                    Err(e) => {
                        tracing::warn!("Failed to import relation: {}", e);
                        continue;
                    }
                };

                let rel_enum = match rel.relation_type.as_str() {
                    "SEQUEL" => RelationType::Sequel,
                    "PREQUEL" => RelationType::Prequel,
                    "SIDE_STORY" => RelationType::SideStory,
                    "SPIN_OFF" => RelationType::Spinoff,
                    "ADAPTATION" => RelationType::Adaptation,
                    "PARENT" => RelationType::Parent,
                    "SUMMARY" => RelationType::Summary,
                    _ => RelationType::Alternative,
                };

                let _ = RelationRepository::upsert(conn, &ContentRelation {
                    id: None,
                    source_cid: cid.clone(),
                    target_cid,
                    relation_type: rel_enum,
                    created_at: chrono::Utc::now().timestamp(),
                });
            }
        }

        Ok(cid)
    }

    fn add_mapping(conn: &Connection, cid: &str, tracker: &str, id: &str, url: &str) -> CoreResult<()> {
        let now = chrono::Utc::now().timestamp();
        TrackerRepository::add_mapping(conn, &TrackerMapping {
            cid: cid.to_string(),
            tracker_name: tracker.to_string(),
            tracker_id: id.to_string(),
            tracker_url: Some(url.to_string()),
            sync_enabled: true,
            last_synced: Some(now),
            created_at: now,
            updated_at: now,
        })?;
        Ok(())
    }

    fn add_cross_mappings(
        conn: &Connection,
        cid: &str,
        cross_ids: &std::collections::HashMap<String, String>,
        skip_tracker: &str,
    ) -> CoreResult<()> {
        for (tracker, id) in cross_ids {
            if tracker == skip_tracker { continue; }
            if TrackerRepository::find_cid_by_tracker(conn, tracker, id)?.is_none() {
                let _ = Self::add_mapping(conn, cid, tracker, id, &Self::tracker_url(tracker, id));
            }
        }
        Ok(())
    }

    fn tracker_url(tracker: &str, id: &str) -> String {
        match tracker {
            "anilist"     => format!("https://anilist.co/anime/{}", id),
            "myanimelist" => format!("https://myanimelist.net/anime/{}", id),
            "simkl"       => format!("https://simkl.com/anime/{}", id),
            _             => String::new(),
        }
    }

    fn to_core_metadata(cid: &str, tracker_name: &str, media: &TrackerMedia) -> CoreMetadata {
        let now = chrono::Utc::now().timestamp();
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

        CoreMetadata {
            cid: cid.to_string(),
            content_type: media.content_type.clone(),
            subtype: media.format.clone(),
            title: media.title.clone(),
            alt_titles: media.alt_titles.clone(),
            synopsis: media.synopsis.clone(),
            cover_image: media.cover_image.clone(),
            banner_image: media.banner_image.clone(),
            eps_or_chapters: EpisodeData::Count(count),
            status,
            tags: media.tags.clone(),
            genres: media.genres.clone(),
            nsfw: media.nsfw,
            release_date: media.release_date.clone(),
            end_date: media.end_date.clone(),
            rating: media.rating,
            trailer_url: media.trailer_url.clone(),
            characters: media.characters.clone(),
            studio: media.studio.clone(),
            staff: media.staff.clone(),
            sources: Some(tracker_name.to_string()),
            external_ids: json!({}),
            created_at: now,
            updated_at: now,
        }
    }
}

pub struct ContentService;

impl ContentService {
    pub async fn create_content(
        state: &Arc<AppState>,
        meta: CoreMetadata,
        trackers: Option<Vec<TrackerMapping>>,
        exts: Option<Vec<ExtensionSource>>,
    ) -> CoreResult<ContentWithMappings> {
        let db = state.db.connection();
        let conn = db.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;

        let cid = ContentRepository::create(&conn, meta)?;

        if let Some(mappings) = trackers {
            for mut m in mappings {
                m.cid = cid.clone();
                TrackerRepository::add_mapping(&conn, &m)?;
            }
        }
        if let Some(sources) = exts {
            for mut s in sources {
                s.cid = cid.clone();
                ExtensionRepository::add_source(&conn, &s)?;
            }
        }

        ContentRepository::get_full_content(&conn, &cid)?
            .ok_or_else(|| CoreError::Internal("Created content not found".into()))
    }

    pub async fn get_content(state: &Arc<AppState>, cid: &str) -> CoreResult<ContentWithMappings> {
        let db = state.db.connection();

        let (content_type, needs_enrichment, tracker_id, lacks_simkl) = {
            let conn = db.lock().unwrap();
            let meta = ContentRepository::get_by_cid(&conn, cid)?;
            let mappings = TrackerRepository::get_mappings_by_cid(&conn, cid).unwrap_or_default();
            let al_id = mappings.iter().find(|m| m.tracker_name == "anilist").map(|m| m.tracker_id.clone());

            // 1. Comprobamos inteligentemente si falta simkl
            let lacks_simkl = !mappings.iter().any(|m| m.tracker_name == "simkl");

            let is_minimal = meta.as_ref().map_or(false, |m| m.synopsis.is_none() && m.characters.is_empty());

            (
                meta.map(|m| m.content_type),
                is_minimal,
                al_id,
                lacks_simkl
            )
        };

        if needs_enrichment {
            if let Some(id) = tracker_id {
                if let Some(provider) = state.tracker_registry.get("anilist") {
                    if let Ok(Some(media)) = provider.get_by_id(&id).await {
                        let conn = db.lock().unwrap();
                        let _ = ContentImportService::import_media(&conn, "anilist", &media);
                    }
                }
            }
        }

        // 2. SOLO enriquecemos si no hay mapping de simkl
        if lacks_simkl && content_type == Some(ContentType::Anime) {
            let _ = ContentImportService::enrich_with_simkl(
                db.clone(),
                state.tracker_registry.clone(),
                cid,
            ).await;
        }

        let conn = db.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;
        ContentRepository::get_full_content(&conn, cid)?
            .ok_or_else(|| CoreError::NotFound(format!("Content {} not found", cid)))
    }

    pub async fn update_content(
        state: &Arc<AppState>,
        cid: &str,
        meta: CoreMetadata,
    ) -> CoreResult<ContentWithMappings> {
        let db = state.db.connection();
        let conn = db.lock().unwrap();
        ContentRepository::update(&conn, cid, &meta)?;
        ContentRepository::get_full_content(&conn, cid)?
            .ok_or_else(|| CoreError::NotFound("Content not found after update".into()))
    }

    pub async fn search_content(
        state: &Arc<AppState>,
        params: SearchParams,
    ) -> CoreResult<ContentListResult> {
        let query_str = params.query.clone().unwrap_or_default();

        if let Some(ext_name) = params.extension.clone() {
            let filters = params.extension_filters.as_deref().unwrap_or("{}");
            return if !query_str.is_empty() || filters != "{}" {
                let ct = params.r#type.as_deref().map(parse_content_type);
                let results = Self::search_via_extension(
                    state, query_str, ext_name, ct, params.extension_filters.clone(),
                ).await?;
                Ok(ContentListResult { total: results.len(), data: results })
            } else {
                Ok(ContentListResult { total: 0, data: vec![] })
            };
        }

        let imported_cids = ContentImportService::search_and_import(
            state.db.connection(),
            state.tracker_registry.clone(),
            &params,
        ).await?;

        let db = state.db.connection();
        let conn = db.lock().unwrap();
        let mut results = Vec::new();
        for cid in imported_cids {
            if let Ok(Some(full)) = ContentRepository::get_full_content(&conn, &cid) {
                results.push(full);
            }
        }

        Ok(ContentListResult { total: results.len(), data: results })
    }

    async fn search_via_extension(
        state: &Arc<AppState>,
        query: String,
        ext_name: String,
        content_type: Option<TrackerContentType>,
        filters_json: Option<String>,
    ) -> CoreResult<Vec<ContentWithMappings>> {
        let filters: Value = filters_json
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or(json!({}));
        let args = json!({ "query": query, "filters": filters });
        let db = state.db.connection();

        let items = match state.extension_manager
            .call_extension_function(&ext_name, "search", vec![args])
            .await
        {
            Ok(Value::Array(arr)) => arr,
            _ => return Ok(vec![]),
        };

        let mut resolved = Vec::new();
        for item in &items {
            let ext_id = match item.get("id").and_then(|v| v.as_str()) {
                Some(id) if !id.is_empty() => id.to_string(),
                _ => continue,
            };

            let inferred_type = content_type.clone().unwrap_or(TrackerContentType::Anime);
            let conn = db.lock().unwrap();
            let res = ContentResolverService::resolve_content(
                &conn, &ext_name, &ext_id, item.clone(), inferred_type,
            )?;

            let cid = match res {
                crate::content::resolver::ResolutionResult::Canonical { cid } => cid,
                crate::content::resolver::ResolutionResult::Derived { cid } => cid,
            };

            if let Ok(Some(full)) = ContentRepository::get_full_content(&conn, &cid) {
                resolved.push(full);
            }
        }

        Ok(resolved)
    }

    pub async fn get_content_items(
        state: &Arc<AppState>,
        cid: &str,
        ext_name: &str,
    ) -> CoreResult<Value> {
        let db = state.db.connection();

        let (ct, ext_id, cache_key, cached) = {
            let conn = db.lock().unwrap();
            let (type_str, id) = ExtensionRepository::get_extension_id_and_type(&conn, cid, ext_name)?
                .ok_or_else(|| CoreError::NotFound("Extension link not found".into()))?;
            let ct = serde_json::from_str::<ContentType>(&format!("\"{}\"", type_str))
                .unwrap_or(ContentType::Anime);
            let key = format!("items:{}:{}", ext_name, id);
            let cached = CacheRepository::get(&conn, &key)?;
            (ct, id, key, cached)
        };

        if let Some(data) = cached { return Ok(data); }

        let func = match ct { ContentType::Anime => "findEpisodes", _ => "findChapters" };
        let items = state.extension_manager
            .call_extension_function(ext_name, func, vec![json!(ext_id)])
            .await
            .map_err(|e| CoreError::Internal(format!("Extension error: {}", e)))?;

        {
            let conn = db.lock().unwrap();
            let _ = CacheRepository::set(&conn, &cache_key, ext_name, "items", &items, 1800);
        }

        Ok(items)
    }

    pub async fn play_content(
        state: &Arc<AppState>,
        cid: &str,
        ext_name: &str,
        number: f64,
        server: Option<String>,
        category: Option<String>,
    ) -> CoreResult<Value> {
        let db = state.db.connection();

        let (ct, ext_id) = {
            let conn = db.lock().unwrap();
            let (t, id) = ExtensionRepository::get_extension_id_and_type(&conn, cid, ext_name)?
                .ok_or_else(|| CoreError::NotFound("Extension link not found".into()))?;
            let ct = serde_json::from_str::<ContentType>(&format!("\"{}\"", t))
                .unwrap_or(ContentType::Anime);
            (ct, id)
        };

        let cache_key = format!("items:{}:{}", ext_name, ext_id);
        let cached = { let conn = db.lock().unwrap(); CacheRepository::get(&conn, &cache_key)? };

        let func = match ct { ContentType::Anime => "findEpisodes", _ => "findChapters" };
        let items_list = match cached {
            Some(d) => d,
            None => state.extension_manager
                .call_extension_function(ext_name, func, vec![json!(ext_id)])
                .await?,
        };

        let items_arr = items_list.as_array()
            .ok_or_else(|| CoreError::Internal("Invalid items list".into()))?;

        let real_id = items_arr.iter()
            .find(|i| i.get("number").and_then(|v| v.as_f64())
                .map(|n| (n - number).abs() < 0.01).unwrap_or(false))
            .and_then(|i| i.get("id").and_then(|v| v.as_str()).map(String::from))
            .ok_or_else(|| CoreError::NotFound("Item number not found".into()))?;

        match ct {
            ContentType::Anime => {
                let srv = server.unwrap_or_else(|| "default".into());
                let cat = category.unwrap_or_else(|| "sub".into());
                let data = state.extension_manager
                    .call_extension_function(ext_name, "findEpisodeServer",
                                             vec![json!(real_id), json!(srv), json!(cat)])
                    .await?;
                Ok(json!({ "type": "video", "data": data }))
            }
            _ => {
                let data = state.extension_manager
                    .call_extension_function(ext_name, "findChapterPages", vec![json!(real_id)])
                    .await?;
                Ok(json!({ "type": "reader", "data": data }))
            }
        }
    }
    
    pub fn add_tracker_mapping(state: &Arc<AppState>, mut mapping: TrackerMapping) -> CoreResult<()> {
        let db = state.db.connection();
        let conn = db.lock().unwrap();
        if !TrackerRepository::has_canonical_mapping(&conn, &mapping.cid)? {
            return Err(CoreError::BadRequest("Cannot add tracker mappings to extension-only content".into()));
        }
        let now = chrono::Utc::now().timestamp();
        mapping.created_at = now;
        mapping.updated_at = now;
        TrackerRepository::add_mapping(&conn, &mapping)?;
        Ok(())
    }

    pub fn update_tracker_mapping(
        state: &Arc<AppState>, cid: &str, tracker_name: &str, tracker_id: &str,
    ) -> CoreResult<()> {
        let db = state.db.connection();
        let conn = db.lock().unwrap();
        if !TrackerRepository::has_canonical_mapping(&conn, cid)? {
            return Err(CoreError::BadRequest("Content is extension-only".into()));
        }
        let now = chrono::Utc::now().timestamp();
        TrackerRepository::add_mapping(&conn, &TrackerMapping {
            cid: cid.to_string(), tracker_name: tracker_name.to_string(),
            tracker_id: tracker_id.to_string(), tracker_url: None,
            sync_enabled: false, last_synced: None, created_at: now, updated_at: now,
        })?;
        Ok(())
    }

    pub fn delete_tracker_mapping(state: &Arc<AppState>, cid: &str, tracker_name: &str) -> CoreResult<()> {
        let db = state.db.connection();
        let conn = db.lock().unwrap();
        let rows = TrackerRepository::delete_mapping(&conn, cid, tracker_name)?;
        if rows == 0 { return Err(CoreError::NotFound("Mapping not found".into())); }
        Ok(())
    }
    
    pub fn add_extension_source(state: &Arc<AppState>, mut source: ExtensionSource) -> CoreResult<i64> {
        let db = state.db.connection();
        let conn = db.lock().unwrap();
        let now = chrono::Utc::now().timestamp();
        source.created_at = now;
        source.updated_at = now;
        let id = ExtensionRepository::add_source(&conn, &source)?;
        Ok(id)
    }

    pub fn update_extension_mapping(
        state: &Arc<AppState>, cid: &str, ext_name: &str, ext_id: &str, meta: Value,
    ) -> CoreResult<ContentWithMappings> {
        let db = state.db.connection();
        let conn = db.lock().unwrap();
        let now = chrono::Utc::now().timestamp();

        if let Some(id) = ExtensionRepository::find_mapping_id(&conn, cid, ext_name)? {
            ExtensionRepository::update_source(&conn, id, ext_id, &meta.to_string())?;
        } else {
            ExtensionRepository::add_source(&conn, &ExtensionSource {
                id: None, cid: cid.to_string(), extension_name: ext_name.to_string(),
                extension_id: ext_id.to_string(), content_url: None, stream_url: None,
                read_url: None, download_url: None, metadata: meta, nsfw: false,
                quality: None, language: None, created_at: now, updated_at: now,
            })?;
        }

        ContentRepository::get_full_content(&conn, cid)?
            .ok_or_else(|| CoreError::NotFound("Content not found".into()))
    }
    
    pub fn resolve_by_tracker(state: &Arc<AppState>, tracker: &str, id: &str) -> CoreResult<ContentWithMappings> {
        let db = state.db.connection();
        let conn = db.lock().unwrap();
        let cid = TrackerRepository::find_cid_by_tracker(&conn, tracker, id)?
            .ok_or_else(|| CoreError::NotFound("Content mapping not found".into()))?;
        ContentRepository::get_full_content(&conn, &cid)?
            .ok_or_else(|| CoreError::NotFound("Content data missing".into()))
    }

    pub async fn resolve_by_extension(
        state: &Arc<AppState>, ext_name: &str, ext_id: &str,
    ) -> CoreResult<ContentWithMappings> {
        let content_type = state.extension_manager.list_extensions().iter()
            .find(|e| e.name == ext_name)
            .map(|e| match e.ext_type {
                ExtensionType::Anime  => TrackerContentType::Anime,
                ExtensionType::Manga  => TrackerContentType::Manga,
                ExtensionType::Novel  => TrackerContentType::Novel,
                ExtensionType::Booru  => TrackerContentType::Booru,
                _                     => TrackerContentType::Anime,
            })
            .ok_or_else(|| CoreError::NotFound("Extension not found".into()))?;

        let meta = state.extension_manager
            .call_extension_function(ext_name, "getMetadata", vec![json!(ext_id)])
            .await
            .map_err(|e| CoreError::Internal(format!("Metadata fetch failed: {}", e)))?;

        let db = state.db.connection();
        let cid = {
            let conn = db.lock().unwrap();
            match ContentResolverService::resolve_content(&conn, ext_name, ext_id, meta, content_type)? {
                crate::content::resolver::ResolutionResult::Canonical { cid } => cid,
                crate::content::resolver::ResolutionResult::Derived { cid } => cid,
            }
        };

        let conn = db.lock().unwrap();
        ContentRepository::get_full_content(&conn, &cid)?
            .ok_or_else(|| CoreError::NotFound("Resolved content not found".into()))
    }

    pub async fn search_extension_direct(
        state: &Arc<AppState>,
        ext_name: &str,
        query: Option<String>,
        filters_json: Option<String>,
    ) -> CoreResult<ExtensionSearchResponse> {
        let filters: Value = filters_json
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or(json!({}));

        let args = json!({
            "query": query.unwrap_or_default(),
            "filters": filters,
        });

        let results = state.extension_manager
            .call_extension_function(ext_name, "search", vec![args])
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?;

        Ok(ExtensionSearchResponse { success: true, results })
    }
}