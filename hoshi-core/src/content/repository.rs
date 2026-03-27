use rusqlite::{params, Connection, OptionalExtension, Row};
use tracing::{debug, instrument};

use crate::error::{CoreResult};
use crate::tracker::repository::TrackerRepository;

use super::models::{
    Content, ContentMetadata, ContentType, ContentWithMappings,
    EpisodeData, normalize_title, similarity,
};
use super::extension_repository::ExtensionRepository;
use super::aux_repositories::{RelationRepository, UnitRepository};

pub struct ContentRepository;

impl ContentRepository {
    #[instrument(skip(conn, meta))]
    pub fn create(conn: &Connection, meta: ContentMetadata) -> CoreResult<String> {
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            r#"
            INSERT INTO content (cid, type, nsfw, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(cid) DO NOTHING
            "#,
            params![
                meta.cid,
                "anime",
                if meta.tags.is_empty() { 0i32 } else { 0i32 },
                now,
                now,
            ],
        ).ok();

        Self::upsert_metadata(conn, &meta)?;
        Ok(meta.cid)
    }

    #[instrument(skip(conn, meta))]
    pub fn create_with_type(
        conn: &Connection,
        content_type: &ContentType,
        nsfw: bool,
        meta: ContentMetadata,
    ) -> CoreResult<String> {
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            r#"
            INSERT INTO content (cid, type, nsfw, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(cid) DO NOTHING
            "#,
            params![
                meta.cid,
                content_type.as_str(),
                if nsfw { 1 } else { 0 },
                now,
                now,
            ],
        )?;

        Self::upsert_metadata(conn, &meta)?;
        Ok(meta.cid)
    }

    #[instrument(skip(conn, meta))]
    pub fn upsert_metadata(conn: &Connection, meta: &ContentMetadata) -> CoreResult<()> {
        let now = chrono::Utc::now().timestamp();
        debug!(cid = %meta.cid, source = %meta.source_name, "Upserting content metadata");

        conn.execute(
            r#"
            INSERT INTO metadata (
                cid, source_name, source_id, subtype, title, alt_titles, title_i18n, synopsis,
                cover_image, banner_image, eps_or_chapters, status, tags, genres,
                release_date, end_date, rating, trailer_url, characters, studio,
                staff, external_ids, created_at, updated_at
            ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24)
            ON CONFLICT(cid, source_name) DO UPDATE SET
                source_id       = excluded.source_id,
                subtype         = excluded.subtype,
                title           = excluded.title,
                alt_titles      = excluded.alt_titles,
                title_i18n      = CASE
                                    WHEN excluded.title_i18n != '{}' THEN excluded.title_i18n
                                    ELSE metadata.title_i18n
                                  END,
                synopsis        = COALESCE(excluded.synopsis, metadata.synopsis),
                cover_image     = COALESCE(excluded.cover_image, metadata.cover_image),
                banner_image    = COALESCE(excluded.banner_image, metadata.banner_image),
                eps_or_chapters = excluded.eps_or_chapters,
                status          = excluded.status,
                tags            = excluded.tags,
                genres          = excluded.genres,
                release_date    = excluded.release_date,
                end_date        = excluded.end_date,
                rating          = COALESCE(excluded.rating, metadata.rating),
                trailer_url     = COALESCE(excluded.trailer_url, metadata.trailer_url),
                characters      = excluded.characters,
                studio          = excluded.studio,
                staff           = excluded.staff,
                external_ids    = excluded.external_ids,
                updated_at      = excluded.updated_at
            "#,
            params![
                meta.cid,
                meta.source_name,
                meta.source_id,
                meta.subtype,
                meta.title,
                serde_json::to_string(&meta.alt_titles)?,
                serde_json::to_string(&meta.title_i18n)?,
                meta.synopsis,
                meta.cover_image,
                meta.banner_image,
                serde_json::to_string(&meta.eps_or_chapters)?,
                meta.status.as_ref().map(|s| serde_json::to_string(s).unwrap()),
                serde_json::to_string(&meta.tags)?,
                serde_json::to_string(&meta.genres)?,
                meta.release_date,
                meta.end_date,
                meta.rating,
                meta.trailer_url,
                serde_json::to_string(&meta.characters)?,
                meta.studio,
                serde_json::to_string(&meta.staff)?,
                meta.external_ids.to_string(),
                now,
                now,
            ],
        )?;
        Ok(())
    }

