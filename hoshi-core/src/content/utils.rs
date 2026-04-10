use serde_json::Value;
use uuid::Uuid;
use crate::state::AppState;
use crate::config::repository::ConfigRepository;
use crate::tracker::provider::TrackerMedia;

pub async fn show_adult(state: &AppState, user_id: i32) -> bool {
    ConfigRepository::get_config(&state.pool, user_id)
        .await
        .map(|c| c.general.show_adult_content)
        .unwrap_or(false)
}

pub fn filter_home_nsfw(mut view: Value) -> Value {
    if let Some(obj) = view.as_object_mut() {
        for section_key in ["anime", "manga", "novel"] {
            if let Some(section) = obj.get_mut(section_key).and_then(|s| s.as_object_mut()) {
                for list_key in ["trending", "topRated", "seasonal"] {
                    if let Some(arr) = section.get_mut(list_key).and_then(|v| v.as_array_mut()) {
                        arr.retain(|item| {
                            !item.get("content")
                                .and_then(|c| c.get("nsfw"))
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false)
                        });
                    }
                }
            }
        }
    }
    view
}

pub fn filter_array_nsfw(value: Value) -> Value {
    if let Value::Array(mut arr) = value {
        arr.retain(|item| {
            !item.get("content")
                .and_then(|c| c.get("nsfw"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
        });
        Value::Array(arr)
    } else {
        value
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