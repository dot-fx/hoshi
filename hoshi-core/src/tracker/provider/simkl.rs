use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;
use crate::content::models::{ContentType, EpisodeData, Metadata};
use crate::error::{CoreError, CoreResult};

use super::{TokenData, TrackerAuthConfig, TrackerMedia, TrackerProvider, UpdateEntryParams, UserListEntry};

const CLIENT_ID: &str = "d8385263a0cd0e60acd779d9db61310f41c8f99e40571af596ef79c7de1d4b2e";
const BASE_URL:  &str = "https://api.simkl.com";

pub struct SimklProvider {
    client: Client,
}

impl SimklProvider {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .expect("Failed to build Simkl HTTP client");
        Self { client }
    }

    async fn get_public(&self, path: &str, params: &[(&str, &str)]) -> CoreResult<Value> {
        let mut query: Vec<(&str, &str)> = vec![("client_id", CLIENT_ID)];
        query.extend_from_slice(params);

        let res = self.client.get(format!("{}{}", BASE_URL, path))
            .query(&query)
            .send()
            .await
            .map_err(|e| CoreError::Internal(format!("Simkl network error: {}", e)))?;

        if !res.status().is_success() {
            let status = res.status();
            let text   = res.text().await.unwrap_or_default();
            return Err(CoreError::Internal(format!("Simkl HTTP {}: {}", status, text)));
        }

        res.json::<Value>().await
            .map_err(|e| CoreError::Internal(format!("Simkl JSON parse: {}", e)))
    }

    async fn get_auth(&self, path: &str, token: &str, params: &[(&str, &str)]) -> CoreResult<Value> {
        let mut query: Vec<(&str, &str)> = vec![("client_id", CLIENT_ID)];
        query.extend_from_slice(params);

        let res = self.client.get(format!("{}{}", BASE_URL, path))
            .header("Authorization", format!("Bearer {}", token))
            .query(&query)
            .send()
            .await
            .map_err(|e| CoreError::Internal(format!("Simkl auth network error: {}", e)))?;

        if !res.status().is_success() {
            let status = res.status();
            let text   = res.text().await.unwrap_or_default();
            return Err(CoreError::Internal(format!("Simkl HTTP {}: {}", status, text)));
        }

        res.json::<Value>().await
            .map_err(|e| CoreError::Internal(format!("Simkl auth JSON parse: {}", e)))
    }

    async fn post_auth(&self, path: &str, token: &str, body: &Value) -> CoreResult<Value> {
        let res = self.client.post(format!("{}{}", BASE_URL, path))
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .query(&[("client_id", CLIENT_ID)])
            .json(body)
            .send()
            .await
            .map_err(|e| CoreError::Internal(format!("Simkl post error: {}", e)))?;

        if !res.status().is_success() {
            let status = res.status();
            let text   = res.text().await.unwrap_or_default();
            return Err(CoreError::Internal(format!("Simkl HTTP {}: {}", status, text)));
        }

        res.json::<Value>().await.unwrap_or(json!({}));
        Ok(json!({}))
    }

    fn item_to_tracker_media(&self, item: &Value, content_type: ContentType) -> Option<TrackerMedia> {
        let show       = item.get("show").unwrap_or(item);
        let tracker_id = show.get("ids")?.get("simkl")?.as_i64()?.to_string();

        let mut cross_ids = HashMap::new();
        if let Some(ids_obj) = show.get("ids").and_then(|v| v.as_object()) {
            for (k, v) in ids_obj {
                let val_str = if let Some(s) = v.as_str() {
                    s.to_string()
                } else if let Some(i) = v.as_i64() {
                    i.to_string()
                } else {
                    continue;
                };
                cross_ids.insert(k.clone(), val_str);
            }
        }

        let title = show.get("title").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();

        let alt_titles = show.get("all_titles")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        let poster_url = show.get("poster").and_then(|v| v.as_str())
            .map(|p| format!("https://simkl.in/posters/{}_m.jpg", p));
        let synopsis   = show.get("overview").and_then(|v| v.as_str()).map(String::from);
        let trailer_url = show.get("trailers")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|t| t.get("youtube"))
            .and_then(|v| v.as_str())
            .map(|id| format!("https://www.youtube.com/watch?v={}", id));

        Some(TrackerMedia {
            tracker_id,
            tracker_url:   None,
            cross_ids,
            content_type,
            title,
            alt_titles,
            title_i18n: Default::default(),
            synopsis,
            cover_image:   poster_url,
            banner_image:  None,
            episode_count: show.get("ep_count").and_then(|v| v.as_i64()).map(|i| i as i32)
                .or(show.get("episodes").and_then(|v| v.as_array()).map(|a| a.len() as i32)),
            chapter_count: None,
            status:        show.get("status").and_then(|v| v.as_str()).map(String::from),
            genres:        show.get("genres")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
            tags:          vec![],
            nsfw:          false,
            release_date:  show.get("year").and_then(|v| v.as_i64()).map(|y| format!("{}-01-01", y)),
            end_date:      None,
            rating:        show.get("ratings")
                .and_then(|r| r.get("simkl")).and_then(|s| s.get("rating"))
                .and_then(|v| v.as_f64()).map(|v| v as f32),
            trailer_url,
            format:        show.get("anime_type").or(show.get("type")).and_then(|v| v.as_str()).map(String::from),
            studio:        None,
            characters:    vec![],
            staff:         vec![],
            relations:     vec![],
        })
    }
    
    pub async fn find_by_cross_id(
        &self,
        id_type: &str,
        id_value: &str,
        content_type: ContentType,
    ) -> CoreResult<Option<TrackerMedia>> {
        let simkl_param = match id_type {
            "anilist" => "anilist",
            "mal"     => "mal",
            "kitsu"   => "kitsu",
            other     => other,
        };
        let res = self.get_public("/search/id", &[(simkl_param, id_value)]).await?;
        let arr = res.as_array().ok_or_else(|| CoreError::Internal("Simkl: expected array response".into()))?;
        if arr.is_empty() { return Ok(None); }
        Ok(arr.first().and_then(|item| self.item_to_tracker_media(item, content_type)))
    }

    pub async fn get_episodes(&self, simkl_id: &str) -> CoreResult<Vec<Value>> {
        let path = format!("/anime/episodes/{}", simkl_id);
        let res  = self.get_public(&path, &[]).await?;
        Ok(res.as_array().cloned().unwrap_or_default())
    }
}