    pub fn get_content_by_cid(conn: &Connection, cid: &str) -> CoreResult<Option<Content>> {
        conn.query_row(
            "SELECT cid, type, nsfw, created_at, updated_at FROM content WHERE cid = ?1",
            params![cid],
            |row| {
                Ok(Content {
                    cid: row.get(0)?,
                    content_type: serde_json::from_str(
                        &format!("\"{}\"", row.get::<_, String>(1)?)
                    ).unwrap_or(ContentType::Anime), // Fallback seguro
                    nsfw: row.get::<_, i32>(2)? == 1,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            },
        )
            .optional()
            .map_err(Into::into)
    }

    pub fn get_metadata_by_source(
        conn: &Connection,
        cid: &str,
        source_name: &str,
    ) -> CoreResult<Option<ContentMetadata>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM metadata WHERE cid = ?1 AND source_name = ?2",
        )?;
        stmt.query_row(params![cid, source_name], |row| Self::row_to_metadata(row))
            .optional()
            .map_err(Into::into)
    }

    #[instrument(skip(conn))]
    pub fn get_all_metadata(conn: &Connection, cid: &str) -> CoreResult<Vec<ContentMetadata>> {
        debug!(cid = %cid, "Fetching all metadata sources for content");
        let mut stmt = conn.prepare(
            "SELECT * FROM metadata WHERE cid = ?1 \
             ORDER BY CASE source_name WHEN 'anilist' THEN 0 ELSE 1 END",
        )?;
        let rows = stmt.query_map(params![cid], |row| Self::row_to_metadata(row))?;
        let mut results = Vec::new();
        for row in rows { results.push(row?); }
        Ok(results)
    }

    pub fn get_by_cid(conn: &Connection, cid: &str) -> CoreResult<Option<ContentMetadata>> {
        let all = Self::get_all_metadata(conn, cid)?;
        Ok(all.into_iter().next())
    }

    pub fn update(conn: &Connection, meta: &ContentMetadata) -> CoreResult<()> {
        Self::upsert_metadata(conn, meta)
    }

