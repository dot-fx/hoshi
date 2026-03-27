use chrono::Utc;
use rusqlite::Connection;
use serde_json::{json, Value};
use uuid::Uuid;
use tracing::{info, warn, debug, instrument};

use crate::content::{
    ContentRepository, ContentMetadata, ExtensionRepository, ContentType, EpisodeData, ExtensionSource,
};
use crate::tracker::repository::TrackerRepository;
use crate::error::CoreResult;

pub struct ContentResolverService;

#[derive(Debug)]
pub enum ResolutionResult {
    Canonical { cid: String },
    Derived { cid: String },
}

impl ContentResolverService {

    #[instrument(skip(conn, ext_metadata))]
    pub fn resolve_content(
        conn: &Connection,
        ext_name: &str,
        ext_id: &str,
        ext_metadata: Value,
        content_type: ContentType,
        ext_nsfw: bool,
    ) -> CoreResult<ResolutionResult> {
        if let Some(existing_cid) = ExtensionRepository::find_cid_by_extension(conn, ext_name, ext_id)? {
            debug!(cid = %existing_cid, "Found existing direct mapping for extension item");
            return Ok(ResolutionResult::Canonical { cid: existing_cid });
        }

        let title = ext_metadata.get("title").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();
        let year  = ext_metadata.get("year").and_then(|v| v.as_i64());
        let nsfw  = ext_nsfw || ext_metadata.get("nsfw").and_then(|v| v.as_bool()).unwrap_or(false);

        if let Some(tracker_cid) = Self::find_by_external_ids(conn, &ext_metadata, &content_type)? {
            info!(cid = %tracker_cid, title = %title, "Content resolved via external IDs (cross-reference)");
            Self::link_extension_to_cid(conn, &tracker_cid, ext_name, ext_id, nsfw)?;
            return Ok(ResolutionResult::Canonical { cid: tracker_cid });
        }

        if let Some(matched_meta) = ContentRepository::find_closest_match(conn, &title, Some(content_type.clone()), year)? {
            info!(cid = %matched_meta.cid, title = %title, "Content resolved via title similarity match");
            Self::link_extension_to_cid(conn, &matched_meta.cid, ext_name, ext_id, nsfw)?;
            return Ok(ResolutionResult::Canonical { cid: matched_meta.cid });
        }

        debug!(ext = %ext_name, title = %title, "No match found, creating new derived content entry");
        let new_cid = Self::create_derived_content(conn, ext_name, ext_id, ext_metadata, content_type, nsfw)?;

        info!(cid = %new_cid, "New derived content created successfully");
        Ok(ResolutionResult::Derived { cid: new_cid })
    }

    #[instrument(skip(conn, ext_metadata))]
    fn find_by_external_ids(
        conn: &Connection,
        ext_metadata: &Value,
        expected_type: &ContentType,
    ) -> CoreResult<Option<String>> {
        let external_ids = ext_metadata.get("externalIds")
            .or(ext_metadata.get("external_ids"))
            .and_then(|v| v.as_object());

        let Some(ids) = external_ids else { return Ok(None) };

        for (tracker, id_val) in ids {
            let id_str = id_val.as_str()
                .map(|s| s.to_string())
                .or_else(|| id_val.as_i64().map(|i| i.to_string()));

            if let Some(id_str) = id_str {
                let tracker_lower = tracker.to_lowercase();

                if let Some(cid) = TrackerRepository::find_cid_by_tracker(conn, &tracker_lower, &id_str)? {
                    match ContentRepository::get_content_by_cid(conn, &cid)? {
                        Some(content) if content.content_type == *expected_type => {
                            debug!(tracker = %tracker_lower, external_id = %id_str, cid = %cid, "Found matching canonical content via cross-id");
                            return Ok(Some(cid));
                        }
                        Some(content) => {
                            warn!(
                                tracker = %tracker_lower,
                                id = %id_str,
                                cid = %cid,
                                "External ID match discarded: type mismatch (expected {:?}, found {:?})",
                                expected_type, content.content_type
                            );
                        }
                        None => {
                            warn!(cid = %cid, "Orphan tracker mapping found (no corresponding content row)");
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    #[instrument(skip(conn))]
    fn link_extension_to_cid(
        conn: &Connection,
        cid: &str,
        ext_name: &str,
        ext_id: &str,
        nsfw: bool,
    ) -> CoreResult<()> {
        let now = Utc::now().timestamp();
        let source = ExtensionSource {
            id: None,
            cid: cid.to_string(),
            extension_name: ext_name.to_string(),
            extension_id: ext_id.to_string(),
            nsfw,
            language: None,
            created_at: now,
            updated_at: now,
        };

        debug!(cid = %cid, ext = %ext_name, "Linking extension item to existing content");
        ExtensionRepository::add_source(conn, &source)?;
        Ok(())
    }

    #[instrument(skip(conn, meta))]
    fn create_derived_content(
        conn: &Connection,
        ext_name: &str,
        ext_id: &str,
        meta: Value,
        c_type: ContentType,
        nsfw: bool,
    ) -> CoreResult<String> {
        let now = Utc::now().timestamp();
        let cid = Uuid::new_v4().to_string();

        let title = meta.get("title").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();

        let content_metadata = ContentMetadata {
            id: None,
            cid: cid.clone(),
            source_name: ext_name.to_string(),
            source_id: Some(ext_id.to_string()),
            subtype: None,
            title,
            alt_titles: vec![],
            title_i18n: Default::default(),
            synopsis: meta.get("description").or(meta.get("synopsis"))
                .and_then(|v| v.as_str()).map(String::from),
            cover_image: meta.get("image").or(meta.get("cover"))
                .and_then(|v| v.as_str()).map(String::from),
            banner_image: None,
            eps_or_chapters: EpisodeData::Count(0),
            status: None,
            tags: vec![],
            genres: vec![],
            release_date: meta.get("year").and_then(|v| v.as_i64()).map(|y| y.to_string()),
            end_date: None,
            rating: None,
            trailer_url: None,
            characters: vec![],
            studio: None,
            staff: vec![],
            external_ids: json!({}),
            created_at: now,
            updated_at: now,
        };

        ContentRepository::create_with_type(conn, &c_type, nsfw, content_metadata)?;
        Self::link_extension_to_cid(conn, &cid, ext_name, ext_id, nsfw)?;

        Ok(cid)
    }
}