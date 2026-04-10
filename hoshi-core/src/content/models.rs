use std::collections::HashMap;
use std::fmt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::tracker::types::TrackerMapping;

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
pub struct Metadata {
    pub id: Option<i64>,
    pub cid: String,
    pub source_name: String,
    pub source_id: Option<String>,
    pub subtype: Option<String>,
    pub title: String,
    pub alt_titles: Vec<String>,
    #[serde(default)]
    pub title_i18n: HashMap<String, String>,
    pub synopsis: Option<String>,
    pub cover_image: Option<String>,
    pub banner_image: Option<String>,
    pub eps_or_chapters: EpisodeData,
    pub status: Option<Status>,
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
pub struct FullContent {
    pub content: Content,
    pub metadata: Vec<Metadata>,
    pub tracker_mappings: Vec<TrackerMapping>,
    pub extension_sources: Vec<ExtensionSource>,
    pub relations: Vec<Relation>,
    #[serde(default)]
    pub content_units: Vec<ContentUnit>,
}

impl FullContent {
    pub fn primary_metadata(&self) -> Option<&Metadata> {
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
}

impl ContentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentType::Anime => "anime",
            ContentType::Manga => "manga",
            ContentType::Novel => "novel",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Planned,
    Ongoing,
    Completed,
    Cancelled,
    Hiatus,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EpisodeData {
    Count(i32),
    List(Vec<EpisodeInfo>),
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
            RelationType::Sequel      => "sequel",
            RelationType::Prequel     => "prequel",
            RelationType::SideStory   => "side_story",
            RelationType::Spinoff     => "spinoff",
            RelationType::Adaptation  => "adaptation",
            RelationType::Alternative => "alternative",
            RelationType::Parent      => "parent",
            RelationType::Summary     => "summary",
        }
    }
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
pub struct Relation {
    pub id: Option<i64>,
    pub source_cid: String,
    pub target_cid: String,
    pub relation_type: RelationType,
    pub source_name: String,
    pub created_at: i64,
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

pub fn generate_cid() -> String {
    Uuid::new_v4().to_string()
}

pub fn generate_semantic_cid(tracker: &str, tracker_id: &str) -> String {
    format!("{}:{}", tracker, tracker_id)
}

pub fn normalize_title(s: &str) -> String {
    s.to_lowercase()
        .replace([':', '-', '!', '?', '.', ',', '\'', '"', '·', '~'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn levenshtein_distance(s1: &str, s2: &str) -> usize {
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
            column[y] = std::cmp::min(column[y] + 1, std::cmp::min(column[y - 1] + 1, last_diag + cost));
            last_diag = old_diag;
        }
    }
    column[len1]
}

pub fn similarity(s1: &str, s2: &str) -> f64 {
    if s1 == s2 { return 1.0; }
    let max_len = std::cmp::max(s1.chars().count(), s2.chars().count());
    if max_len == 0 { return 1.0; }
    let dist = levenshtein_distance(s1, s2);
    1.0 - (dist as f64 / max_len as f64)
}