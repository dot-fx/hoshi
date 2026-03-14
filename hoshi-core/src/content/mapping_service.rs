use std::sync::Arc;
use serde_json::{json, Value};

use crate::content::{
    ContentRepository, ContentType, EpisodeData, ExtensionRepository,
    ExtensionSource, ContentMetadata, ContentWithMappings, generate_cid,
};
use crate::content::resolver::ContentResolverService;
use crate::error::{CoreError, CoreResult};
use crate::extensions::ExtensionType;
use crate::state::AppState;
use crate::tracker::repository::{TrackerMapping, TrackerRepository};
use crate::tracker::provider::ContentType as TrackerContentType;

use super::import_service::ContentImportService;
use super::types::{ExtensionSearchResponse, ResolveExtensionResponse, TrackerCandidate, similarity_score};

const AUTO_LINK_THRESHOLD: f64 = 0.85;
const AMBIGUITY_DELTA: f64     = 0.05;

pub struct ContentService;

impl ContentService {

    // ── Tracker mapping ───────────────────────────────────────────────────────

    pub fn add_tracker_mapping(state: &Arc<AppState>, mut mapping: TrackerMapping) -> CoreResult<()> {
        let db   = state.db.connection();
        let conn = db.lock().unwrap();
        if !TrackerRepository::has_canonical_mapping(&conn, &mapping.cid)? {
            return Err(CoreError::BadRequest(
                "Cannot add tracker mappings to extension-only content".into()
            ));
        }
        let now = chrono::Utc::now().timestamp();
        mapping.created_at = now;
        mapping.updated_at = now;
        TrackerRepository::add_mapping(&conn, &mapping)?;
        Ok(())
    }

    pub fn update_tracker_mapping(
        state: &Arc<AppState>,
        cid: &str,
        tracker_name: &str,
        tracker_id: &str,
    ) -> CoreResult<()> {
        let db   = state.db.connection();
        let conn = db.lock().unwrap();
        if !TrackerRepository::has_canonical_mapping(&conn, cid)? {
            return Err(CoreError::BadRequest("Content is extension-only".into()));
        }
        let now = chrono::Utc::now().timestamp();
        TrackerRepository::add_mapping(&conn, &TrackerMapping {
            cid:          cid.to_string(),
            tracker_name: tracker_name.to_string(),
            tracker_id:   tracker_id.to_string(),
            tracker_url:  None,
            sync_enabled: false,
            last_synced:  None,
            created_at:   now,
            updated_at:   now,
        })?;
        Ok(())
    }

    pub fn delete_tracker_mapping(state: &Arc<AppState>, cid: &str, tracker_name: &str) -> CoreResult<()> {
        let db   = state.db.connection();
        let conn = db.lock().unwrap();
        let rows = TrackerRepository::delete_mapping(&conn, cid, tracker_name)?;
        if rows == 0 { return Err(CoreError::NotFound("Mapping not found".into())); }
        Ok(())
    }

    // ── Extension source ──────────────────────────────────────────────────────

    pub fn add_extension_source(state: &Arc<AppState>, mut source: ExtensionSource) -> CoreResult<i64> {
        let db   = state.db.connection();
        let conn = db.lock().unwrap();
        let now  = chrono::Utc::now().timestamp();
        source.created_at = now;
        source.updated_at = now;
        let id = ExtensionRepository::add_source(&conn, &source)?;
        Ok(id)
    }

    pub async fn update_extension_mapping(
        state: &Arc<AppState>,
        cid: &str,
        ext_name: &str,
        ext_id: &str,
    ) -> CoreResult<ContentWithMappings> {
        let ext_nsfw = state.extension_manager.read().await
            .list_extensions().iter()
            .find(|e| e.id == ext_name)
            .map(|e| e.nsfw)
            .unwrap_or(false);

        let db = state.db.connection();
        {
            let conn = db.lock().unwrap();
            let now  = chrono::Utc::now().timestamp();
            if let Some(id) = ExtensionRepository::find_mapping_id(&conn, cid, ext_name)? {
                ExtensionRepository::update_source(&conn, id, ext_id)?;
            } else {
                ExtensionRepository::add_source(&conn, &ExtensionSource {
                    id: None, cid: cid.to_string(), extension_name: ext_name.to_string(),
                    extension_id: ext_id.to_string(), nsfw: ext_nsfw,
                    language: None, created_at: now, updated_at: now,
                })?;
            }
        }

        super::service::ContentService::save_extension_metadata(state, cid, ext_name, ext_id).await;

        let db2  = state.db.connection();
        let conn = db2.lock().unwrap();
        ContentRepository::get_full_content(&conn, cid)?
            .ok_or_else(|| CoreError::NotFound("Content not found".into()))
    }

