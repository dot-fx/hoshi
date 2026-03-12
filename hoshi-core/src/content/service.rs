use std::sync::Arc;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::config::repository::ConfigRepo;
use crate::content::repository::{
    ContentRepository, ContentMetadata, ContentStatus, ContentType, ContentWithMappings,
    EpisodeData, ExtensionRepository, ExtensionSource, CacheRepository, generate_cid,
    RelationType, ContentRelation, RelationRepository, ContentUnitRepository,
};
use crate::tracker::repository::{TrackerMapping, TrackerRepository};
use crate::content::resolver::ContentResolverService;
use crate::db::DatabaseManager;
use crate::error::{CoreError, CoreResult};
use crate::extensions::ExtensionType;
use crate::state::AppState;
use crate::tracker::provider::{ContentType as TrackerContentType, TrackerMedia, TrackerRegistry};
use crate::tracker::provider::simkl::SimklProvider;

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
    #[serde(rename = "type")]
    pub play_type: Value,
    pub data: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaSection {
    pub trending:  Vec<Value>,
    pub top_rated: Vec<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seasonal:  Option<Vec<Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HomeView {
    pub anime: MediaSection,
    pub manga: MediaSection,
    pub novel: MediaSection,
    pub cached_at: i64,
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackerCandidate {
    pub tracker_name: String,
    pub tracker_id: String,
    pub title: String,
    pub cover_image: Option<String>,
    pub score: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveExtensionResponse {
    pub success: bool,
    pub data: ContentWithMappings,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracker_candidates: Option<Vec<TrackerCandidate>>,
    pub auto_linked: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateContentRequest {
    pub content_type: ContentType,
    pub nsfw: bool,
    pub metadata: ContentMetadata,
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
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTrackerMappingRequest {
    pub tracker_name: String,
    pub tracker_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkTrackerRequest {
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
const HOME_CACHE_KEY: &str  = "home_view_v2";
const HOME_CACHE_TTL: i64   = 6 * 3600;
pub struct ContentImportService;

impl ContentImportService {
    pub async fn get_home_view(
        db_manager: Arc<DatabaseManager>,
        registry: Arc<TrackerRegistry>,
        user_id: Option<i32>,
    ) -> CoreResult<Value> {

        let show_adult = user_id.map(|uid| {
            let conn_arc = db_manager.connection();
            let Ok(conn) = conn_arc.lock() else { return false };
            ConfigRepo::get_config(&conn, uid)
                .map(|c| c.general.show_adult_content)
                .unwrap_or(false)
        }).unwrap_or(false);

        {
            let db_arc = db_manager.connection();
            let conn = db_arc.lock()
                .map_err(|_| CoreError::Internal("DB Lock".into()))?;
            if let Some(cached) = CacheRepository::get(&conn, HOME_CACHE_KEY)? {
                tracing::debug!("home_view: cache hit");
                return Ok(if show_adult { cached } else { Self::filter_home_nsfw(cached) });
            }
        }

        let provider = registry.get("anilist")
            .ok_or_else(|| CoreError::Internal("AniList provider not registered".into()))?;

        let sections = provider.get_home().await?;
        let db = db_manager.connection();

        let mut enrich = |section_key: &str| -> CoreResult<Vec<Value>> {
            let items = sections.get(section_key).cloned().unwrap_or_default();
            let mut enriched = Vec::with_capacity(items.len());
            for media in &items {
                let cid = {
                    let conn = db.lock().map_err(|_| CoreError::Internal("DB Lock".into()))?;
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
                trending:  enrich("trending_anime")?,
                top_rated: enrich("top_rated_anime")?,
                seasonal:  Some(enrich("seasonal_anime")?),
            },
            manga: MediaSection {
                trending:  enrich("trending_manga")?,
                top_rated: enrich("top_rated_manga")?,
                seasonal:  None,
            },
            novel: MediaSection {
                trending:  enrich("trending_novel")?,
                top_rated: enrich("top_rated_novel")?,
                seasonal:  None,
            },
            cached_at: now,
        };

        let value = serde_json::to_value(&view)
            .map_err(|e| CoreError::Internal(e.to_string()))?;

        {
            let db_arc = db_manager.connection();
            let conn = db_arc.lock()
                .map_err(|_| CoreError::Internal("DB Lock".into()))?;
            if let Err(e) = CacheRepository::set(&conn, HOME_CACHE_KEY, "anilist", "home", &value, HOME_CACHE_TTL) {
                tracing::warn!("home_view: failed to write cache: {}", e);
            }
        }

        Ok(if show_adult { value } else { Self::filter_home_nsfw(value) })
    }

    /// Filters nsfw items from all sections of a home view JSON value.
    /// Cache always stores unfiltered — filter on the way out per user.
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
            params.nsfw,
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

        let (existing_mappings, mut meta) = {
            let conn = db.lock().unwrap();
            let mappings = TrackerRepository::get_mappings_by_cid(&conn, cid)?;
            let meta = ContentRepository::get_by_cid(&conn, cid)?
                .ok_or_else(|| CoreError::NotFound("Content metadata not found".into()))?;
            (mappings, meta)
        };

        let mut simkl_id = existing_mappings.iter()
            .find(|m| m.tracker_name == "simkl")
            .map(|m| m.tracker_id.clone());

        if simkl_id.is_none() {
            let al_id  = existing_mappings.iter().find(|m| m.tracker_name == "anilist").map(|m| m.tracker_id.clone());
            let mal_id = existing_mappings.iter().find(|m| m.tracker_name == "myanimelist").map(|m| m.tracker_id.clone());

            if al_id.is_none() && mal_id.is_none() {
                return Ok(());
            }

            let id_type = if al_id.is_some() { "anilist" } else { "mal" };
            let id_val  = al_id.as_deref().or(mal_id.as_deref()).unwrap();

            if let Ok(mut results) = simkl.search(
                Some(&format!("id:{}:{}", id_type, id_val)),
                TrackerContentType::Anime, 1, None, None, None, None,
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

        let mut external_ids_map = match meta.external_ids.clone() {
            Value::Object(map) => map,
            _ => serde_json::Map::new(),
        };

        let allowed_trackers = ["myanimelist", "anilist", "kitsu", "anidb", "imdb", "livechart", "trakt", "animeplanet"];
        let now = chrono::Utc::now().timestamp();
        let mut new_mappings: Vec<TrackerMapping> = Vec::new();

        if !existing_mappings.iter().any(|m| m.tracker_name == "simkl") {
            new_mappings.push(TrackerMapping {
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

        for (key, val) in &simkl_media.cross_ids {
            let tracker_name = match key.as_str() {
                "mal"                    => "myanimelist",
                "ann"                    => "animenewsnetwork",
                "trakttv" | "trakttvslug" => "trakt",
                other                    => other,
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
                        cid: cid.to_string(),
                        tracker_name: tracker_name.to_string(),
                        tracker_id: val.to_string(),
                        tracker_url: url,
                        sync_enabled: false,
                        last_synced: None,
                        created_at: now,
                        updated_at: now,
                    });
                }
                external_ids_map.remove(tracker_name);
                external_ids_map.remove(key);
            } else {
                external_ids_map.insert(key.clone(), Value::String(val.clone()));
            }
        }

        meta.external_ids = Value::Object(external_ids_map);

        if meta.synopsis.is_none()    && simkl_media.synopsis.is_some()    { meta.synopsis    = simkl_media.synopsis; }
        if meta.trailer_url.is_none() && simkl_media.trailer_url.is_some() { meta.trailer_url = simkl_media.trailer_url; }
        if meta.cover_image.is_none() && simkl_media.cover_image.is_some() { meta.cover_image = simkl_media.cover_image; }
        if meta.rating.is_none()      && simkl_media.rating.is_some()      { meta.rating      = simkl_media.rating; }

        let simkl_provider = SimklProvider::new();
        let episodes = simkl_provider.get_episodes(&simkl_id).await.ok();

        {
            let conn = db.lock().map_err(|_| CoreError::Internal("DB Lock".into()))?;
            for mapping in &new_mappings {
                let _ = TrackerRepository::add_mapping(&conn, mapping);
            }
            if let Some(eps) = episodes {
                for ep_json in eps {
                    if let Err(e) = ContentUnitRepository::upsert(&conn, cid, &ep_json) {
                        tracing::error!("Failed to upsert content unit for {}: {}", cid, e);
                    }
                }
            }
            let _ = ContentRepository::upsert_metadata(&conn, &meta);
        }

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
                let meta = Self::to_content_metadata(&cid, tracker_name, media);
                let _ = ContentRepository::upsert_metadata(conn, &meta);
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
                            tracing::warn!(
                                "cross_id match discarded: tracker='{}' id='{}' → cid='{}' \
                                 type={:?} but importing type={:?}",
                                cross_tracker, cross_id, cid,
                                existing.content_type, media.content_type
                            );
                        }
                        None => {}
                    }
                }
            }

            if let Some(cid) = found_cross {
                Self::add_mapping(conn, &cid, tracker_name, &media.tracker_id, &Self::tracker_url(tracker_name, &media.tracker_id, &media.content_type))?;
                Self::add_cross_mappings(conn, &cid, &media.cross_ids, tracker_name, &media.content_type)?;
                if is_full {
                    let meta = Self::to_content_metadata(&cid, tracker_name, media);
                    let _ = ContentRepository::upsert_metadata(conn, &meta);
                }
                cid
            } else {
                let cid = generate_cid();
                let meta = Self::to_content_metadata(&cid, tracker_name, media);
                ContentRepository::create_with_type(conn, &media.content_type, media.nsfw, meta)?;
                Self::add_mapping(conn, &cid, tracker_name, &media.tracker_id, &Self::tracker_url(tracker_name, &media.tracker_id, &media.content_type))?;
                Self::add_cross_mappings(conn, &cid, &media.cross_ids, tracker_name, &media.content_type)?;
                cid
            }
        };

        if is_full {
            for rel in &media.relations {
                let target_cid = match Self::import_media(conn, tracker_name, &rel.media) {
                    Ok(id) => id,
                    Err(e) => { tracing::warn!("Failed to import relation: {}", e); continue; }
                };

                let rel_enum = match rel.relation_type.as_str() {
                    "SEQUEL"      => RelationType::Sequel,
                    "PREQUEL"     => RelationType::Prequel,
                    "SIDE_STORY"  => RelationType::SideStory,
                    "SPIN_OFF"    => RelationType::Spinoff,
                    "ADAPTATION"  => RelationType::Adaptation,
                    "PARENT"      => RelationType::Parent,
                    "SUMMARY"     => RelationType::Summary,
                    _             => RelationType::Alternative,
                };

                let _ = RelationRepository::upsert(conn, &ContentRelation {
                    id: None,
                    source_cid: cid.clone(),
                    target_cid,
                    relation_type: rel_enum,
                    source_name: tracker_name.to_string(),
                    created_at: chrono::Utc::now().timestamp(),
                });
            }
        }

        Ok(cid)
    }

    pub fn add_mapping(conn: &Connection, cid: &str, tracker: &str, id: &str, url: &str) -> CoreResult<()> {
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
        let now = chrono::Utc::now().timestamp();
        let count = match media.content_type {
            TrackerContentType::Anime => media.episode_count.unwrap_or(0),
            _                         => media.chapter_count.unwrap_or(0),
        };

        let status = media.status.as_deref().map(|s| match s {
            "FINISHED" | "ended" | "completed"  => ContentStatus::Completed,
            "RELEASING" | "ongoing" | "airing"  => ContentStatus::Ongoing,
            "NOT_YET_RELEASED" | "planned"      => ContentStatus::Planned,
            "CANCELLED" | "canceled"            => ContentStatus::Cancelled,
            "HIATUS"                            => ContentStatus::Hiatus,
            _                                   => ContentStatus::Ongoing,
        });

        ContentMetadata {
            id: None,
            cid: cid.to_string(),
            source_name: tracker_name.to_string(),
            source_id: Some(media.tracker_id.clone()),
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
            release_date: media.release_date.clone(),
            end_date: media.end_date.clone(),
            rating: media.rating,
            trailer_url: media.trailer_url.clone(),
            characters: media.characters.clone(),
            studio: media.studio.clone(),
            staff: media.staff.clone(),
            external_ids: json!({}),
            created_at: now,
            updated_at: now,
        }
    }
    pub async fn get_trending(
        db_manager: Arc<DatabaseManager>,
        registry: Arc<TrackerRegistry>,
        media_type: &str,
        user_id: Option<i32>,
    ) -> CoreResult<Value> {
        let cache_key = format!("trending_{}_v2", media_type);

        let show_adult = user_id.map(|uid| {
            let conn_arc = db_manager.connection();
            let Ok(conn) = conn_arc.lock() else { return false };
            ConfigRepo::get_config(&conn, uid)
                .map(|c| c.general.show_adult_content)
                .unwrap_or(false)
        }).unwrap_or(false);

        {
            let db_arc = db_manager.connection();
            let conn = db_arc.lock()
                .map_err(|_| CoreError::Internal("DB Lock".into()))?;
            if let Some(cached) = CacheRepository::get(&conn, &cache_key)? {
                return Ok(if show_adult { cached } else { Self::filter_array_nsfw(cached) });
            }
        }

        {
            let db_arc = db_manager.connection();
            let conn = db_arc.lock()
                .map_err(|_| CoreError::Internal("DB Lock".into()))?;
            if let Some(home) = CacheRepository::get(&conn, HOME_CACHE_KEY)? {
                if let Some(section) = home.get(media_type) {
                    if let Some(trending) = section.get("trending") {
                        let val = trending.clone();
                        return Ok(if show_adult { val } else { Self::filter_array_nsfw(val) });
                    }
                }
            }
        }

        let provider = registry.get("anilist")
            .ok_or_else(|| CoreError::Internal("AniList provider not registered".into()))?;

        let sections = provider.get_home().await?;
        let db = db_manager.connection();

        let section_key = format!("trending_{}", media_type);
        let items = sections.get(&section_key).cloned().unwrap_or_default();

        let mut enriched = Vec::with_capacity(items.len());
        for media in &items {
            let cid = {
                let conn = db.lock().map_err(|_| CoreError::Internal("DB Lock".into()))?;
                Self::import_media(&conn, "anilist", media)?
            };
            let mut item_json = serde_json::to_value(media).unwrap_or(json!({}));
            item_json["cid"] = json!(cid);
            enriched.push(item_json);
        }

        let value = json!(enriched);

        {
            let db_arc = db_manager.connection();
            let conn = db_arc.lock()
                .map_err(|_| CoreError::Internal("DB Lock".into()))?;
            let _ = CacheRepository::set(&conn, &cache_key, "anilist", "trending", &value, HOME_CACHE_TTL);
        }

        Ok(if show_adult { value } else { Self::filter_array_nsfw(value) })
    }

}

pub struct ContentService;

impl ContentService {

    /// Reads show_adult_content from the user's config in the DB.
    /// Returns true (show all) if user_id is None or config can't be read.
    fn show_adult(state: &Arc<AppState>, user_id: Option<i32>) -> bool {
        let uid = match user_id {
            Some(id) => id,
            None => return false, // no user = safe default
        };
        let conn = state.db.connection();
        let Ok(lock) = conn.lock() else { return false };
        ConfigRepo::get_config(&lock, uid)
            .map(|c| c.general.show_adult_content)
            .unwrap_or(false)
    }

    async fn save_extension_metadata(
        state: &Arc<AppState>,
        cid: &str,
        ext_name: &str,
        ext_id: &str,
    ) {
        tracing::debug!("[ext_meta] fetching getMetadata ext={} id={}", ext_name, ext_id);
        let ext_meta = match state.extension_manager.read().await
            .call_extension_function(ext_name, "getMetadata", vec![json!(ext_id)])
            .await
        {
            Ok(v)  => v,
            Err(e) => {
                tracing::warn!("[ext_meta] getMetadata failed ext={} id={}: {}", ext_name, ext_id, e);
                return;
            }
        };

        let title = match ext_meta.get("title").and_then(|v| v.as_str()) {
            Some(t) => t.to_string(),
            None => {
                tracing::warn!("[ext_meta] no title field ext={} id={}", ext_name, ext_id);
                return;
            }
        };

        let now = chrono::Utc::now().timestamp();
        let meta = ContentMetadata {
            id:              None,
            cid:             cid.to_string(),
            source_name:     ext_name.to_string(),
            source_id:       Some(ext_id.to_string()),
            subtype:         None,
            title,
            alt_titles:      vec![],
            synopsis:        ext_meta.get("synopsis").and_then(|v| v.as_str()).map(String::from),
            cover_image:     ext_meta.get("image").or(ext_meta.get("cover")).and_then(|v| v.as_str()).map(String::from),
            banner_image:    None,
            eps_or_chapters: ext_meta.get("eps_or_chapters").and_then(|v| v.as_i64())
                .map(|n| EpisodeData::Count(n as i32))
                .unwrap_or(EpisodeData::Count(0)),
            status:          None,
            tags:            vec![],
            genres:          ext_meta.get("genres").and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
            release_date:    ext_meta.get("year").and_then(|v| v.as_i64()).map(|y| format!("{}-01-01", y)),
            end_date:        None,
            rating:          ext_meta.get("rating").and_then(|v| v.as_f64()).map(|v| v as f32),
            trailer_url:     None,
            characters:      vec![],
            studio:          None,
            staff:           vec![],
            external_ids:    ext_meta.get("external_ids").cloned().unwrap_or(serde_json::json!({})),
            created_at:      now,
            updated_at:      now,
        };

        let db = state.db.connection();
        let conn = db.lock().unwrap();
        match ContentRepository::upsert_metadata(&conn, &meta) {
            Ok(_)  => tracing::info!("[ext_meta] saved cid={} source={}", cid, ext_name),
            Err(e) => tracing::error!("[ext_meta] upsert_metadata FAILED cid={} source={}: {:?}", cid, ext_name, e),
        }
    }

    pub async fn create_content(
        state: &Arc<AppState>,
        content_type: ContentType,
        nsfw: bool,
        meta: ContentMetadata,
        trackers: Option<Vec<TrackerMapping>>,
        exts: Option<Vec<ExtensionSource>>,
    ) -> CoreResult<ContentWithMappings> {
        let db = state.db.connection();
        let conn = db.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;

        let cid = ContentRepository::create_with_type(&conn, &content_type, nsfw, meta)?;

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
        let cid = cid.to_string();

        let (content_type, needs_enrichment, tracker_id, lacks_simkl, is_releasing) =
            tokio::task::spawn_blocking({
                let db = db.clone();
                let cid = cid.clone();
                move || -> CoreResult<_> {
                    let conn = db.lock().map_err(|_| CoreError::Internal("DB Lock".into()))?;
                    let content  = ContentRepository::get_content_by_cid(&conn, &cid)?;
                    let meta     = ContentRepository::get_by_cid(&conn, &cid)?;
                    let mappings = TrackerRepository::get_mappings_by_cid(&conn, &cid).unwrap_or_default();
                    let al_id    = mappings.iter().find(|m| m.tracker_name == "anilist").map(|m| m.tracker_id.clone());
                    let lacks_simkl  = !mappings.iter().any(|m| m.tracker_name == "simkl");
                    let is_minimal   = meta.as_ref().map_or(false, |m| m.synopsis.is_none() && m.characters.is_empty());
                    let is_releasing = meta.as_ref()
                        .and_then(|m| m.status.as_ref())
                        .map(|s| matches!(s, ContentStatus::Ongoing))
                        .unwrap_or(false);
                    Ok((content.map(|c| c.content_type), is_minimal, al_id, lacks_simkl, is_releasing))
                }
            })
                .await
                .map_err(|e| CoreError::Internal(e.to_string()))??;

        if needs_enrichment {
            if let Some(id) = tracker_id.clone() {
                if let Some(provider) = state.tracker_registry.get("anilist") {
                    if let Ok(Some(media)) = provider.get_by_id(&id).await {
                        let db = db.clone();
                        tokio::task::spawn_blocking(move || {
                            let conn = db.lock().unwrap();
                            let _ = ContentImportService::import_media(&conn, "anilist", &media);
                        }).await.ok();
                    }
                }
            }
        }

        if lacks_simkl && content_type == Some(ContentType::Anime) {
            let _ = ContentImportService::enrich_with_simkl(
                db.clone(), state.tracker_registry.clone(), &cid,
            ).await;
        }

        // For releasing content: fire a background task to refresh metadata and content
        // units without blocking the response. This catches new episodes/chapters and
        // eventual status changes (Ongoing -> Completed).
        if is_releasing {
            let state_bg = state.clone();
            let cid_bg   = cid.clone();
            let al_id_bg = tracker_id.clone();

            tokio::spawn(async move {
                // 1. Re-fetch metadata from AniList to catch status changes
                if let Some(id) = al_id_bg {
                    if let Some(provider) = state_bg.tracker_registry.get("anilist") {
                        if let Ok(Some(media)) = provider.get_by_id(&id).await {
                            let db = state_bg.db.connection();
                            let _ = tokio::task::spawn_blocking(move || {
                                let conn = db.lock().unwrap();
                                let _ = ContentImportService::import_media(&conn, "anilist", &media);
                            }).await;
                            tracing::debug!("[bg_refresh] metadata updated for cid={}", cid_bg);
                        }
                    }
                }

                // 2. Refresh content units for all linked extensions and bust their cache
                let extensions = {
                    let db = state_bg.db.connection();
                    let conn = db.lock().unwrap();
                    ExtensionRepository::get_by_cid(&conn, &cid_bg).unwrap_or_default()
                };

                for source in extensions {
                    let ext_name  = source.extension_name.clone();
                    let ext_id    = source.extension_id.clone();
                    let ct = {
                        let db = state_bg.db.connection();
                        let conn = db.lock().unwrap();
                        ExtensionRepository::get_extension_id_and_type(&conn, &cid_bg, &ext_name)
                            .ok()
                            .flatten()
                            .map(|(t, _)| serde_json::from_str::<ContentType>(&format!("\"{}\"", t)).unwrap_or(ContentType::Anime))
                            .unwrap_or(ContentType::Anime)
                    };

                    let func      = match ct { ContentType::Anime => "findEpisodes", _ => "findChapters" };
                    let cache_key = format!("items:{}:{}", ext_name, ext_id);

                    match state_bg.extension_manager.read().await
                        .call_extension_function(&ext_name, func, vec![json!(ext_id)])
                        .await
                    {
                        Ok(items) => {
                            let db = state_bg.db.connection();
                            let conn = db.lock().unwrap();
                            let _ = CacheRepository::set(&conn, &cache_key, &ext_name, "items", &items, 1800);
                            tracing::debug!("[bg_refresh] units refreshed cid={} ext={}", cid_bg, ext_name);
                        }
                        Err(e) => {
                            tracing::warn!("[bg_refresh] failed units refresh cid={} ext={}: {}", cid_bg, ext_name, e);
                        }
                    }
                }
            });
        }

        tokio::task::spawn_blocking(move || {
            let conn = db.lock().map_err(|_| CoreError::Internal("DB Lock".into()))?;
            ContentRepository::get_full_content(&conn, &cid)?
                .ok_or_else(|| CoreError::NotFound(format!("Content {} not found", cid)))
        })
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?
    }

    pub async fn update_content(
        state: &Arc<AppState>,
        cid: &str,
        meta: ContentMetadata,
    ) -> CoreResult<ContentWithMappings> {
        let db = state.db.connection();
        let conn = db.lock().unwrap();
        ContentRepository::upsert_metadata(&conn, &meta)?;
        ContentRepository::get_full_content(&conn, cid)?
            .ok_or_else(|| CoreError::NotFound("Content not found after update".into()))
    }

    pub async fn search_content(
        state: &Arc<AppState>,
        params: SearchParams,
        user_id: Option<i32>,
    ) -> CoreResult<ContentListResult> {
        let show_adult = Self::show_adult(state, user_id);
        let query_str = params.query.clone().unwrap_or_default();

        if let Some(ext_name) = params.extension.clone() {
            let filters = params.extension_filters.as_deref().unwrap_or("{}");
            return if !query_str.is_empty() || filters != "{}" {
                let ct = params.r#type.as_deref().map(parse_content_type);
                let mut results = Self::search_via_extension(state, query_str, ext_name, ct, params.extension_filters.clone()).await?;
                if !show_adult {
                    results.retain(|c| !c.content.nsfw);
                }
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
                if show_adult || !full.content.nsfw {
                    results.push(full);
                }
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

        let items = match state.extension_manager.read().await
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

        let maybe_link = {
            let conn = db.lock().unwrap();
            ExtensionRepository::get_extension_id_and_type(&conn, cid, ext_name)?
        };

        let (ct, ext_id, cache_key, cached) = if let Some((type_str, id)) = maybe_link {
            tracing::info!("[items] existing link: cid={} ext={} ext_id={}", cid, ext_name, id);
            let ct = serde_json::from_str::<ContentType>(&format!("\"{}\"", type_str))
                .unwrap_or(ContentType::Anime);
            let key = format!("items:{}:{}", ext_name, id);
            let cached = {
                let conn = db.lock().unwrap();
                CacheRepository::get(&conn, &key)?
            };

            let has_meta = {
                let conn = db.lock().unwrap();
                conn.query_row(
                    "SELECT COUNT(*) FROM metadata WHERE cid=?1 AND source_name=?2",
                    rusqlite::params![cid, ext_name],
                    |row| row.get::<_, i64>(0),
                ).unwrap_or(0) > 0
            };
            if !has_meta {
                Self::save_extension_metadata(state, cid, ext_name, &id).await;
            }

            (ct, id, key, cached)
        } else {
            tracing::info!("[items] no link found for cid={} ext={}, will auto-link", cid, ext_name);
            let (title, ct) = {
                let conn = db.lock().unwrap();
                let content = ContentRepository::get_content_by_cid(&conn, cid)?
                    .ok_or_else(|| CoreError::NotFound("Content not found".into()))?;
                let meta = ContentRepository::get_by_cid(&conn, cid)?
                    .ok_or_else(|| CoreError::NotFound("Content metadata not found".into()))?;
                (meta.title, content.content_type)
            };

            tracing::info!("No extension link for cid={} ext={}, auto-linking by title '{}'", cid, ext_name, title);

            let results = state.extension_manager.read().await
                .call_extension_function(ext_name, "search", vec![json!({ "query": title, "filters": {} })])
                .await
                .map_err(|e| CoreError::Internal(format!("Extension search failed: {}", e)))?;

            let best_id = results.as_array()
                .and_then(|arr| {
                    arr.iter()
                        .filter_map(|item| {
                            let id = item.get("id")?.as_str()?;
                            let item_title = item.get("title")?.as_str()?;
                            let score = str_similarity(&title, item_title);
                            Some((id.to_string(), score))
                        })
                        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                })
                .filter(|(_, score)| *score >= 0.6)
                .map(|(id, _)| id)
                .ok_or_else(|| CoreError::NotFound(format!("No match found in {} for '{}'", ext_name, title)))?;

            {
                let now = chrono::Utc::now().timestamp();
                let conn = db.lock().unwrap();
                if let Err(e) = ExtensionRepository::add_source(&conn, &ExtensionSource {
                    id: None, cid: cid.to_string(), extension_name: ext_name.to_string(),
                    extension_id: best_id.clone(), nsfw: false,
                    language: None, created_at: now, updated_at: now,
                }) {
                    tracing::warn!("[items/else] add_source failed (may already exist): {}", e);
                }
            }

            Self::save_extension_metadata(state, cid, ext_name, &best_id).await;

            let key = format!("items:{}:{}", ext_name, best_id);
            (ct, best_id, key, None)
        };

        if let Some(data) = cached { return Ok(data); }

        let func = match ct { ContentType::Anime => "findEpisodes", _ => "findChapters" };
        let items = state.extension_manager.read().await
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

        let existing = {
            let conn = db.lock().unwrap();
            ExtensionRepository::get_extension_id_and_type(&conn, cid, ext_name)?
        };

        if existing.is_none() {
            let title = {
                let conn = db.lock().unwrap();
                ContentRepository::get_by_cid(&conn, cid)?
                    .ok_or_else(|| CoreError::NotFound(format!("Content '{}' not found", cid)))?
                    .title
            };

            tracing::debug!("No extension link for cid='{}' ext='{}', searching by title '{}'", cid, ext_name, title);

            let search_results = state.extension_manager.read().await
                .call_extension_function(ext_name, "search", vec![json!({ "query": title, "filters": {} })])
                .await
                .map_err(|e| CoreError::Internal(format!("Extension search failed: {}", e)))?;

            let found_ext_id = search_results
                .as_array()
                .and_then(|arr| arr.first())
                .and_then(|item| item.get("id").and_then(|v| v.as_str()).map(String::from))
                .ok_or_else(|| CoreError::NotFound(format!(
                    "No results in extension '{}' for title '{}'", ext_name, title
                )))?;

            Self::resolve_extension_item(state, ext_name, &found_ext_id).await?;
            Self::save_extension_metadata(state, cid, ext_name, &found_ext_id).await;
        }

        let (ct, ext_id) = {
            let conn = db.lock().unwrap();
            let (t, id) = ExtensionRepository::get_extension_id_and_type(&conn, cid, ext_name)?
                .ok_or_else(|| CoreError::Internal(format!(
                    "Auto-link failed: no link found for cid='{}' ext='{}'", cid, ext_name
                )))?;
            let ct = serde_json::from_str::<ContentType>(&format!("\"{}\"", t))
                .unwrap_or(ContentType::Anime);
            (ct, id)
        };

        let cache_key = format!("items:{}:{}", ext_name, ext_id);
        let cached = { let conn = db.lock().unwrap(); CacheRepository::get(&conn, &cache_key)? };

        let func = match ct { ContentType::Anime => "findEpisodes", _ => "findChapters" };
        let items_list = match cached {
            Some(d) => d,
            None => state.extension_manager.read().await
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
                let data = state.extension_manager.read().await
                    .call_extension_function(ext_name, "findEpisodeServer",
                                             vec![json!(real_id), json!(srv), json!(cat)])
                    .await?;
                Ok(json!({ "type": "video", "data": data }))
            }
            _ => {
                let data = state.extension_manager.read().await
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

    pub async fn update_extension_mapping(
        state: &Arc<AppState>, cid: &str, ext_name: &str, ext_id: &str,
    ) -> CoreResult<ContentWithMappings> {
        let db = state.db.connection();
        {
            let conn = db.lock().unwrap();
            let now = chrono::Utc::now().timestamp();
            if let Some(id) = ExtensionRepository::find_mapping_id(&conn, cid, ext_name)? {
                ExtensionRepository::update_source(&conn, id, ext_id)?;
            } else {
                ExtensionRepository::add_source(&conn, &ExtensionSource {
                    id: None, cid: cid.to_string(), extension_name: ext_name.to_string(),
                    extension_id: ext_id.to_string(), nsfw: false,
                    language: None, created_at: now, updated_at: now,
                })?;
            }
        }

        Self::save_extension_metadata(state, cid, ext_name, ext_id).await;

        let db2 = state.db.connection();
        let conn = db2.lock().unwrap();
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
        let content_type = {
            let mgr = state.extension_manager.read().await;
            mgr.list_extensions().iter()
                .find(|e| e.id == ext_name)
                .map(|e| match e.ext_type {
                    ExtensionType::Anime  => TrackerContentType::Anime,
                    ExtensionType::Manga  => TrackerContentType::Manga,
                    ExtensionType::Novel  => TrackerContentType::Novel,
                    ExtensionType::Booru  => TrackerContentType::Booru,
                    _                     => TrackerContentType::Anime,
                })
                .ok_or_else(|| CoreError::NotFound("Extension not found".into()))?
        };

        let meta = state.extension_manager.read().await
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
        let args = json!({ "query": query.unwrap_or_default(), "filters": filters });

        let results = state.extension_manager.read().await
            .call_extension_function(ext_name, "search", vec![args])
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?;

        Ok(ExtensionSearchResponse { success: true, results })
    }

    pub async fn link_tracker(
        state: &Arc<AppState>,
        cid: &str,
        tracker_name: &str,
        tracker_id: &str,
    ) -> CoreResult<ContentWithMappings> {
        let provider = state.tracker_registry.get(tracker_name)
            .ok_or_else(|| CoreError::NotFound(format!("Tracker '{}' not registered", tracker_name)))?;

        let media = provider.get_by_id(tracker_id).await?
            .ok_or_else(|| CoreError::NotFound(format!("ID {} not found in {}", tracker_id, tracker_name)))?;

        let cid_owned          = cid.to_string();
        let tracker_name_owned = tracker_name.to_string();
        let db = state.db.connection();

        tokio::task::spawn_blocking({
            let db           = db.clone();
            let cid          = cid_owned.clone();
            let media        = media.clone();
            let tracker_name = tracker_name_owned.clone();
            move || -> CoreResult<()> {
                let conn = db.lock().unwrap();
                let rich_meta = ContentImportService::to_content_metadata(&cid, &tracker_name, &media);
                ContentRepository::upsert_metadata(&conn, &rich_meta)?;

                let now = chrono::Utc::now().timestamp();
                let _ = TrackerRepository::add_mapping(&conn, &TrackerMapping {
                    cid: cid.clone(),
                    tracker_name: tracker_name.clone(),
                    tracker_id: media.tracker_id.clone(),
                    tracker_url: Some(ContentImportService::tracker_url(&tracker_name, &media.tracker_id, &media.content_type)),
                    sync_enabled: true,
                    last_synced: Some(now),
                    created_at: now,
                    updated_at: now,
                });

                ContentImportService::add_cross_mappings(&conn, &cid, &media.cross_ids, &tracker_name, &media.content_type)?;
                Ok(())
            }
        }).await.map_err(|e| CoreError::Internal(e.to_string()))??;

        let is_anime = {
            let conn = db.lock().unwrap();
            ContentRepository::get_content_by_cid(&conn, &cid_owned)?
                .map(|c| c.content_type == ContentType::Anime)
                .unwrap_or(false)
        };

        if is_anime {
            let _ = ContentImportService::enrich_with_simkl(
                db.clone(), state.tracker_registry.clone(), &cid_owned,
            ).await;
        }

        let conn = db.lock().unwrap();
        ContentRepository::get_full_content(&conn, &cid_owned)?
            .ok_or_else(|| CoreError::NotFound("Content not found after link".into()))
    }

    pub async fn resolve_extension_item(
        state: &Arc<AppState>,
        ext_name: &str,
        ext_id: &str,
    ) -> CoreResult<ResolveExtensionResponse> {
        const AUTO_LINK_THRESHOLD: f64 = 0.85;
        const AMBIGUITY_DELTA: f64     = 0.05;

        let db = state.db.connection();

        {
            let conn = db.lock().unwrap();
            if let Some(cid) = ExtensionRepository::find_cid_by_extension(&conn, ext_name, ext_id)? {
                let data = ContentRepository::get_full_content(&conn, &cid)?
                    .ok_or_else(|| CoreError::NotFound("Content not found".into()))?;
                return Ok(ResolveExtensionResponse { success: true, data, tracker_candidates: None, auto_linked: false });
            }
        }

        let ext_meta = state.extension_manager.read().await
            .call_extension_function(ext_name, "getMetadata", vec![json!(ext_id)])
            .await
            .map_err(|e| CoreError::Internal(format!("Extension getMetadata failed: {}", e)))?;

        let title    = ext_meta.get("title").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();
        let year     = ext_meta.get("year").and_then(|v| v.as_i64());
        let nsfw     = ext_meta.get("nsfw").and_then(|v| v.as_bool()).unwrap_or(false);
        let language = ext_meta.get("language").and_then(|v| v.as_str()).map(String::from);

        let content_type = {
            let mgr = state.extension_manager.read().await;
            mgr.list_extensions().iter()
                .find(|e| e.id == ext_name)
                .map(|e| match e.ext_type {
                    ExtensionType::Anime => ContentType::Anime,
                    ExtensionType::Manga => ContentType::Manga,
                    ExtensionType::Novel => ContentType::Novel,
                    _                    => ContentType::Anime,
                })
                .unwrap_or(ContentType::Anime)
        };

        let tracker_content_type = match content_type {
            ContentType::Manga => TrackerContentType::Manga,
            ContentType::Novel => TrackerContentType::Novel,
            _                  => TrackerContentType::Anime,
        };

        let existing_cid = {
            let conn = db.lock().unwrap();
            ContentRepository::find_closest_match(&conn, &title, Some(content_type.clone()), year)?
                .map(|m| m.cid)
        };

        if let Some(cid) = existing_cid {
            let now = chrono::Utc::now().timestamp();
            let conn = db.lock().unwrap();
            let _ = ExtensionRepository::add_source(&conn, &ExtensionSource {
                id: None, cid: cid.clone(), extension_name: ext_name.to_string(),
                extension_id: ext_id.to_string(), nsfw,
                language, created_at: now, updated_at: now,
            });
            let data = ContentRepository::get_full_content(&conn, &cid)?
                .ok_or_else(|| CoreError::NotFound("Content not found".into()))?;
            return Ok(ResolveExtensionResponse { success: true, data, tracker_candidates: None, auto_linked: false });
        }

        let anilist_provider = state.tracker_registry.get("anilist");
        let mut candidates: Vec<(TrackerMedia, f64)> = if let Some(provider) = &anilist_provider {
            match provider.search(Some(&title), tracker_content_type, 5, None, None, None, None).await {
                Ok(results) => results
                    .into_iter()
                    .filter_map(|m| {
                        let score = similarity_score(&title, &m, year);
                        if score >= AUTO_LINK_THRESHOLD - 0.15 { Some((m, score)) } else { None }
                    })
                    .collect(),
                Err(_) => vec![],
            }
        } else { vec![] };
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let best_score   = candidates.first().map(|c| c.1).unwrap_or(0.0);
        let second_score = candidates.get(1).map(|c| c.1).unwrap_or(0.0);
        let auto_linkable = best_score >= AUTO_LINK_THRESHOLD
            && (best_score - second_score) > AMBIGUITY_DELTA;

        if auto_linkable {
            let best_media     = candidates.into_iter().next().map(|(m, _)| m).unwrap();
            let ext_name_s     = ext_name.to_string();
            let ext_id_s       = ext_id.to_string();

            let cid = tokio::task::spawn_blocking({
                let db = db.clone();
                move || -> CoreResult<String> {
                    let conn = db.lock().unwrap();
                    let cid  = ContentImportService::import_media(&conn, "anilist", &best_media)?;
                    let now  = chrono::Utc::now().timestamp();
                    let _ = ExtensionRepository::add_source(&conn, &ExtensionSource {
                        id: None, cid: cid.clone(),
                        extension_name: ext_name_s, extension_id: ext_id_s,
                        nsfw, language,
                        created_at: now, updated_at: now,
                    });
                    Ok(cid)
                }
            }).await.map_err(|e| CoreError::Internal(e.to_string()))??;

            if content_type == ContentType::Anime {
                let _ = ContentImportService::enrich_with_simkl(db.clone(), state.tracker_registry.clone(), &cid).await;
            }

            let data = {
                let conn = db.lock().unwrap();
                ContentRepository::get_full_content(&conn, &cid)?
                    .ok_or_else(|| CoreError::NotFound("Content not found after import".into()))?
            };
            return Ok(ResolveExtensionResponse { success: true, data, tracker_candidates: None, auto_linked: true });
        }

        let tracker_candidates = if candidates.is_empty() {
            None
        } else {
            Some(candidates.into_iter().map(|(m, score)| TrackerCandidate {
                tracker_name: "anilist".to_string(),
                tracker_id:   m.tracker_id,
                title:        m.title,
                cover_image:  m.cover_image,
                score,
            }).collect())
        };

        let cid = {
            let now      = chrono::Utc::now().timestamp();
            let new_cid  = generate_cid();
            let cover    = ext_meta.get("image").or(ext_meta.get("cover")).and_then(|v| v.as_str()).map(String::from);
            let synopsis = ext_meta.get("description").or(ext_meta.get("synopsis")).and_then(|v| v.as_str()).map(String::from);
            let meta = ContentMetadata {
                id: None, cid: new_cid.clone(),
                source_name: ext_name.to_string(),
                source_id: Some(ext_id.to_string()),
                subtype: None, title, alt_titles: vec![], synopsis,
                cover_image: cover, banner_image: None,
                eps_or_chapters: EpisodeData::Count(0), status: None,
                tags: vec![], genres: vec![],
                release_date: year.map(|y| y.to_string()),
                end_date: None, rating: None, trailer_url: None,
                characters: vec![], studio: None, staff: vec![],
                external_ids: json!({}),
                created_at: now, updated_at: now,
            };
            let conn = db.lock().unwrap();
            ContentRepository::create_with_type(&conn, &content_type, nsfw, meta)?;
            ExtensionRepository::add_source(&conn, &ExtensionSource {
                id: None, cid: new_cid.clone(), extension_name: ext_name.to_string(),
                extension_id: ext_id.to_string(), nsfw,
                language, created_at: now, updated_at: now,
            })?;
            new_cid
        };

        let data = {
            let conn = db.lock().unwrap();
            ContentRepository::get_full_content(&conn, &cid)?
                .ok_or_else(|| CoreError::NotFound("Content not found".into()))?
        };

        Ok(ResolveExtensionResponse { success: true, data, tracker_candidates, auto_linked: false })
    }
}

fn similarity_score(query_title: &str, candidate: &TrackerMedia, query_year: Option<i64>) -> f64 {
    let q = normalize_title_svc(query_title);
    let mut best = str_similarity(&q, &normalize_title_svc(&candidate.title));
    for alt in &candidate.alt_titles {
        if alt.trim().is_empty() { continue; }
        let s = str_similarity(&q, &normalize_title_svc(alt));
        if s > best { best = s; }
    }
    if let (Some(qy), Some(release)) = (query_year, &candidate.release_date) {
        if let Ok(dy) = release.chars().take(4).collect::<String>().parse::<i64>() {
            if (qy - dy).abs() > 1 { return best * 0.6; }
        }
    }
    best
}

fn normalize_title_svc(s: &str) -> String {
    s.to_lowercase()
        .replace([':', '-', '!', '?', '.', ',', '\'', '"', '·', '~'], " ")
        .split_whitespace().collect::<Vec<_>>().join(" ")
}

fn str_similarity(s1: &str, s2: &str) -> f64 {
    let a = s1.to_lowercase();
    let b = s2.to_lowercase();
    if a == b { return 1.0; }
    let max_len = a.chars().count().max(b.chars().count());
    if max_len == 0 { return 1.0; }
    let dist = levenshtein(&a, &b);
    1.0 - (dist as f64 / max_len as f64)
}

fn levenshtein(s1: &str, s2: &str) -> usize {
    let v1: Vec<char> = s1.chars().collect();
    let v2: Vec<char> = s2.chars().collect();
    let (len1, len2) = (v1.len(), v2.len());
    let mut col: Vec<usize> = (0..=len1).collect();
    for x in 1..=len2 {
        col[0] = x;
        let mut last = x - 1;
        for y in 1..=len1 {
            let old  = col[y];
            let cost = if v1[y-1] == v2[x-1] { 0 } else { 1 };
            col[y]   = col[y].min(col[y-1].min(last + cost) + 1);
            last     = old;
        }
    }
    col[len1]
}