#[async_trait]
impl TrackerProvider for SimklProvider {
    fn name(&self)         -> &'static str { "simkl" }
    fn display_name(&self) -> &'static str { "Simkl" }
    fn icon_url(&self)     -> &'static str { "https://eu.simkl.in/img_favicon/v2/favicon-192x192.png" }

    fn supported_types(&self) -> Vec<ContentType> {
        vec![ContentType::Anime]
    }

    fn auth_config(&self) -> TrackerAuthConfig {
        TrackerAuthConfig {
            oauth_flow: "code".into(),
            auth_url:   "https://simkl.com/pin/".into(),
            token_url:  None,
            client_id:  Some(CLIENT_ID.to_string()),
            scopes:     vec![],
        }
    }

    async fn validate_and_store_token(&self, access_token: &str, _token_type: &str) -> CoreResult<TokenData> {
        let res = self.get_auth("/users/settings", access_token, &[]).await?;

        let user_id = res.get("account")
            .and_then(|a| a.get("id"))
            .and_then(|id| id.as_i64())
            .ok_or_else(|| CoreError::AuthError("Invalid Simkl token".into()))?;

        let expires_at = Utc::now()
            .checked_add_signed(chrono::Duration::days(365))
            .ok_or_else(|| CoreError::Internal("Date overflow".into()))?
            .to_rfc3339();

        Ok(TokenData {
            access_token:    access_token.to_string(),
            refresh_token:   None,
            token_type:      "Bearer".to_string(),
            expires_at,
            tracker_user_id: user_id.to_string(),
        })
    }

