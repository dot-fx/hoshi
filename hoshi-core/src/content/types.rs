use serde::{Deserialize, Serialize};
use crate::content::models::ContentType;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaSection {
    pub trending:  Vec<TrackerMedia>,
    pub top_rated: Vec<TrackerMedia>,
    pub seasonal:  Option<Vec<TrackerMedia>>,
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

impl MediaSection {
    pub fn filter_nsfw(&mut self) {
        self.trending.retain(|m| !m.nsfw);
        self.top_rated.retain(|m| !m.nsfw);

        if let Some(ref mut seasonal_list) = self.seasonal {
            seasonal_list.retain(|m| !m.nsfw);
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

pub fn parse_content_type(t: &str) -> ContentType {
    match t {
        "manga" => ContentType::Manga,
        "novel" => ContentType::Novel,
        _       => ContentType::Anime,
    }
}