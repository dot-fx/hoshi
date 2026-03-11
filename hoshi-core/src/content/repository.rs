use crate::error::CoreResult;
use crate::tracker::repository::{TrackerMapping, TrackerRepository};
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde_json;
use std::cmp::min;
use std::fmt;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    pub cid: String,
    pub content_type: ContentType,
    pub nsfw: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentMetadata {
    pub id: Option<i64>,
    pub cid: String,
    pub source_name: String,
    pub source_id: Option<String>,
    pub subtype: Option<String>,
    pub title: String,
    pub alt_titles: Vec<String>,
    pub synopsis: Option<String>,
    pub cover_image: Option<String>,
    pub banner_image: Option<String>,
    pub eps_or_chapters: EpisodeData,
    pub status: Option<ContentStatus>,
    pub tags: Vec<String>,
    pub genres: Vec<String>,
    pub release_date: Option<String>,
    pub end_date: Option<String>,
    pub rating: Option<f32>,
    pub trailer_url: Option<String>,
    pub characters: Vec<Character>,
    pub studio: Option<String>,
    pub staff: Vec<StaffMember>,
    pub external_ids: Value,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentWithMappings {
    pub content: Content,
    pub metadata: Vec<ContentMetadata>,
    pub tracker_mappings: Vec<TrackerMapping>,
    pub extension_sources: Vec<ExtensionSource>,
    pub relations: Vec<ContentRelation>,
    #[serde(default)]
    pub content_units: Vec<ContentUnit>,
}

impl ContentWithMappings {
    pub fn primary_metadata(&self) -> Option<&ContentMetadata> {
        self.metadata.iter().find(|m| m.source_name == "anilist")
            .or_else(|| self.metadata.first())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    Anime,
    Manga,
    Novel,
    Booru,
}

impl ContentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentType::Anime => "anime",
            ContentType::Manga => "manga",
            ContentType::Novel => "novel",
            ContentType::Booru => "booru",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentStatus {
    Planned,
    Ongoing,
    Completed,
    Cancelled,
    Hiatus,
}

impl fmt::Display for ContentStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentUnit {
    pub id: Option<i64>,
    pub cid: String,
    pub unit_number: f64,
    pub content_type: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub thumbnail_url: Option<String>,
    pub released_at: Option<String>,
    pub duration: Option<i32>,
    pub absolute_number: Option<i32>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EpisodeData {
    Count(i32),
    List(Vec<EpisodeInfo>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeInfo {
    pub number: i32,
    pub title: Option<String>,
    pub aired: Option<String>,
    pub duration: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub name: String,
    pub role: String,
    pub actor: Option<String>,
    pub image: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffMember {
    pub name: String,
    pub role: String,
    pub image: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionSource {
    pub id: Option<i64>,
    pub cid: String,
    pub extension_name: String,
    pub extension_id: String,
    pub nsfw: bool,
    pub language: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentRelation {
    pub id: Option<i64>,
    pub source_cid: String,
    pub target_cid: String,
    pub relation_type: RelationType,
    pub source_name: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationType {
    Sequel,
    Prequel,
    SideStory,
    Spinoff,
    Adaptation,
    Alternative,
    Parent,
    Summary,
}

impl RelationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RelationType::Sequel => "sequel",
            RelationType::Prequel => "prequel",
            RelationType::SideStory => "side_story",
            RelationType::Spinoff => "spinoff",
            RelationType::Adaptation => "adaptation",
            RelationType::Alternative => "alternative",
            RelationType::Parent => "parent",
            RelationType::Summary => "summary",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentTag {
    pub id: Option<i64>,
    pub cid: String,
    pub source_name: String,
    pub tag: String,
    pub tag_type: TagType,
    pub spoiler: bool,
    pub votes: i32,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TagType {
    Genre,
    Theme,
    Demographic,
    Custom,
}

use uuid::Uuid;

pub fn generate_cid() -> String {
    Uuid::new_v4().to_string()
}

pub fn generate_semantic_cid(tracker: &str, tracker_id: &str) -> String {
    format!("{}:{}", tracker, tracker_id)
}

pub struct ContentRepository;

impl ContentRepository {
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
                meta.cid,
            ],
        ).ok();

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

    pub fn upsert_metadata(conn: &Connection, meta: &ContentMetadata) -> CoreResult<()> {
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            r#"
            INSERT INTO metadata (
                cid, source_name, source_id, subtype, title, alt_titles, synopsis,
                cover_image, banner_image, eps_or_chapters, status, tags, genres,
                release_date, end_date, rating, trailer_url, characters, studio,
                staff, external_ids, created_at, updated_at
            ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23)
            ON CONFLICT(cid, source_name) DO UPDATE SET
                source_id      = excluded.source_id,
                subtype        = excluded.subtype,
                title          = excluded.title,
                alt_titles     = excluded.alt_titles,
                synopsis       = COALESCE(excluded.synopsis, metadata.synopsis),
                cover_image    = COALESCE(excluded.cover_image, metadata.cover_image),
                banner_image   = COALESCE(excluded.banner_image, metadata.banner_image),
                eps_or_chapters= excluded.eps_or_chapters,
                status         = excluded.status,
                tags           = excluded.tags,
                genres         = excluded.genres,
                release_date   = excluded.release_date,
                end_date       = excluded.end_date,
                rating         = COALESCE(excluded.rating, metadata.rating),
                trailer_url    = COALESCE(excluded.trailer_url, metadata.trailer_url),
                characters     = excluded.characters,
                studio         = excluded.studio,
                staff          = excluded.staff,
                external_ids   = excluded.external_ids,
                updated_at     = excluded.updated_at
            "#,
            params![
                meta.cid,
                meta.source_name,
                meta.source_id,
                meta.subtype,
                meta.title,
                serde_json::to_string(&meta.alt_titles).unwrap(),
                meta.synopsis,
                meta.cover_image,
                meta.banner_image,
                serde_json::to_string(&meta.eps_or_chapters).unwrap(),
                meta.status.as_ref().map(|s| serde_json::to_string(s).unwrap()),
                serde_json::to_string(&meta.tags).unwrap(),
                serde_json::to_string(&meta.genres).unwrap(),
                meta.release_date,
                meta.end_date,
                meta.rating,
                meta.trailer_url,
                serde_json::to_string(&meta.characters).unwrap(),
                meta.studio,
                serde_json::to_string(&meta.staff).unwrap(),
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
                    content_type: serde_json::from_str(&format!("\"{}\"", row.get::<_, String>(1)?)).unwrap(),
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

    pub fn get_all_metadata(conn: &Connection, cid: &str) -> CoreResult<Vec<ContentMetadata>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM metadata WHERE cid = ?1 ORDER BY CASE source_name WHEN 'anilist' THEN 0 ELSE 1 END",
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

        let (sql, param_refs_owned): (String, Vec<String>) = if let Some(year) = release_year {
            (
                "SELECT m.cid, m.title, m.alt_titles, m.release_date \
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
                "SELECT m.cid, m.title, m.alt_titles, m.release_date \
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
            ))
        })?;

        let target_normalized = normalize_title(title);
        let mut best_match: Option<String> = None;
        let mut highest_score = 0.0_f64;
        const THRESHOLD: f64 = 0.85;

        for row in rows {
            let (cid, db_title, db_alt_titles_json) = row?;

            let mut max_local_score = similarity(&target_normalized, &normalize_title(&db_title));

            if let Ok(alt_titles) = serde_json::from_str::<Vec<String>>(&db_alt_titles_json) {
                for alt in alt_titles {
                    if alt.trim().is_empty() { continue; }
                    let alt_score = similarity(&target_normalized, &normalize_title(&alt));
                    if alt_score > max_local_score { max_local_score = alt_score; }
                }
            }

            if max_local_score >= THRESHOLD && max_local_score > highest_score {
                highest_score = max_local_score;
                best_match = Some(cid);
            }
        }

        if let Some(cid) = best_match {
            return Self::get_by_cid(conn, &cid);
        }
        Ok(None)
    }

    pub fn get_full_content(conn: &Connection, cid: &str) -> CoreResult<Option<ContentWithMappings>> {
        let content = match Self::get_content_by_cid(conn, cid)? {
            Some(c) => c,
            None => return Ok(None),
        };

        let metadata = Self::get_all_metadata(conn, cid)?;
        let tracker_mappings = TrackerRepository::get_mappings_by_cid(conn, cid)?;
        let extension_sources = ExtensionRepository::get_by_cid(conn, cid)?;
        let relations = RelationRepository::get_by_source(conn, cid)?;
        let content_units = UnitRepository::get_by_cid(conn, cid)?;

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
            id: row.get(0)?,
            cid: row.get(1)?,
            source_name: row.get(2)?,
            source_id: row.get(3)?,
            subtype: row.get(4)?,
            title: row.get(5)?,
            alt_titles: serde_json::from_str(&row.get::<_, String>(6)?).unwrap_or_default(),
            synopsis: row.get(7)?,
            cover_image: row.get(8)?,
            banner_image: row.get(9)?,
            eps_or_chapters: serde_json::from_str(&row.get::<_, String>(10)?).unwrap_or(EpisodeData::Count(0)),
            status: row.get::<_, Option<String>>(11)?
                .map(|s| serde_json::from_str(&s).unwrap()),
            tags: serde_json::from_str(&row.get::<_, String>(12)?).unwrap_or_default(),
            genres: serde_json::from_str(&row.get::<_, String>(13)?).unwrap_or_default(),
            release_date: row.get(14)?,
            end_date: row.get(15)?,
            rating: row.get(16)?,
            trailer_url: row.get(17)?,
            characters: serde_json::from_str(&row.get::<_, String>(18)?).unwrap_or_default(),
            studio: row.get(19)?,
            staff: serde_json::from_str(&row.get::<_, String>(20)?).unwrap_or_default(),
            external_ids: serde_json::from_str(&row.get::<_, String>(21)?)
                .unwrap_or(serde_json::json!({})),
            created_at: row.get(22)?,
            updated_at: row.get(23)?,
        })
    }
}

pub struct ExtensionRepository;

impl ExtensionRepository {
    pub fn add_source(conn: &Connection, source: &ExtensionSource) -> CoreResult<i64> {
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            r#"
            INSERT INTO extension_sources
            (cid, extension_name, extension_id, nsfw, language, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(extension_name, extension_id) DO UPDATE SET
                nsfw       = excluded.nsfw,
                language   = excluded.language,
                updated_at = excluded.updated_at
            "#,
            params![
                source.cid, source.extension_name, source.extension_id,
                if source.nsfw { 1 } else { 0 }, source.language, now, now,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_source(conn: &Connection, id: i64, ext_id: &str) -> CoreResult<()> {
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "UPDATE extension_sources SET extension_id = ?1, updated_at = ?2 WHERE id = ?3",
            params![ext_id, now, id],
        )?;
        Ok(())
    }

    pub fn find_cid_by_extension(
        conn: &Connection,
        extension_name: &str,
        extension_id: &str,
    ) -> CoreResult<Option<String>> {
        conn.query_row(
            "SELECT cid FROM extension_sources WHERE extension_name = ?1 AND extension_id = ?2",
            params![extension_name, extension_id],
            |row| row.get(0),
        )
            .optional()
            .map_err(Into::into)
    }

    pub fn get_by_cid(conn: &Connection, cid: &str) -> CoreResult<Vec<ExtensionSource>> {
        let mut stmt = conn.prepare(
            "SELECT id, cid, extension_name, extension_id, nsfw, language, created_at, updated_at \
             FROM extension_sources WHERE cid = ?1"
        )?;
        let rows = stmt.query_map(params![cid], |row| {
            Ok(ExtensionSource {
                id: Some(row.get(0)?),
                cid: row.get(1)?,
                extension_name: row.get(2)?,
                extension_id: row.get(3)?,
                nsfw: row.get::<_, i32>(4)? == 1,
                language: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;
        let mut results = Vec::new();
        for row in rows { results.push(row?); }
        Ok(results)
    }

    pub fn find_mapping_id(conn: &Connection, cid: &str, ext_name: &str) -> CoreResult<Option<i64>> {
        conn.query_row(
            "SELECT id FROM extension_sources WHERE cid = ?1 AND extension_name = ?2",
            params![cid, ext_name],
            |row| row.get(0),
        )
            .optional()
            .map_err(Into::into)
    }

    pub fn get_extension_id_and_type(
        conn: &Connection,
        cid: &str,
        ext_name: &str,
    ) -> CoreResult<Option<(String, String)>> {
        conn.query_row(
            r#"
            SELECT c.type, es.extension_id
            FROM content c
            JOIN extension_sources es ON c.cid = es.cid
            WHERE c.cid = ?1 AND es.extension_name = ?2
            "#,
            params![cid, ext_name],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
            .optional()
            .map_err(Into::into)
    }
}

pub struct UnitRepository;

impl UnitRepository {
    pub fn upsert(conn: &Connection, unit: &ContentUnit) -> CoreResult<()> {
        conn.execute(
            r#"
            INSERT INTO content_units (
                cid, unit_number, type, title, description,
                thumbnail_url, released_at, duration, absolute_number, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            ON CONFLICT(cid, type, unit_number) DO UPDATE SET
                title         = excluded.title,
                description   = excluded.description,
                thumbnail_url = excluded.thumbnail_url,
                released_at   = excluded.released_at
            "#,
            params![
                unit.cid, unit.unit_number, unit.content_type, unit.title,
                unit.description, unit.thumbnail_url, unit.released_at,
                unit.duration, unit.absolute_number, unit.created_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_by_cid(conn: &Connection, cid: &str) -> CoreResult<Vec<ContentUnit>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM content_units WHERE cid = ?1 \
             ORDER BY CASE WHEN type = 'episode' THEN 1 ELSE 2 END, unit_number ASC",
        )?;
        let rows = stmt.query_map(params![cid], |row| {
            Ok(ContentUnit {
                id: Some(row.get(0)?),
                cid: row.get(1)?,
                unit_number: row.get(2)?,
                content_type: row.get(3)?,
                title: row.get(4)?,
                description: row.get(5)?,
                thumbnail_url: row.get(6)?,
                released_at: row.get(7)?,
                duration: row.get(8)?,
                absolute_number: row.get(9)?,
                created_at: row.get(10)?,
            })
        })?;
        let mut units = Vec::new();
        for unit in rows { units.push(unit?); }
        Ok(units)
    }
}

pub struct RelationRepository;

impl RelationRepository {
    pub fn get_by_source(conn: &Connection, source_cid: &str) -> CoreResult<Vec<ContentRelation>> {
        let mut stmt = conn.prepare(
            "SELECT id, source_cid, target_cid, relation_type, source_name, created_at \
             FROM content_relations WHERE source_cid = ?1"
        )?;
        let rows = stmt.query_map(params![source_cid], |row| {
            Ok(ContentRelation {
                id: Some(row.get(0)?),
                source_cid: row.get(1)?,
                target_cid: row.get(2)?,
                relation_type: serde_json::from_str(
                    &format!("\"{}\"", row.get::<_, String>(3)?)
                ).unwrap(),
                source_name: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;
        let mut results = Vec::new();
        for row in rows { results.push(row?); }
        Ok(results)
    }

    pub fn upsert(conn: &Connection, relation: &ContentRelation) -> CoreResult<()> {
        conn.execute(
            r#"
            INSERT INTO content_relations (source_cid, target_cid, relation_type, source_name, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(source_cid, target_cid, relation_type, source_name) DO NOTHING
            "#,
            params![
                relation.source_cid,
                relation.target_cid,
                relation.relation_type.as_str(),
                relation.source_name,
                relation.created_at,
            ],
        )?;
        Ok(())
    }
}

pub struct CacheRepository;

impl CacheRepository {
    pub fn set(
        conn: &Connection,
        key: &str,
        source: &str,
        query_type: &str,
        data: &Value,
        ttl_seconds: i64,
    ) -> CoreResult<()> {
        let now = chrono::Utc::now().timestamp();
        let expires_at = now + ttl_seconds;
        conn.execute(
            r#"
            INSERT OR REPLACE INTO cache_metadata (key, source, query_type, data, created_at, expires_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![key, source, query_type, data.to_string(), now, expires_at],
        )?;
        Ok(())
    }

    pub fn get(conn: &Connection, key: &str) -> CoreResult<Option<Value>> {
        let now = chrono::Utc::now().timestamp();
        conn.query_row(
            "SELECT data FROM cache_metadata WHERE key = ?1 AND expires_at > ?2",
            params![key, now],
            |row| {
                let data_str: String = row.get(0)?;
                Ok(serde_json::from_str(&data_str).unwrap_or(Value::Null))
            },
        )
            .optional()
            .map_err(Into::into)
    }

    pub fn cleanup(conn: &Connection) -> CoreResult<()> {
        let now = chrono::Utc::now().timestamp();
        conn.execute("DELETE FROM cache_metadata WHERE expires_at <= ?1", params![now])?;
        Ok(())
    }
}

pub struct ContentUnitRepository;

impl ContentUnitRepository {
    pub fn upsert(conn: &Connection, cid: &str, unit: &Value) -> rusqlite::Result<()> {
        let unit_type = unit.get("type").and_then(|v| v.as_str()).unwrap_or("episode");
        let unit_number = unit.get("episode").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let title = unit.get("title").and_then(|v| v.as_str());
        let description = unit.get("description").and_then(|v| v.as_str());
        let released_at = unit.get("date").and_then(|v| v.as_str());
        let thumbnail_url = unit.get("img").and_then(|v| v.as_str())
            .map(|img| format!("https://simkl.in/episodes/{}_m.jpg", img));
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT INTO content_units (\
                cid, unit_number, type, title, description, thumbnail_url, released_at, created_at\
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(cid, type, unit_number) DO UPDATE SET
                title = excluded.title,
                description = excluded.description,
                thumbnail_url = excluded.thumbnail_url,
                released_at = excluded.released_at",
            rusqlite::params![cid, unit_number, unit_type, title, description, thumbnail_url, released_at, now],
        )?;
        Ok(())
    }
}

fn normalize_title(s: &str) -> String {
    s.to_lowercase()
        .replace([':', '-', '!', '?', '.', ',', '\'', '"', '·', '~'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let v1: Vec<char> = s1.chars().collect();
    let v2: Vec<char> = s2.chars().collect();
    let len1 = v1.len();
    let len2 = v2.len();
    let mut column: Vec<usize> = (0..=len1).collect();
    for x in 1..=len2 {
        column[0] = x;
        let mut last_diag = x - 1;
        for y in 1..=len1 {
            let old_diag = column[y];
            let cost = if v1[y - 1] == v2[x - 1] { 0 } else { 1 };
            column[y] = min(column[y] + 1, min(column[y - 1] + 1, last_diag + cost));
            last_diag = old_diag;
        }
    }
    column[len1]
}

fn similarity(s1: &str, s2: &str) -> f64 {
    if s1 == s2 { return 1.0; }
    let len1 = s1.chars().count();
    let len2 = s2.chars().count();
    let max_len = std::cmp::max(len1, len2);
    if max_len == 0 { return 1.0; }
    let dist = levenshtein_distance(s1, s2);
    1.0 - (dist as f64 / max_len as f64)
}