    async fn search(
        &self,
        query: Option<&str>,
        content_type: ContentType,
        limit: usize,
        _sort: Option<&str>,
        _genre: Option<&str>,
        _format: Option<&str>,
        _nsfw: Option<bool>,
    ) -> CoreResult<Vec<TrackerMedia>> {
        let q = query.unwrap_or("").trim();
        if q.is_empty() { return Ok(vec![]); }

        if q.starts_with("id:") {
            let parts: Vec<&str> = q.split(':').collect();
            if parts.len() == 3 {
                if let Ok(Some(media)) = self.find_by_cross_id(parts[1], parts[2], content_type).await {
                    return Ok(vec![media]);
                }
                return Ok(vec![]);
            }
        }

        if !matches!(content_type, ContentType::Anime) {
            return Ok(vec![]);
        }

        let limit_str = limit.to_string();
        let res = self.client.get(format!("{}/search/anime", BASE_URL))
            .query(&[("client_id", CLIENT_ID), ("q", q), ("limit", &limit_str)])
            .send()
            .await
            .map_err(|e| CoreError::Internal(format!("Simkl search error: {}", e)))?
            .json::<Value>()
            .await
            .map_err(|e| CoreError::Internal(format!("Simkl search JSON: {}", e)))?;

        Ok(res.as_array()
            .map(|arr| arr.iter()
                .filter_map(|item| self.item_to_tracker_media(item, ContentType::Anime))
                .collect())
            .unwrap_or_default())
    }
    
    async fn get_by_id(&self, tracker_id: &str) -> CoreResult<Option<TrackerMedia>> {
        let res = self.get_public(
            &format!("/anime/{}", tracker_id),
            &[("extended", "full")],
        ).await?;
        Ok(self.item_to_tracker_media(&res, ContentType::Anime))
    }

    async fn get_home(&self) -> CoreResult<HashMap<String, Vec<TrackerMedia>>> {
        let mut home = HashMap::new();

        if let Ok(res) = self.get_public("/anime/trending", &[("limit", "20")]).await {
            if let Some(arr) = res.as_array() {
                home.insert(
                    "Trending Anime".to_string(),
                    arr.iter()
                        .filter_map(|i| self.item_to_tracker_media(i, ContentType::Anime))
                        .collect(),
                );
            }
        }

        if let Ok(res) = self.get_public("/anime/best/all", &[("limit", "20")]).await {
            if let Some(arr) = res.as_array() {
                home.insert(
                    "Top Rated Anime".to_string(),
                    arr.iter()
                        .filter_map(|i| self.item_to_tracker_media(i, ContentType::Anime))
                        .collect(),
                );
            }
        }

        Ok(home)
    }

