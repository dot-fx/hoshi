use async_trait::async_trait;
use chrono::{Duration, Utc};
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration as StdDuration;

use crate::content::repository::{
    Character, ContentStatus, ContentType, CoreMetadata, StaffMember,
};
use crate::error::{CoreError, CoreResult};
use crate::schedule::repository::AiringEpisode;

pub(crate) use super::{TokenData, TrackerMedia, TrackerProvider, UpdateEntryParams, UserListEntry};

const MEDIA_FRAGMENT: &str = r#"
fragment mediaFields on Media {
  id idMal type format
  title { romaji english native userPreferred }
  synonyms
  description bannerImage
  coverImage { extraLarge large }
  episodes chapters
  status
  startDate { year month day }
  endDate   { year month day }
  genres
  tags { name isMediaSpoiler isAdult }
  isAdult
  averageScore meanScore
  trailer { id site }
  relations {
    edges {
      relationType(version: 2)
      node {
        id
        idMal
        title { romaji english native userPreferred }
        type
        status
        format
        coverImage { large }
      }
    }
  }
  characters(role: MAIN, perPage: 6) {
    edges {
      role
      node {
        id
        name { full }
        image { large }
      }
      voiceActors(language: JAPANESE, sort: [RELEVANCE, ID]) {
        id
        name { full }
        image { large }
      }
    }
  }
  staff(perPage: 8) {
    edges {
      role
      node {
        id
        name { full }
        image { large }
      }
    }
  }
  studios(isMain: true) { nodes { name } }
}
"#;

const SEARCH_QUERY: &str = r#"
query ($search: String, $page: Int, $perPage: Int, $type: MediaType,
       $sort: [MediaSort], $status: MediaStatus, $genre: String, $format: MediaFormat, $isAdult: Boolean) {
  Page(page: $page, perPage: $perPage) {
    media(search: $search, type: $type, sort: $sort, status: $status,
          genre: $genre, format: $format, isAdult: $isAdult) {
      ...mediaFields
    }
  }
}
"#;

const HOME_QUERY: &str = r#"
query {
  trending_anime: Page(perPage: 10) {
    media(sort: TRENDING_DESC, type: ANIME) { ...mediaFields }
  }
  trending_manga: Page(perPage: 10) {
    media(sort: TRENDING_DESC, type: MANGA) { ...mediaFields }
  }
  top_rated: Page(perPage: 10) {
    media(sort: SCORE_DESC, type: ANIME) { ...mediaFields }
  }
  seasonal: Page(perPage: 10) {
    media(sort: POPULARITY_DESC, status: RELEASING, type: ANIME) { ...mediaFields }
  }
}
"#;

const USER_LIST_QUERY: &str = r#"
query ($userId: Int) {
  anime: MediaListCollection(userId: $userId, type: ANIME) {
    lists {
      entries {
        media {
          ...mediaFields
          nextAiringEpisode { episode } # Campo exclusivo que tenías aquí
        }
        status progress score repeat notes private
        startedAt   { year month day }
        completedAt { year month day }
      }
    }
  }
  manga: MediaListCollection(userId: $userId, type: MANGA) {
    lists {
      entries {
        media {
          ...mediaFields
        }
        status progress score repeat notes private
        startedAt   { year month day }
        completedAt { year month day }
      }
    }
  }
}
"#;

const AIRING_SCHEDULE_QUERY: &str = r#"
query ($mediaId: Int, $page: Int) {
  Page(page: $page, perPage: 50) {
    airingSchedules(mediaId: $mediaId) {
      episode
      airingAt
      media {
        ...mediaFields
      }
    }
  }
}
"#;

pub struct AniListProvider {
    client: Client,
}