    // ── Resolve / link ────────────────────────────────────────────────────────

    pub fn resolve_by_tracker(
        state: &Arc<AppState>,
        tracker: &str,
        id: &str,
    ) -> CoreResult<ContentWithMappings> {
        let db   = state.db.connection();
        let conn = db.lock().unwrap();
        let cid  = TrackerRepository::find_cid_by_tracker(&conn, tracker, id)?
            .ok_or_else(|| CoreError::NotFound("Content mapping not found".into()))?;
        ContentRepository::get_full_content(&conn, &cid)?
            .ok_or_else(|| CoreError::NotFound("Content data missing".into()))
    }

    pub async fn resolve_by_extension(
        state: &Arc<AppState>,
        ext_name: &str,
        ext_id: &str,
    ) -> CoreResult<ContentWithMappings> {
        let content_type = {
            let mgr = state.extension_manager.read().await;
            mgr.list_extensions().iter()
                .find(|e| e.id == ext_name)
                .map(|e| match e.ext_type {
                    ExtensionType::Anime => TrackerContentType::Anime,
                    ExtensionType::Manga => TrackerContentType::Manga,
                    ExtensionType::Novel => TrackerContentType::Novel,
                    ExtensionType::Booru => TrackerContentType::Booru,
                    _                    => TrackerContentType::Anime,
                })
                .ok_or_else(|| CoreError::NotFound("Extension not found".into()))?
        };

        let ext_nsfw = state.extension_manager.read().await
            .list_extensions().iter()
            .find(|e| e.id == ext_name)
            .map(|e| e.nsfw)
            .unwrap_or(false);

        let meta = state.extension_manager.read().await
            .call_extension_function(ext_name, "getMetadata", vec![json!(ext_id)])
            .await
            .map_err(|e| CoreError::Internal(format!("Metadata fetch failed: {}", e)))?;

        let db  = state.db.connection();
        let cid = {
            let conn = db.lock().unwrap();
            match ContentResolverService::resolve_content(&conn, ext_name, ext_id, meta, content_type, ext_nsfw)? {
                crate::content::resolver::ResolutionResult::Canonical { cid } => cid,
                crate::content::resolver::ResolutionResult::Derived { cid }   => cid,
            }
        };

        let conn = db.lock().unwrap();
        ContentRepository::get_full_content(&conn, &cid)?
            .ok_or_else(|| CoreError::NotFound("Resolved content not found".into()))
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
                let conn      = db.lock().unwrap();
                let rich_meta = ContentImportService::to_content_metadata(&cid, &tracker_name, &media);
                ContentRepository::upsert_metadata(&conn, &rich_meta)?;

                let now = chrono::Utc::now().timestamp();
                let _   = TrackerRepository::add_mapping(&conn, &TrackerMapping {
                    cid: cid.clone(),
                    tracker_name: tracker_name.clone(),
                    tracker_id:  media.tracker_id.clone(),
                    tracker_url: Some(ContentImportService::tracker_url(&tracker_name, &media.tracker_id, &media.content_type)),
                    sync_enabled: true,
                    last_synced: Some(now),
                    created_at:  now,
                    updated_at:  now,
                });

                ContentImportService::add_cross_mappings(
                    &conn, &cid, &media.cross_ids, &tracker_name, &media.content_type
                )?;
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

    // ── Extension resolve ─────────────────────────────────────────────────────

    pub async fn resolve_extension_item(
        state: &Arc<AppState>,
        ext_name: &str,
        ext_id: &str,
    ) -> CoreResult<ResolveExtensionResponse> {
        let db = state.db.connection();

        {
            let conn = db.lock().unwrap();
            if let Some(cid) = ExtensionRepository::find_cid_by_extension(&conn, ext_name, ext_id)? {
                let data = ContentRepository::get_full_content(&conn, &cid)?
                    .ok_or_else(|| CoreError::NotFound("Content not found".into()))?;
                return Ok(ResolveExtensionResponse { success: true, data, tracker_candidates: None, auto_linked: false });
            }
        }

        let (ext_nsfw, skip_resolve) = {
            let mgr = state.extension_manager.read().await;
            mgr.list_extensions().iter()
                .find(|e| e.id == ext_name)
                .map(|e| (e.nsfw, e.skip_default_processing))
                .unwrap_or((false, false))
        };

        let ext_meta = state.extension_manager.read().await
            .call_extension_function(ext_name, "getMetadata", vec![json!(ext_id)])
            .await
            .map_err(|e| CoreError::Internal(format!("Extension getMetadata failed: {}", e)))?;

        let title    = ext_meta.get("title").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();
        let year     = ext_meta.get("year").and_then(|v| v.as_i64());
        let nsfw     = ext_nsfw || ext_meta.get("nsfw").and_then(|v| v.as_bool()).unwrap_or(false);
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

        // ── Fast path: cross-id direct link ───────────────────────────────────
        // If getMetadata returns known tracker ids (anilist_id, mal_id, etc.)
        // we can skip fuzzy search entirely — look up the cid directly in the DB
        // or fetch+import from the provider and then link.
        {
            // Map of field names the extension may return → tracker name in our DB
            let cross_id_fields: &[(&str, &str)] = &[
                ("anilist_id", "anilist"),
                ("mal_id",     "myanimelist"),
            ];

            // Collect every cross-id the extension provided — fully owned (String, String)
            // so the Vec is Send and can cross await points safely.
            let provided: Vec<(String, String)> = cross_id_fields.iter()
                .filter_map(|(field, tracker)| {
                    let val = ext_meta.get(*field)?;
                    let id_str = val.as_str()
                        .map(String::from)
                        .or_else(|| val.as_i64().map(|n| n.to_string()))?;
                    if id_str.is_empty() { return None; }
                    Some((tracker.to_string(), id_str))
                })
                .collect();

            if !provided.is_empty() {
                // 1. Check if any of these ids already maps to a cid in the DB.
                //    Lock, query, drop — no await inside this block.
                let existing_cid = {
                    let conn = db.lock().unwrap();
                    provided.iter().find_map(|(tracker, id)| {
                        TrackerRepository::find_cid_by_tracker(&conn, tracker, id).ok().flatten()
                    })
                }; // conn dropped here

                let cid = if let Some(cid) = existing_cid {
                    tracing::debug!(
                        "[resolve_ext] cross-id DB hit for ext={} ext_id={} → cid={}",
                        ext_name, ext_id, cid
                    );
                    cid
                } else {
                    // 2. Not in DB — fetch from provider and import.
                    //    Prefer anilist, fall back to first available cross-id.
                    let (tracker_name, tracker_id) = provided.iter()
                        .find(|(t, _)| t == "anilist")
                        .or_else(|| provided.first())
                        .map(|(t, id)| (t.clone(), id.clone()))
                        .unwrap(); // safe: provided is non-empty

                    let provider = state.tracker_registry.get(&tracker_name);
                    match provider {
                        Some(p) => {
                            // .await with NO locks held
                            match p.get_by_id(&tracker_id).await {
                                Ok(Some(media)) => {
                                    tracing::debug!(
                                        "[resolve_ext] cross-id import: tracker={} id={} title={}",
                                        tracker_name, tracker_id, media.title
                                    );
                                    let db2 = db.clone();
                                    let tn  = tracker_name.clone();
                                    match tokio::task::spawn_blocking(move || {
                                        let conn = db2.lock().unwrap();
                                        ContentImportService::import_media(&conn, &tn, &media)
                                    }).await.map_err(|e| CoreError::Internal(e.to_string()))? {
                                        Ok(cid) => cid,
                                        Err(e) => {
                                            tracing::warn!(
                                                "[resolve_ext] cross-id import failed: {}", e
                                            );
                                            String::new()
                                        }
                                    }
                                }
                                Ok(None) => {
                                    tracing::debug!(
                                        "[resolve_ext] cross-id not found in provider {} id={}",
                                        tracker_name, tracker_id
                                    );
                                    String::new()
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        "[resolve_ext] provider.get_by_id failed tracker={} id={}: {}",
                                        tracker_name, tracker_id, e
                                    );
                                    String::new()
                                }
                            }
                        }
                        None => String::new(),
                    }
                };

                if !cid.is_empty() {
                    // Link the extension source — lock, write, drop before any await.
                    {
                        let now  = chrono::Utc::now().timestamp();
                        let conn = db.lock().unwrap();
                        let _    = ExtensionRepository::add_source(&conn, &ExtensionSource {
                            id: None, cid: cid.clone(),
                            extension_name: ext_name.to_string(),
                            extension_id: ext_id.to_string(),
                            nsfw, language: language.clone(),
                            created_at: now, updated_at: now,
                        });
                    } // conn dropped here — safe to .await below

                    // Simkl enrichment (async) — no lock held
                    if content_type == ContentType::Anime {
                        let _ = ContentImportService::enrich_with_simkl(
                            db.clone(), state.tracker_registry.clone(), &cid,
                        ).await;
                    }

                    // Final read — fresh lock after all awaits
                    let data = {
                        let conn = db.lock().unwrap();
                        ContentRepository::get_full_content(&conn, &cid)?
                            .ok_or_else(|| CoreError::NotFound("Content not found".into()))?
                    }; // conn dropped
                    return Ok(ResolveExtensionResponse {
                        success: true,
                        data,
                        tracker_candidates: None,
                        auto_linked: true,
                    });
                }
            }
        }

        // ── skip_resolve path ─────────────────────────────────────────────────
        if skip_resolve {
            let existing_cid = {
                let conn = db.lock().unwrap();
                ContentRepository::find_closest_match(&conn, &title, Some(content_type.clone()), year)?
                    .map(|m| m.cid)
            };

            let cid = if let Some(cid) = existing_cid {
                cid
            } else {
                let now      = chrono::Utc::now().timestamp();
                let new_cid  = generate_cid();
                let cover    = ext_meta.get("image").or(ext_meta.get("cover")).and_then(|v| v.as_str()).map(String::from);
                let synopsis = ext_meta.get("description").or(ext_meta.get("synopsis")).and_then(|v| v.as_str()).map(String::from);
                let genres   = ext_meta.get("genres").and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_default();
                let meta = ContentMetadata {
                    id: None, cid: new_cid.clone(),
                    source_name: ext_name.to_string(), source_id: Some(ext_id.to_string()),
                    subtype: None, title, alt_titles: vec![], synopsis,
                    cover_image: cover, banner_image: None,
                    eps_or_chapters: EpisodeData::Count(0), status: None,
                    tags: vec![], genres,
                    release_date: year.map(|y| y.to_string()), end_date: None,
                    rating: None, trailer_url: None, characters: vec![], studio: None,
                    staff: vec![], external_ids: json!({}),
                    created_at: now, updated_at: now,
                };
                let conn = db.lock().unwrap();
                ContentRepository::create_with_type(&conn, &content_type, nsfw, meta)?;
                new_cid
            };

            {
                let now  = chrono::Utc::now().timestamp();
                let conn = db.lock().unwrap();
                let _    = ExtensionRepository::add_source(&conn, &ExtensionSource {
                    id: None, cid: cid.clone(), extension_name: ext_name.to_string(),
                    extension_id: ext_id.to_string(), nsfw,
                    language, created_at: now, updated_at: now,
                });
            }

            let data = {
                let conn = db.lock().unwrap();
                ContentRepository::get_full_content(&conn, &cid)?
                    .ok_or_else(|| CoreError::NotFound("Content not found".into()))?
            };
            return Ok(ResolveExtensionResponse { success: true, data, tracker_candidates: None, auto_linked: false });
        }

        // ── Normal path: attempt tracker resolution ───────────────────────────

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
            let now  = chrono::Utc::now().timestamp();
            let conn = db.lock().unwrap();
            let _    = ExtensionRepository::add_source(&conn, &ExtensionSource {
                id: None, cid: cid.clone(), extension_name: ext_name.to_string(),
                extension_id: ext_id.to_string(), nsfw,
                language, created_at: now, updated_at: now,
            });
            let data = ContentRepository::get_full_content(&conn, &cid)?
                .ok_or_else(|| CoreError::NotFound("Content not found".into()))?;
            return Ok(ResolveExtensionResponse { success: true, data, tracker_candidates: None, auto_linked: false });
        }