    async fn get_user_list(&self, access_token: &str, _tracker_user_id: &str) -> CoreResult<Vec<UserListEntry>> {
        let res = self.get_auth("/sync/all-items/anime", access_token, &[("extended", "full")]).await?;

        let statuses = ["watching", "plantowatch", "hold", "completed", "dropped"];
        let mut entries = Vec::new();

        for status_key in &statuses {
            if let Some(items) = res.get(status_key).and_then(|v| v.get("anime")).and_then(|v| v.as_array()) {
                for item in items {
                    let show = match item.get("show") { Some(s) => s, None => continue };
                    let tracker_id = show.get("ids")
                        .and_then(|ids| ids.get("simkl"))
                        .and_then(|v| v.as_i64())
                        .map(|i| i.to_string())
                        .unwrap_or_default();

                    let title  = show.get("title").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();
                    let poster = show.get("poster").and_then(|v| v.as_str())
                        .map(|p| format!("https://simkl.in/posters/{}_m.jpg", p));

                    let normalized_status = match *status_key {
                        "watching"    => "CURRENT",
                        "plantowatch" => "PLANNING",
                        "hold"        => "PAUSED",
                        "completed"   => "COMPLETED",
                        "dropped"     => "DROPPED",
                        other         => other,
                    };

                    entries.push(UserListEntry {
                        tracker_media_id: tracker_id,
                        title,
                        poster,
                        content_type:   ContentType::Anime,
                        format:         show.get("anime_type").and_then(|v| v.as_str()).map(String::from),
                        status:         Some(normalized_status.to_string()),
                        progress:       item.get("watched_episodes_count").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                        score:          item.get("user_rating").and_then(|v| v.as_f64()),
                        start_date:     None,
                        end_date:       None,
                        repeat_count:   0,
                        notes:          None,
                        is_private:     false,
                        total_episodes: show.get("total_episodes").and_then(|v| v.as_i64()).map(|i| i as i32),
                        total_chapters: None,
                        media:          self.item_to_tracker_media(item, ContentType::Anime),
                    });
                }
            }
        }

        Ok(entries)
    }

    async fn update_entry(&self, access_token: &str, params: UpdateEntryParams) -> CoreResult<()> {
        let media_id: i64 = params.media_id.parse().unwrap_or(0);

        if let Some(ref status) = params.status {
            let simkl_status = match status.as_str() {
                "CURRENT"   => "watching",
                "PLANNING"  => "plantowatch",
                "PAUSED"    => "hold",
                "COMPLETED" => "completed",
                "DROPPED"   => "dropped",
                other       => other,
            };

            self.post_auth("/sync/add-to-list", access_token, &json!({
                "anime": [{ "ids": { "simkl": media_id }, "to": simkl_status }]
            })).await?;
        }

        if let Some(progress) = params.progress {
            self.post_auth("/sync/history", access_token, &json!({
                "anime": [{
                    "ids":              { "simkl": media_id },
                    "watched_episodes": progress
                }]
            })).await?;
        }

        if let Some(score) = params.score {
            self.post_auth("/sync/ratings", access_token, &json!({
                "anime": [{
                    "ids":    { "simkl": media_id },
                    "rating": score.round() as i32
                }]
            })).await?;
        }

        Ok(())
    }

    async fn delete_entry(&self, access_token: &str, media_id: &str) -> CoreResult<bool> {
        let id: i64 = media_id.parse().unwrap_or(0);
        let body = json!({ "anime": [{ "ids": { "simkl": id } }] });

        let res = self.client.post(format!("{}/sync/remove-from-list", BASE_URL))
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .query(&[("client_id", CLIENT_ID)])
            .json(&body)
            .send()
            .await
            .map_err(|e| CoreError::Internal(format!("Simkl delete error: {}", e)))?;

        Ok(res.status().is_success())
    }

    fn to_core_metadata(&self, cid: &str, media: &TrackerMedia) -> Metadata {
        let now = Utc::now().timestamp();
        Metadata {
            id:              None,
            cid:             cid.to_string(),
            source_name:     self.name().to_string(),
            source_id:       Some(media.tracker_id.clone()),
            subtype:         media.format.clone(),
            title:           media.title.clone(),
            alt_titles:      media.alt_titles.clone(),
            title_i18n: Default::default(),
            synopsis:        media.synopsis.clone(),
            cover_image:     media.cover_image.clone(),
            banner_image:    media.banner_image.clone(),
            eps_or_chapters: EpisodeData::Count(media.episode_count.unwrap_or(0)),
            status:          None,
            genres:          media.genres.clone(),
            release_date:    media.release_date.clone(),
            end_date:        media.end_date.clone(),
            rating:          media.rating,
            trailer_url:     media.trailer_url.clone(),
            characters:      vec![],
            studio:          media.studio.clone(),
            staff:           vec![],
            external_ids:    json!({}),
            created_at:      now,
            updated_at:      now,
        }
    }
}