impl AniListProvider {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(StdDuration::from_secs(15))
            .build()
            .expect("Failed to build AniList HTTP client");
        Self { client }
    }

    const BASE_URL: &'static str = "https://graphql.anilist.co";
    const MAX_RETRIES: u32 = 3;

    async fn graphql(&self, token: Option<&str>, body: &Value) -> CoreResult<Value> {
        let mut req = self.client.post(Self::BASE_URL)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json");

        if let Some(t) = token {
            req = req.header("Authorization", format!("Bearer {}", t));
        }

        let req = req.json(body);
        let res = Self::with_retry(req).await
            .map_err(|e| CoreError::Internal(format!("AniList network error: {}", e)))?;

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(CoreError::Internal(format!("AniList HTTP {}: {}", status, text)));
        }

        let json: Value = res.json().await
            .map_err(|e| CoreError::Internal(format!("AniList JSON parse: {}", e)))?;

        if let Some(errors) = json.get("errors").and_then(|e| e.as_array()) {
            if !errors.is_empty() {
                // 404 es "no encontrado", no es un error fatal
                if errors[0].get("status").and_then(|s| s.as_i64()) == Some(404) {
                    return Ok(json);
                }
                let msg = errors[0].get("message").and_then(|s| s.as_str()).unwrap_or("GraphQL error");
                return Err(CoreError::Internal(format!("AniList GraphQL: {}", msg)));
            }
        }

        Ok(json)
    }

    async fn with_retry(request_builder: reqwest::RequestBuilder) -> Result<reqwest::Response, reqwest::Error> {
        let mut delay = StdDuration::from_secs(1);
        let mut last_err = None;

        for attempt in 0..=Self::MAX_RETRIES {
            match request_builder.try_clone().unwrap().send().await {
                Ok(res) => {
                    if res.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                        let wait = res.headers()
                            .get("Retry-After")
                            .and_then(|h| h.to_str().ok())
                            .and_then(|s| s.parse::<u64>().ok())
                            .map(StdDuration::from_secs)
                            .unwrap_or(delay);

                        if attempt < Self::MAX_RETRIES {
                            tracing::warn!("AniList rate limit. Waiting {}s…", wait.as_secs());
                            tokio::time::sleep(wait).await;
                            delay = delay.min(StdDuration::from_secs(60)) * 2;
                            continue;
                        }
                    }
                    if res.status().is_server_error() && attempt < Self::MAX_RETRIES {
                        tokio::time::sleep(delay).await;
                        delay = delay.min(StdDuration::from_secs(30)) * 2;
                        continue;
                    }
                    return Ok(res);
                }
                Err(e) => {
                    last_err = Some(e);
                    if attempt < Self::MAX_RETRIES {
                        tokio::time::sleep(delay).await;
                        delay = delay.min(StdDuration::from_secs(30)) * 2;
                    }
                }
            }
        }
        Err(last_err.unwrap())
    }

    fn parse_date(obj: &Value) -> Option<String> {
        let y = obj.get("year").and_then(|v| v.as_i64())?;
        let m = obj.get("month").and_then(|v| v.as_i64()).unwrap_or(1);
        let d = obj.get("day").and_then(|v| v.as_i64()).unwrap_or(1);
        Some(format!("{:04}-{:02}-{:02}", y, m, d))
    }

    fn to_fuzzy_date(date_str: Option<&str>) -> Value {
        if let Some(s) = date_str {
            let parts: Vec<&str> = s.split('-').collect();
            if parts.len() == 3 {
                return json!({
                    "year":  parts[0].parse::<i32>().unwrap_or(0),
                    "month": parts[1].parse::<i32>().unwrap_or(0),
                    "day":   parts[2].parse::<i32>().unwrap_or(0),
                });
            }
        }
        json!(null)
    }

    fn media_to_tracker_media(&self, data: &Value) -> Option<TrackerMedia> {
        let tracker_id = data.get("id")?.as_i64()?.to_string();

        let mut cross_ids = HashMap::new();
        if let Some(mal_id) = data.get("idMal").and_then(|v| v.as_i64()) {
            cross_ids.insert("myanimelist".to_string(), mal_id.to_string());
        }

        let format_str = data.get("format").and_then(|v| v.as_str());

        let content_type = match data.get("type").and_then(|v| v.as_str()) {
            Some("MANGA") if matches!(format_str, Some("NOVEL") | Some("LIGHT_NOVEL")) => ContentType::Novel,
            Some("MANGA") => ContentType::Manga,
            _ => ContentType::Anime,
        };

        let title = data.get("title")
            .and_then(|t| t.get("romaji").or(t.get("english")))
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let mut alt_titles = vec![];
        if let Some(t) = data.get("title").and_then(|t| t.get("english")).and_then(|v| v.as_str()) {
            alt_titles.push(t.to_string());
        }
        if let Some(t) = data.get("title").and_then(|t| t.get("native")).and_then(|v| v.as_str()) {
            alt_titles.push(t.to_string());
        }
        if let Some(syns) = data.get("synonyms").and_then(|v| v.as_array()) {
            alt_titles.extend(syns.iter().filter_map(|v| v.as_str().map(String::from)));
        }

        let cover_image = data.get("coverImage")
            .and_then(|i| i.get("extraLarge").or(i.get("large")))
            .and_then(|v| v.as_str())
            .map(String::from);

        let genres: Vec<String> = data.get("genres").and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        let tags: Vec<String> = data.get("tags").and_then(|v| v.as_array())
            .map(|arr| arr.iter()
                .filter(|t| t.get("isAdult").and_then(|v| v.as_bool()).unwrap_or(false) == false)
                .filter_map(|t| t.get("name").and_then(|v| v.as_str()).map(String::from))
                .collect())
            .unwrap_or_default();

        let trailer_url = data.get("trailer").and_then(|t| {
            let site = t.get("site").and_then(|s| s.as_str())?;
            let id = t.get("id").and_then(|s| s.as_str())?;
            match site {
                "youtube" => Some(format!("https://www.youtube.com/watch?v={}", id)),
                "dailymotion" => Some(format!("https://www.dailymotion.com/video/{}", id)),
                _ => None,
            }
        });

        let rating = data.get("averageScore").or(data.get("meanScore"))
            .and_then(|v| v.as_f64())
            .map(|v| (v / 10.0) as f32);

        let studio = data.get("studios")
            .and_then(|s| s.get("nodes"))
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|n| n.get("name").and_then(|v| v.as_str()).map(String::from));

        let mut characters = Vec::new();
        if let Some(edges) = data.get("characters").and_then(|c| c.get("edges")).and_then(|e| e.as_array()) {
            for edge in edges {
                let role = edge.get("role").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let node = edge.get("node");
                let name = node.and_then(|n| n.get("name")).and_then(|n| n.get("full")).and_then(|v| v.as_str()).unwrap_or("").to_string();
                let image = node.and_then(|n| n.get("image")).and_then(|i| i.get("large")).and_then(|v| v.as_str()).map(String::from);
                let actor = edge.get("voiceActors").and_then(|v| v.as_array()).and_then(|arr| arr.first())
                    .and_then(|va| va.get("name")).and_then(|n| n.get("full")).and_then(|v| v.as_str()).map(String::from);

                characters.push(Character { name, role, actor, image });
            }
        }

        let mut staff = Vec::new();
        if let Some(edges) = data.get("staff").and_then(|s| s.get("edges")).and_then(|e| e.as_array()) {
            for edge in edges {
                let role = edge.get("role").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let node = edge.get("node");
                let name = node.and_then(|n| n.get("name")).and_then(|n| n.get("full")).and_then(|v| v.as_str()).unwrap_or("").to_string();
                let image = node.and_then(|n| n.get("image")).and_then(|i| i.get("large")).and_then(|v| v.as_str()).map(String::from);
                staff.push(StaffMember { name, role, image });
            }
        }

        let mut relations = Vec::new();
        if let Some(edges) = data.get("relations").and_then(|r| r.get("edges")).and_then(|e| e.as_array()) {
            for edge in edges {
                let relation_type = edge.get("relationType").and_then(|v| v.as_str()).unwrap_or("").to_string();
                if let Some(node) = edge.get("node") {
                    if let Some(rel_media) = self.media_to_tracker_media(node) {
                        relations.push(super::TrackerRelation {
                            relation_type,
                            media: rel_media,
                        });
                    }
                }
            }
        }

        Some(TrackerMedia {
            tracker_id,
            tracker_url: None,
            cross_ids,
            content_type,
            title,
            alt_titles,
            synopsis: data.get("description").and_then(|v| v.as_str()).map(String::from),
            cover_image,
            banner_image: data.get("bannerImage").and_then(|v| v.as_str()).map(String::from),
            episode_count: data.get("episodes").and_then(|v| v.as_i64()).map(|i| i as i32),
            chapter_count: data.get("chapters").and_then(|v| v.as_i64()).map(|i| i as i32),
            status: data.get("status").and_then(|v| v.as_str()).map(String::from),
            genres,
            tags,
            nsfw: data.get("isAdult").and_then(|v| v.as_bool()).unwrap_or(false),
            release_date: data.get("startDate").and_then(Self::parse_date),
            end_date: data.get("endDate").and_then(Self::parse_date),
            rating,
            trailer_url,
            format: format_str.map(String::from),
            studio,
            characters,
            staff,
            relations,
        })
    }

    fn normalize_status(s: &str) -> ContentStatus {
        match s {
            "FINISHED" => ContentStatus::Completed,
            "RELEASING" => ContentStatus::Ongoing,
            "NOT_YET_RELEASED" => ContentStatus::Planned,
            "CANCELLED" => ContentStatus::Cancelled,
            "HIATUS" => ContentStatus::Hiatus,
            _ => ContentStatus::Ongoing,
        }
    }
}

