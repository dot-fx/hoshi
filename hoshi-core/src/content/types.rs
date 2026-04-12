use serde::{Deserialize, Serialize};
use crate::content::models::{ContentType, FullContent};
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
    pub extension_filters: Option<String>,
    pub page: Option<u32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentListResponse {
    pub data: Vec<TrackerMedia>,
    pub total: usize,
    pub limit: i32,
    pub offset: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaSection {
    pub trending:  Vec<FullContent>,
    pub top_rated: Vec<FullContent>,
    pub seasonal:  Option<Vec<FullContent>>,
}

impl MediaSection {
    pub fn filter_nsfw(&mut self) {
        self.trending.retain(|m| !m.content.nsfw);
        self.top_rated.retain(|m| !m.content.nsfw);
        if let Some(ref mut seasonal) = self.seasonal {
            seasonal.retain(|m| !m.content.nsfw);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HomeView {
    pub anime:     MediaSection,
    pub manga:     MediaSection,
    pub novel:     MediaSection,
    pub cached_at: i64,
}

impl HomeView {
    pub fn filter_nsfw(&mut self) {
        self.anime.filter_nsfw();
        self.manga.filter_nsfw();
        self.novel.filter_nsfw();
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

pub fn parse_content_type(t: &str) -> ContentType {
    match t {
        "manga" => ContentType::Manga,
        "novel" => ContentType::Novel,
        _       => ContentType::Anime,
    }
}