    #[instrument(skip(conn))]
    pub fn find_closest_match(
        conn: &Connection,
        title: &str,
        content_type: Option<ContentType>,
        release_year: Option<i64>,
    ) -> CoreResult<Option<ContentMetadata>> {
        let content_type = match content_type {
            Some(t) => t,
            None => return Ok(None),
        };

        debug!(title = %title, content_type = content_type.as_str(), year = ?release_year, "Searching for closest title match in DB");

        let (sql, param_refs_owned): (String, Vec<String>) = if let Some(year) = release_year {
            (
                "SELECT m.cid, m.title, m.alt_titles, m.title_i18n, m.release_date \
                 FROM metadata m \
                 JOIN content c ON c.cid = m.cid \
                 WHERE c.type = ?1 AND (\
                     m.release_date IS NULL \
                     OR CAST(substr(m.release_date, 1, 4) AS INTEGER) BETWEEN ?2 AND ?3\
                 ) \
                 GROUP BY m.cid"
                    .to_string(),
                vec![
                    content_type.as_str().to_string(),
                    (year - 1).to_string(),
                    (year + 1).to_string(),
                ],
            )
        } else {
            (
                "SELECT m.cid, m.title, m.alt_titles, m.title_i18n, m.release_date \
                 FROM metadata m \
                 JOIN content c ON c.cid = m.cid \
                 WHERE c.type = ?1 \
                 GROUP BY m.cid"
                    .to_string(),
                vec![content_type.as_str().to_string()],
            )
        };

        let mut stmt = conn.prepare(&sql)?;
        let param_refs: Vec<&dyn rusqlite::ToSql> =
            param_refs_owned.iter().map(|p| p as &dyn rusqlite::ToSql).collect();

        let rows = stmt.query_map(&param_refs[..], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?;

        let target_normalized = normalize_title(title);
        let mut best_match: Option<String> = None;
        let mut highest_score = 0.0_f64;
        const THRESHOLD: f64 = 0.85;

        for row in rows {
            let (cid, db_title, db_alt_titles_json, db_title_i18n_json) = row?;
            let mut max_local_score = similarity(&target_normalized, &normalize_title(&db_title));

            if let Ok(alt_titles) = serde_json::from_str::<Vec<String>>(&db_alt_titles_json) {
                for alt in alt_titles {
                    if alt.trim().is_empty() { continue; }
                    let alt_score = similarity(&target_normalized, &normalize_title(&alt));
                    if alt_score > max_local_score { max_local_score = alt_score; }
                }
            }

            if let Ok(i18n) = serde_json::from_str::<std::collections::HashMap<String, String>>(&db_title_i18n_json) {
                for (_, localized) in i18n {
                    if localized.trim().is_empty() { continue; }
                    let score = similarity(&target_normalized, &normalize_title(&localized));
                    if score > max_local_score { max_local_score = score; }
                }
            }

            if max_local_score >= THRESHOLD && max_local_score > highest_score {
                highest_score = max_local_score;
                best_match = Some(cid);
            }
        }

        if let Some(cid) = best_match {
            debug!(cid = %cid, score = %highest_score, "Closest match found");
            return Self::get_by_cid(conn, &cid);
        }

        debug!("No close match found in local database");
        Ok(None)
    }

    #[instrument(skip(conn))]
    pub fn get_full_content(conn: &Connection, cid: &str) -> CoreResult<Option<ContentWithMappings>> {
        debug!(cid = %cid, "Assembling full content object with mappings");
        let content = match Self::get_content_by_cid(conn, cid)? {
            Some(c) => c,
            None => return Ok(None),
        };

        let metadata        = Self::get_all_metadata(conn, cid)?;
        let tracker_mappings  = TrackerRepository::get_mappings_by_cid(conn, cid)?;
        let extension_sources = ExtensionRepository::get_by_cid(conn, cid)?;
        let relations         = RelationRepository::get_by_source(conn, cid)?;
        let content_units     = UnitRepository::get_by_cid(conn, cid)?;

        Ok(Some(ContentWithMappings {
            content,
            metadata,
            tracker_mappings,
            extension_sources,
            relations,
            content_units,
        }))
    }

    fn row_to_metadata(row: &Row) -> rusqlite::Result<ContentMetadata> {
        Ok(ContentMetadata {
            id:              row.get(0)?,
            cid:             row.get(1)?,
            source_name:     row.get(2)?,
            source_id:       row.get(3)?,
            subtype:         row.get(4)?,
            title:           row.get(5)?,
            alt_titles:      serde_json::from_str(&row.get::<_, String>(6)?).unwrap_or_default(),
            title_i18n:      serde_json::from_str(&row.get::<_, String>(7)?).unwrap_or_default(),
            synopsis:        row.get(8)?,
            cover_image:     row.get(9)?,
            banner_image:    row.get(10)?,
            eps_or_chapters: serde_json::from_str(&row.get::<_, String>(11)?)
                .unwrap_or(EpisodeData::Count(0)),
            status:          row.get::<_, Option<String>>(12)?
                .map(|s| serde_json::from_str(&s).unwrap()),
            tags:            serde_json::from_str(&row.get::<_, String>(13)?).unwrap_or_default(),
            genres:          serde_json::from_str(&row.get::<_, String>(14)?).unwrap_or_default(),
            release_date:    row.get(15)?,
            end_date:        row.get(16)?,
            rating:          row.get(17)?,
            trailer_url:     row.get(18)?,
            characters:      serde_json::from_str(&row.get::<_, String>(19)?).unwrap_or_default(),
            studio:          row.get(20)?,
            staff:           serde_json::from_str(&row.get::<_, String>(21)?).unwrap_or_default(),
            external_ids:    serde_json::from_str(&row.get::<_, String>(22)?)
                .unwrap_or(serde_json::json!({})),
            created_at:      row.get(23)?,
            updated_at:      row.get(24)?,
        })
    }
}