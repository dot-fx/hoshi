use std::sync::Arc;
use serde_json::{json, Value};

use crate::config::repository::ConfigRepo;
use crate::content::{
    CacheRepository, ContentRepository, ContentStatus, ContentType,
    EpisodeData, ExtensionRepository, ExtensionSource, ContentMetadata,
    ContentWithMappings, generate_cid,
};
use crate::content::resolver::ContentResolverService;
use crate::error::{CoreError, CoreResult};
use crate::extensions::ExtensionType;
use crate::state::AppState;
use crate::tracker::repository::{TrackerMapping, TrackerRepository};
use crate::tracker::provider::ContentType as TrackerContentType;

use super::import_service::ContentImportService;
use super::types::{
    ContentListResult, ExtensionSearchResponse, SearchParams,
    parse_content_type, str_similarity,
};

pub struct ContentService;

impl ContentService {

    fn show_adult(state: &Arc<AppState>, user_id: Option<i32>) -> bool {
        let uid = match user_id {
            Some(id) => id,
            None => return false,
        };
        let conn = state.db.connection();
        let Ok(lock) = conn.lock() else { return false };
        ConfigRepo::get_config(&lock, uid)
            .map(|c| c.general.show_adult_content)
            .unwrap_or(false)
    }

    // ── Extension metadata ────────────────────────────────────────────────────