        let anilist_provider = state.tracker_registry.get("anilist");
        let mut candidates: Vec<(crate::tracker::provider::TrackerMedia, f64)> =
            if let Some(provider) = &anilist_provider {
                match provider.search(Some(&title), tracker_content_type, 5, None, None, None, None).await {
                    Ok(results) => results.into_iter()
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
            let best_media = candidates.into_iter().next().map(|(m, _)| m).unwrap();
            let ext_name_s = ext_name.to_string();
            let ext_id_s   = ext_id.to_string();

            let cid = tokio::task::spawn_blocking({
                let db = db.clone();
                move || -> CoreResult<String> {
                    let conn = db.lock().unwrap();
                    let cid  = ContentImportService::import_media(&conn, "anilist", &best_media)?;
                    let now  = chrono::Utc::now().timestamp();
                    let _    = ExtensionRepository::add_source(&conn, &ExtensionSource {
                        id: None, cid: cid.clone(),
                        extension_name: ext_name_s, extension_id: ext_id_s,
                        nsfw, language,
                        created_at: now, updated_at: now,
                    });
                    Ok(cid)
                }
            }).await.map_err(|e| CoreError::Internal(e.to_string()))??;

            if content_type == ContentType::Anime {
                let _ = ContentImportService::enrich_with_simkl(
                    db.clone(), state.tracker_registry.clone(), &cid,
                ).await;
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
            let now     = chrono::Utc::now().timestamp();
            let new_cid = generate_cid();
            let cover   = ext_meta.get("image").or(ext_meta.get("cover")).and_then(|v| v.as_str()).map(String::from);
            let synopsis = ext_meta.get("description").or(ext_meta.get("synopsis")).and_then(|v| v.as_str()).map(String::from);
            let meta = ContentMetadata {
                id: None, cid: new_cid.clone(),
                source_name: ext_name.to_string(), source_id: Some(ext_id.to_string()),
                subtype: None, title, alt_titles: vec![], synopsis,
                cover_image: cover, banner_image: None,
                eps_or_chapters: EpisodeData::Count(0), status: None,
                tags: vec![], genres: vec![],
                release_date: year.map(|y| y.to_string()), end_date: None,
                rating: None, trailer_url: None, characters: vec![], studio: None,
                staff: vec![], external_ids: json!({}),
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

    // ── Direct extension search ───────────────────────────────────────────────

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

        let ext_nsfw = state.extension_manager.read().await
            .list_extensions().iter()
            .find(|e| e.id == ext_name)
            .map(|e| e.nsfw)
            .unwrap_or(false);

        let raw = state.extension_manager.read().await
            .call_extension_function(ext_name, "search", vec![args])
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?;

        let results = match raw {
            Value::Array(mut arr) => {
                for item in &mut arr {
                    if let Some(obj) = item.as_object_mut() {
                        let item_nsfw = obj.get("nsfw").and_then(|v| v.as_bool()).unwrap_or(false);
                        obj.insert("nsfw".to_string(), json!(ext_nsfw || item_nsfw));
                    }
                }
                Value::Array(arr)
            }
            other => other,
        };

        Ok(ExtensionSearchResponse { success: true, results })
    }
}