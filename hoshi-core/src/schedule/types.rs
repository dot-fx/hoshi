use serde::{Deserialize, Serialize};
use crate::tracker::provider::TrackerMedia;

#[derive(Debug, Clone)]
pub struct AiringEpisode {
    pub episode:   i32,
    pub airing_at: i64,
    pub media:     Option<TrackerMedia>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiringEntryEnriched {
    pub tracker_id:  String,
    pub episode:     i32,
    pub airing_at:   i64,

    pub title:        String,
    #[serde(default)]
    pub title_i18n:   std::collections::HashMap<String, String>,
    pub subtype:      Option<String>,
    pub cover_image:  Option<String>,
    pub banner_image: Option<String>,
    pub synopsis:     Option<String>,
    pub status:       Option<String>,
    pub genres:       Vec<String>,
    pub nsfw:         bool,
    pub rating:       Option<f32>,
    pub release_date: Option<String>,
    pub end_date:     Option<String>,
    pub trailer_url:  Option<String>,
    pub studio:       Option<String>,

    #[serde(default)]
    pub user_status:   Option<String>,
    #[serde(default)]
    pub user_progress: Option<i32>,
    #[serde(default)]
    pub user_score:    Option<f64>,
}

impl AiringEntryEnriched {
    pub fn from_airing_episode(ep: AiringEpisode) -> Option<Self> {
        let m = ep.media?;
        Some(Self {
            tracker_id:   m.tracker_id.clone(),
            episode:      ep.episode,
            airing_at:    ep.airing_at,
            title:        m.title,
            title_i18n:   m.title_i18n,
            subtype:      m.format,
            cover_image:  m.cover_image,
            banner_image: m.banner_image,
            synopsis:     m.synopsis,
            status:       m.status,
            genres:       m.genres,
            nsfw:         m.nsfw,
            rating:       m.rating,
            release_date: m.release_date,
            end_date:     m.end_date,
            trailer_url:  m.trailer_url,
            studio:       m.studio,
            user_status:   None,
            user_progress: None,
            user_score:    None,
        })
    }
}

fn default_days_back()  -> i64 { 1 }
fn default_days_ahead() -> i64 { 7 }

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleWindow {
    #[serde(default = "default_days_back")]
    pub days_back:  i64,
    #[serde(default = "default_days_ahead")]
    pub days_ahead: i64,
}