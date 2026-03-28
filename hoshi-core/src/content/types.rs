use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::content::ContentWithMappings;
use crate::tracker::provider::ContentType as TrackerContentType;
use crate::tracker::provider::TrackerMedia;


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
    pub tracker: Option<String>,
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
    pub tracker: Option<String>,
}

impl SearchQuery {
    pub fn into_params(self) -> SearchParams {
        SearchParams {
            r#type:            self.r#type,
            nsfw:              self.nsfw,
            status:            self.status,
            query:             self.query,
            limit:             self.limit.unwrap_or(20),
            offset:            self.offset.unwrap_or(0),
            extension:         self.extension,
            sort:              self.sort,
            genre:             self.genre,
            format:            self.format,
            extension_filters: self.extension_filters,
            tracker:           self.tracker,
        }
    }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaSection {
    pub trending:  Vec<ContentWithMappings>,
    pub top_rated: Vec<ContentWithMappings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seasonal:  Option<Vec<ContentWithMappings>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HomeView {
    pub anime:     MediaSection,
    pub manga:     MediaSection,
    pub novel:     MediaSection,
    pub cached_at: i64,
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
        "manga" => TrackerContentType::Manga,
        "novel" => TrackerContentType::Novel,
        _       => TrackerContentType::Anime,
    }
}

pub fn similarity_score(
    query_title: &str,
    candidate: &TrackerMedia,
    query_year: Option<i64>,
) -> f64 {
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

pub fn normalize_title_svc(s: &str) -> String {
    s.to_lowercase()
        .replace([':', '-', '!', '?', '.', ',', '\'', '"', '·', '~'], " ")
        .split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn str_similarity(s1: &str, s2: &str) -> f64 {
    let a = s1.to_lowercase();
    let b = s2.to_lowercase();
    if a == b { return 1.0; }
    let max_len = a.chars().count().max(b.chars().count());
    if max_len == 0 { return 1.0; }
    let dist = levenshtein(&a, &b);
    1.0 - (dist as f64 / max_len as f64)
}

pub fn levenshtein(s1: &str, s2: &str) -> usize {
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