    pub(super) async fn save_extension_metadata(
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

    // ── CRUD ──────────────────────────────────────────────────────────────────

    pub async fn create_content(
        state: &Arc<AppState>,
        content_type: ContentType,
        nsfw: bool,
        meta: ContentMetadata,
        trackers: Option<Vec<crate::tracker::repository::TrackerMapping>>,
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

    pub async fn get_content(
        state: &Arc<AppState>,
        cid: &str,
    ) -> CoreResult<ContentWithMappings> {
        let db  = state.db.connection();
        let cid = cid.to_string();

        let (content_type, needs_enrichment, tracker_id, lacks_simkl, is_releasing) =
            tokio::task::spawn_blocking({
                let db  = db.clone();
                let cid = cid.clone();
                move || -> CoreResult<_> {
                    let conn     = db.lock().map_err(|_| CoreError::Internal("DB Lock".into()))?;
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

        if is_releasing {
            let state_bg = state.clone();
            let cid_bg   = cid.clone();
            let al_id_bg = tracker_id.clone();

            tokio::spawn(async move {
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

                let extensions = {
                    let db   = state_bg.db.connection();
                    let conn = db.lock().unwrap();
                    ExtensionRepository::get_by_cid(&conn, &cid_bg).unwrap_or_default()
                };

                for source in extensions {
                    let ext_name  = source.extension_name.clone();
                    let ext_id    = source.extension_id.clone();
                    let ct = {
                        let db   = state_bg.db.connection();
                        let conn = db.lock().unwrap();
                        ExtensionRepository::get_extension_id_and_type(&conn, &cid_bg, &ext_name)
                            .ok().flatten()
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
                            let db   = state_bg.db.connection();
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
        let db   = state.db.connection();
        let conn = db.lock().unwrap();
        ContentRepository::upsert_metadata(&conn, &meta)?;
        ContentRepository::get_full_content(&conn, cid)?
            .ok_or_else(|| CoreError::NotFound("Content not found after update".into()))
    }

    // ── Search ────────────────────────────────────────────────────────────────

    pub async fn search_content(
        state: &Arc<AppState>,
        params: SearchParams,
        user_id: Option<i32>,
    ) -> CoreResult<ContentListResult> {
        let show_adult = Self::show_adult(state, user_id);
        let query_str  = params.query.clone().unwrap_or_default();

        if let Some(ext_name) = params.extension.clone() {
            let filters = params.extension_filters.as_deref().unwrap_or("{}");

            let skip = state.extension_manager.read().await
                .list_extensions().iter()
                .find(|e| e.id == ext_name)
                .map(|e| e.skip_default_processing)
                .unwrap_or(false);

            return if skip || !query_str.is_empty() || filters != "{}" {
                let ct = params.r#type.as_deref().map(parse_content_type);
                let mut results = Self::search_via_extension(
                    state, query_str, ext_name, ct, params.extension_filters.clone(),
                ).await?;
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

        let db   = state.db.connection();
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
        let db   = state.db.connection();

        let ext_nsfw = state.extension_manager.read().await
            .list_extensions().iter()
            .find(|e| e.id == ext_name)
            .map(|e| e.nsfw)
            .unwrap_or(false);

        let skip_resolve = state.extension_manager.read().await
            .list_extensions().iter()
            .find(|e| e.id == ext_name)
            .map(|e| e.skip_default_processing)
            .unwrap_or(false);

        let ext_content_type = state.extension_manager.read().await
            .list_extensions().iter()
            .find(|e| e.id == ext_name)
            .map(|e| match e.ext_type {
                ExtensionType::Manga => ContentType::Manga,
                ExtensionType::Novel => ContentType::Novel,
                _                    => ContentType::Anime,
            })
            .unwrap_or(ContentType::Anime);

        let raw_items = match state.extension_manager.read().await
            .call_extension_function(&ext_name, "search", vec![args])
            .await
        {
            Ok(Value::Array(arr)) => arr,
            _ => return Ok(vec![]),
        };

        let items: Vec<Value> = raw_items.into_iter().map(|mut item| {
            if let Some(obj) = item.as_object_mut() {
                let item_nsfw = obj.get("nsfw").and_then(|v| v.as_bool()).unwrap_or(false);
                obj.insert("nsfw".to_string(), json!(ext_nsfw || item_nsfw));
            }
            item
        }).collect();

        let mut resolved = Vec::new();
        for item in &items {
            let ext_id = match item.get("id").and_then(|v| v.as_str()) {
                Some(id) if !id.is_empty() => id.to_string(),
                _ => continue,
            };

            let cid = if skip_resolve {
                let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();
                let cover = item.get("image").and_then(|v| v.as_str()).map(String::from);
                let nsfw  = item.get("nsfw").and_then(|v| v.as_bool()).unwrap_or(ext_nsfw);

                let conn = db.lock().unwrap();

                if let Some(existing) = ExtensionRepository::find_cid_by_extension(&conn, &ext_name, &ext_id)? {
                    existing
                } else {
                    let now     = chrono::Utc::now().timestamp();
                    let new_cid = generate_cid();
                    let meta = ContentMetadata {
                        id: None, cid: new_cid.clone(),
                        source_name: ext_name.clone(), source_id: Some(ext_id.clone()),
                        subtype: None, title, alt_titles: vec![], synopsis: None,
                        cover_image: cover, banner_image: None,
                        eps_or_chapters: EpisodeData::Count(0), status: None,
                        tags: vec![], genres: vec![],
                        release_date: None, end_date: None, rating: None,
                        trailer_url: None, characters: vec![], studio: None,
                        staff: vec![], external_ids: json!({}),
                        created_at: now, updated_at: now,
                    };
                    ContentRepository::create_with_type(&conn, &ext_content_type, nsfw, meta)?;
                    ExtensionRepository::add_source(&conn, &ExtensionSource {
                        id: None, cid: new_cid.clone(), extension_name: ext_name.clone(),
                        extension_id: ext_id.clone(), nsfw,
                        language: None, created_at: now, updated_at: now,
                    })?;
                    new_cid
                }
            } else {
                let inferred_type = content_type.clone().unwrap_or(TrackerContentType::Anime);
                let conn = db.lock().unwrap();
                let res  = ContentResolverService::resolve_content(
                    &conn, &ext_name, &ext_id, item.clone(), inferred_type, ext_nsfw,
                )?;
                match res {
                    crate::content::resolver::ResolutionResult::Canonical { cid } => cid,
                    crate::content::resolver::ResolutionResult::Derived { cid }   => cid,
                }
            };

            let conn = db.lock().unwrap();
            if let Ok(Some(full)) = ContentRepository::get_full_content(&conn, &cid) {
                resolved.push(full);
            }
        }

        Ok(resolved)
    }

    // ── Items / play ──────────────────────────────────────────────────────────

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
            let ct  = serde_json::from_str::<ContentType>(&format!("\"{}\"", type_str))
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
                let conn    = db.lock().unwrap();
                let content = ContentRepository::get_content_by_cid(&conn, cid)?
                    .ok_or_else(|| CoreError::NotFound("Content not found".into()))?;
                let meta    = ContentRepository::get_by_cid(&conn, cid)?
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
                            let id         = item.get("id")?.as_str()?;
                            let item_title = item.get("title")?.as_str()?;
                            let score      = str_similarity(&title, item_title);
                            Some((id.to_string(), score))
                        })
                        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                })
                .filter(|(_, score)| *score >= 0.6)
                .map(|(id, _)| id)
                .ok_or_else(|| CoreError::NotFound(format!("No match found in {} for '{}'", ext_name, title)))?;

            {
                let ext_nsfw = state.extension_manager.read().await
                    .list_extensions().iter()
                    .find(|e| e.id == ext_name)
                    .map(|e| e.nsfw)
                    .unwrap_or(false);
                let now  = chrono::Utc::now().timestamp();
                let conn = db.lock().unwrap();
                if let Err(e) = ExtensionRepository::add_source(&conn, &ExtensionSource {
                    id: None, cid: cid.to_string(), extension_name: ext_name.to_string(),
                    extension_id: best_id.clone(), nsfw: ext_nsfw,
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

        let func  = match ct { ContentType::Anime => "findEpisodes", _ => "findChapters" };
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

            super::mapping_service::ContentService::resolve_extension_item(state, ext_name, &found_ext_id).await?;
            Self::save_extension_metadata(state, cid, ext_name, &found_ext_id).await;
        }

        let (ct, ext_id) = {
            let conn    = db.lock().unwrap();
            let (t, id) = ExtensionRepository::get_extension_id_and_type(&conn, cid, ext_name)?
                .ok_or_else(|| CoreError::Internal(format!(
                    "Auto-link failed: no link found for cid='{}' ext='{}'", cid, ext_name
                )))?;
            let ct = serde_json::from_str::<ContentType>(&format!("\"{}\"", t))
                .unwrap_or(ContentType::Anime);
            (ct, id)
        };

        let cache_key = format!("items:{}:{}", ext_name, ext_id);
        let cached    = { let conn = db.lock().unwrap(); CacheRepository::get(&conn, &cache_key)? };

        let func       = match ct { ContentType::Anime => "findEpisodes", _ => "findChapters" };
        let items_list = match cached {
            Some(d) => d,
            None    => state.extension_manager.read().await
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
                let srv  = server.unwrap_or_else(|| "default".into());
                let cat  = category.unwrap_or_else(|| "sub".into());
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
}