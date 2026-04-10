use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::content::models::{similarity, ContentType, FullContent};
use crate::tracker::provider::TrackerMedia;


#[derive(Debug, Clone, Deserialize)]
pub struct SearchParams {
    pub r#type: Option<String>,
    pub nsfw: Option<bool>,
    pub status: Option<String>,
    pub query: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub sort: Option<String>,
    pub genre: Option<String>,
    pub format: Option<String>,
    pub tracker: Option<String>,
    pub extension_filters: Option<String>
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentListResponse {
    pub data: Vec<TrackerMedia>,
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
pub struct PlayResponse {
    #[serde(rename = "type")]
    pub play_type: Value,
    pub data: Value,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionSearchResponse {
    pub results: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaSection {
    pub trending:  Vec<FullContent>,
    pub top_rated: Vec<FullContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seasonal:  Option<Vec<FullContent>>,
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

pub fn parse_content_type(t: &str) -> ContentType {
    match t {
        "manga" => ContentType::Manga,
        "novel" => ContentType::Novel,
        _       => ContentType::Anime,
    }
}

pub fn similarity_score(
    query_title: &str,
    candidate: &TrackerMedia,
    query_year: Option<i64>,
) -> f64 {
    let q = normalize_title_svc(query_title);
    let mut best = similarity(&q, &normalize_title_svc(&candidate.title));
    for alt in &candidate.alt_titles {
        if alt.trim().is_empty() { continue; }
        let s = similarity(&q, &normalize_title_svc(alt));
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
