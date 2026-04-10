use serde::{Deserialize, Serialize};
use crate::tracker::provider::TrackerMedia;

#[derive(Debug, Clone)]
pub struct AiringEpisode {
    pub episode:   i32,
    pub airing_at: i64,
    pub media:     Option<TrackerMedia>,
}

#[derive(sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiringEntry {
    pub id:         i64,
    pub cid:        String,
    pub episode:    i32,
    pub airing_at:  i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiringEntryEnriched {
    pub id:         i64,
    pub cid:        String,
    pub episode:    i32,
    pub airing_at:  i64,
    pub title:         String,
    pub subtype:       Option<String>,
    pub cover_image:   Option<String>,
    pub banner_image:  Option<String>,
    pub synopsis:      Option<String>,
    #[serde(default)]
    pub title_i18n: std::collections::HashMap<String, String>,
    pub status:        Option<String>,
    pub genres:        Vec<String>,
    pub nsfw:          bool,
    pub rating:        Option<f32>,
    pub release_date:  Option<String>,
    pub end_date:      Option<String>,
    pub trailer_url:   Option<String>,
    pub studio:        Option<String>,
    pub user_status:   Option<String>,
    pub user_progress: Option<i32>,
    pub user_score:    Option<f64>,
}

fn default_days_back()  -> i64 { 1 }
fn default_days_ahead() -> i64 { 7 }

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleWindow {
    #[serde(default = "default_days_back")]
    pub days_back: i64,
    #[serde(default = "default_days_ahead")]
    pub days_ahead: i64,
}