#[async_trait]
impl TrackerProvider for AniListProvider {
    fn name(&self) -> &'static str { "anilist" }
    fn display_name(&self) -> &'static str { "AniList" }
    fn icon_url(&self) -> &'static str {
        "https://anilist.co/img/icons/android-chrome-512x512.png"
    }
    fn auth_config(&self) -> super::TrackerAuthConfig {
        super::TrackerAuthConfig {
            oauth_flow: "implicit".into(),
            auth_url: "https://anilist.co/api/v2/oauth/authorize".into(),
            client_id: std::env::var("ANILIST_CLIENT_ID").ok(),
            scopes: vec![],
        }
    }
    fn supported_types(&self) -> Vec<ContentType> {
        vec![ContentType::Anime, ContentType::Manga, ContentType::Novel]
    }

    async fn validate_and_store_token(&self, access_token: &str, token_type: &str) -> CoreResult<TokenData> {
        let res = self.graphql(Some(access_token), &json!({ "query": "query { Viewer { id } }" })).await?;

        let user_id = res.get("data")
            .and_then(|d| d.get("Viewer"))
            .and_then(|v| v.get("id"))
            .and_then(|id| id.as_i64())
            .ok_or_else(|| CoreError::AuthError("Invalid AniList token".into()))?;

        let expires_at = Utc::now()
            .checked_add_signed(Duration::days(365))
            .ok_or_else(|| CoreError::Internal("Date overflow".into()))?
            .to_rfc3339();

        Ok(TokenData {
            access_token: access_token.to_string(),
            refresh_token: None,
            token_type: token_type.to_string(),
            expires_at,
            tracker_user_id: user_id.to_string(),
        })
    }

    async fn search(
        &self,
        query: Option<&str>,
        content_type: ContentType,
        limit: usize,
        sort: Option<&str>,
        genre: Option<&str>,
        format: Option<&str>,
        nsfw: Option<bool>,
    ) -> CoreResult<Vec<TrackerMedia>> {
        let al_type = match content_type {
            ContentType::Manga | ContentType::Novel => "MANGA",
            _ => "ANIME",
        };

        let mut variables = json!({
            "page": 1,
            "perPage": limit.min(50),
            "type": al_type,
            "isAdult": nsfw.unwrap_or(false)
        });

        if let Some(q) = query.filter(|q| !q.trim().is_empty()) {
            variables["search"] = json!(q);
        }
        if let Some(s) = sort { variables["sort"] = json!([s]); }
        if let Some(g) = genre { variables["genre"] = json!(g); }
        if let Some(f) = format { variables["format"] = json!(f); }

        let full_query = format!("{}\n{}", SEARCH_QUERY, MEDIA_FRAGMENT);
        let res = self.graphql(None, &json!({ "query": full_query, "variables": variables })).await?;

        let media_list = res.get("data")
            .and_then(|d| d.get("Page"))
            .and_then(|p| p.get("media"))
            .and_then(|v| v.as_array())
            .ok_or_else(|| CoreError::NotFound("No results from AniList".into()))?;

        Ok(media_list.iter().filter_map(|m| self.media_to_tracker_media(m)).collect())
    }

    async fn get_by_id(&self, tracker_id: &str) -> CoreResult<Option<TrackerMedia>> {
        let query = r#"query ($id: Int) { Media(id: $id) { ...mediaFields } } "#;
        let full_query = format!("{}\n{}", query, MEDIA_FRAGMENT);

        let res = self.graphql(None, &json!({
            "query": full_query,
            "variables": { "id": tracker_id.parse::<i64>().unwrap_or(0) }
        })).await?;

        let media = res.get("data").and_then(|d| d.get("Media"));
        Ok(media.and_then(|m| self.media_to_tracker_media(m)))
    }

    async fn get_home(&self) -> CoreResult<HashMap<String, Vec<TrackerMedia>>> {
        let full_query = format!("{}\n{}", HOME_QUERY, MEDIA_FRAGMENT);
        let res = self.graphql(None, &json!({ "query": full_query })).await?;
        let data = res.get("data").ok_or_else(|| CoreError::NotFound("AniList home: no data".into()))?;

        let mut sections = HashMap::new();
        for section_key in &["trending_anime", "trending_manga", "top_rated", "seasonal"] {
            let items: Vec<TrackerMedia> = data.get(section_key)
                .and_then(|p| p.get("media"))
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|m| self.media_to_tracker_media(m)).collect())
                .unwrap_or_default();
            sections.insert(section_key.to_string(), items);
        }

        Ok(sections)
    }

    async fn get_user_list(&self, access_token: &str, tracker_user_id: &str) -> CoreResult<Vec<UserListEntry>> {
        let user_id: i64 = tracker_user_id.parse().unwrap_or(0);
        let res = self.graphql(
            Some(access_token),
            &json!({ "query": USER_LIST_QUERY, "variables": { "userId": user_id } }),
        ).await?;

        let data = res.get("data").ok_or_else(|| CoreError::Internal("AniList list: missing data".into()))?;
        let mut results = Vec::new();

        for (media_type, al_type) in &[("anime", "ANIME"), ("manga", "MANGA")] {
            if let Some(lists) = data.get(media_type).and_then(|a| a.get("lists")).and_then(|l| l.as_array()) {
                for list in lists {
                    if let Some(entries) = list.get("entries").and_then(|e| e.as_array()) {
                        for entry in entries {
                            let media = entry.get("media");

                            let tracker_media_id = media
                                .and_then(|m| m.get("id"))
                                .and_then(|i| i.as_i64())
                                .map(|i| i.to_string())
                                .unwrap_or_default();

                            let format = media.and_then(|m| m.get("format")).and_then(|v| v.as_str());

                            let resolved_type = if *al_type == "MANGA"
                                && matches!(format, Some("LIGHT_NOVEL") | Some("NOVEL"))
                            {
                                ContentType::Novel
                            } else if *al_type == "MANGA" {
                                ContentType::Manga
                            } else {
                                ContentType::Anime
                            };

                            let episodes = media.and_then(|m| m.get("episodes")).and_then(|i| i.as_i64());
                            let next_ep = media.and_then(|m| m.get("nextAiringEpisode"))
                                .and_then(|n| n.get("episode")).and_then(|i| i.as_i64());
                            let total_episodes = episodes.or_else(|| next_ep.map(|e| e - 1)).map(|i| i as i32);
                            let total_chapters = media.and_then(|m| m.get("chapters")).and_then(|i| i.as_i64()).map(|i| i as i32);

                            let title = media.and_then(|m| m.get("title"))
                                .and_then(|t| t.get("userPreferred").or(t.get("english")).or(t.get("romaji")))
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unknown").to_string();

                            let poster = media.and_then(|m| m.get("coverImage"))
                                .and_then(|c| c.get("extraLarge").or(c.get("large")))
                                .and_then(|v| v.as_str())
                                .map(String::from);

                            let tracker_media = media.and_then(|m| self.media_to_tracker_media(m));

                            results.push(UserListEntry {
                                tracker_media_id,
                                title,
                                poster,
                                content_type: resolved_type,
                                format: format.map(String::from),
                                status: entry.get("status").and_then(|s| s.as_str()).map(String::from),
                                progress: entry.get("progress").and_then(|i| i.as_i64()).unwrap_or(0) as i32,
                                score: entry.get("score").and_then(|f| f.as_f64()),
                                start_date: entry.get("startedAt").and_then(|d| Self::parse_date(d)),
                                end_date: entry.get("completedAt").and_then(|d| Self::parse_date(d)),
                                repeat_count: entry.get("repeat").and_then(|i| i.as_i64()).unwrap_or(0) as i32,
                                notes: entry.get("notes").and_then(|s| s.as_str()).map(String::from),
                                is_private: entry.get("private").and_then(|b| b.as_bool()).unwrap_or(false),
                                total_episodes,
                                total_chapters,
                                media: tracker_media,
                            });
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    async fn update_entry(&self, access_token: &str, params: UpdateEntryParams) -> CoreResult<()> {
        let mutation = r#"
            mutation ($mediaId: Int, $status: MediaListStatus, $progress: Int, $score: Float,
                      $startedAt: FuzzyDateInput, $completedAt: FuzzyDateInput,
                      $repeat: Int, $notes: String, $private: Boolean) {
                SaveMediaListEntry(
                    mediaId: $mediaId, status: $status, progress: $progress, score: $score,
                    startedAt: $startedAt, completedAt: $completedAt,
                    repeat: $repeat, notes: $notes, private: $private
                ) { id }
            }
        "#;

        let media_id: i64 = params.media_id.parse().unwrap_or(0);
        let variables = json!({
            "mediaId":     media_id,
            "status":      params.status,
            "progress":    params.progress,
            "score":       params.score,
            "startedAt":   Self::to_fuzzy_date(params.start_date.as_deref()),
            "completedAt": Self::to_fuzzy_date(params.end_date.as_deref()),
            "repeat":      params.repeat_count,
            "notes":       params.notes,
            "private":     params.is_private,
        });

        self.graphql(Some(access_token), &json!({ "query": mutation, "variables": variables })).await?;
        Ok(())
    }

    async fn delete_entry(&self, access_token: &str, media_id: &str) -> CoreResult<bool> {
        let viewer = self.graphql(Some(access_token), &json!({ "query": "query { Viewer { id } }" })).await?;
        let user_id = viewer.get("data").and_then(|d| d.get("Viewer")).and_then(|v| v.get("id"))
            .ok_or_else(|| CoreError::Internal("Failed to get AniList viewer ID".into()))?;

        let mid: i64 = media_id.parse().unwrap_or(0);

        let find = self.graphql(Some(access_token), &json!({
            "query": "query ($mediaId: Int, $userId: Int) { MediaList(mediaId: $mediaId, userId: $userId) { id } }",
            "variables": { "mediaId": mid, "userId": user_id }
        })).await?;

        let list_id = find.get("data").and_then(|d| d.get("MediaList")).and_then(|l| l.get("id"))
            .ok_or_else(|| CoreError::NotFound("Entry not found in AniList".into()))?;

        let del = self.graphql(Some(access_token), &json!({
            "query": "mutation ($id: Int) { DeleteMediaListEntry(id: $id) { deleted } }",
            "variables": { "id": list_id }
        })).await?;

        Ok(del.get("data")
            .and_then(|d| d.get("DeleteMediaListEntry"))
            .and_then(|x| x.get("deleted"))
            .and_then(|b| b.as_bool())
            .unwrap_or(false))
    }

    fn to_core_metadata(&self, cid: &str, media: &TrackerMedia) -> CoreMetadata {
        use crate::content::repository::EpisodeData;
        let now = Utc::now().timestamp();

        let count = match media.content_type {
            ContentType::Anime => media.episode_count.unwrap_or(0),
            _ => media.chapter_count.unwrap_or(0),
        };

        let status = media.status.as_deref().map(Self::normalize_status);

        CoreMetadata {
            cid: cid.to_string(),
            content_type: media.content_type.clone(),
            subtype: media.format.clone(),
            title: media.title.clone(),
            alt_titles: media.alt_titles.clone(),
            synopsis: media.synopsis.clone(),
            cover_image: media.cover_image.clone(),
            banner_image: media.banner_image.clone(),
            eps_or_chapters: EpisodeData::Count(count),
            status,
            tags: media.tags.clone(),
            genres: media.genres.clone(),
            nsfw: media.nsfw,
            release_date: media.release_date.clone(),
            end_date: media.end_date.clone(),
            rating: media.rating,
            trailer_url: media.trailer_url.clone(),
            characters: media.characters.clone(),
            studio: media.studio.clone(),
            staff: media.staff.clone(),
            sources: Some(self.name().to_string()),
            external_ids: json!({}),
            created_at: now,
            updated_at: now,
        }
    }
}

pub async fn fetch_airing_schedule(
    provider: &AniListProvider,
    anilist_id: i64,
) -> CoreResult<Vec<AiringEpisode>> {
    let full_query = format!("{}\n{}", AIRING_SCHEDULE_QUERY, MEDIA_FRAGMENT);

    let mut all_episodes: Vec<AiringEpisode> = Vec::new();
    let mut page = 1i32;

    loop {
        let res = provider.graphql(None, &json!({
            "query": full_query,
            "variables": {
                "mediaId": anilist_id,
                "page":    page,
            }
        })).await?;

        let schedules = res
            .get("data")
            .and_then(|d| d.get("Page"))
            .and_then(|p| p.get("airingSchedules"))
            .and_then(|v| v.as_array());

        let schedules = match schedules {
            Some(s) if !s.is_empty() => s.clone(),
            _ => break,
        };

        for entry in &schedules {
            let episode = match entry.get("episode").and_then(|v| v.as_i64()) {
                Some(e) => e as i32,
                None => continue,
            };
            let airing_at = match entry.get("airingAt").and_then(|v| v.as_i64()) {
                Some(t) => t,
                None => continue,
            };
            // `media` puede ser null para episodios muy futuros; lo convertimos si está presente
            let media = entry
                .get("media")
                .and_then(|m| provider.media_to_tracker_media(m));

            all_episodes.push(AiringEpisode {
                episode,
                airing_at,
                media,
            });
        }

        // AniList devuelve hasta 50 por página; si no llenó la página, no hay más
        if schedules.len() < 50 {
            break;
        }
        page += 1;
    }

    Ok(all_episodes)
}