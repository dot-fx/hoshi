use crate::error::{CoreError, CoreResult};
pub(crate) use super::{
    TokenData, TrackerAuthConfig, TrackerMedia, TrackerProvider, TrackerRelation, UpdateEntryParams,
    UserListEntry,
};
use crate::content::{Character, ContentMetadata, ContentType, EpisodeData, StaffMember};
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

const JIKAN_BASE_URL: &str = "https://api.jikan.moe/v4";
const MAL_API_BASE_URL: &str = "https://api.myanimelist.net/v2";

pub struct MalProvider {
    client: reqwest::Client,
    client_id: String,
}

impl MalProvider {
    pub fn new(client_id: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            client_id,
        }
    }

    /// Parsea si el ID es anime o manga (ej. "anime:123" -> ("anime", "123"))
    /// Si no tiene prefijo, asume anime por defecto.
    fn parse_media_id(id: &str) -> (&str, &str) {
        let parts: Vec<&str> = id.splitn(2, ':').collect();
        if parts.len() == 2 {
            (parts[0], parts[1])
        } else {
            ("anime", id)
        }
    }

    fn normalize_status(s: &str) -> crate::content::ContentStatus {
        use crate::content::ContentStatus;
        match s {
            "finished_airing" | "finished" => ContentStatus::Completed,
            "currently_airing" | "publishing" => ContentStatus::Ongoing,
            "not_yet_aired" | "not_yet_published" => ContentStatus::Planned,
            _ => ContentStatus::Ongoing,
        }
    }
}

#[async_trait]
impl TrackerProvider for MalProvider {
    fn name(&self) -> &'static str {
        "mal"
    }

    fn display_name(&self) -> &'static str {
        "MyAnimeList"
    }

    fn icon_url(&self) -> &'static str {
        "https://upload.wikimedia.org/wikipedia/commons/7/7a/MyAnimeList_Logo.png"
    }

    fn supported_types(&self) -> Vec<ContentType> {
        vec![ContentType::Anime, ContentType::Manga]
    }

    fn auth_config(&self) -> TrackerAuthConfig {
        TrackerAuthConfig {
            oauth_flow: "pkce".to_string(),
            auth_url: "https://myanimelist.net/v1/oauth2/authorize".to_string(),
            token_url: Some("https://myanimelist.net/v1/oauth2/token".to_string()),
            client_id: Some(self.client_id.clone()),
            scopes: vec![],
        }
    }

    async fn validate_and_store_token(
        &self,
        access_token: &str,
        token_type: &str,
    ) -> CoreResult<TokenData> {
        let url = format!("{}/users/@me", MAL_API_BASE_URL);

        let res = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        if !res.status().is_success() {
            return Err(CoreError::AuthError(
                "Token de MAL inválido o expirado".into(),
            ));
        }

        let user_data: MalUserResponse = res
            .json()
            .await
            .map_err(|e| CoreError::Parse(e.to_string()))?;

        Ok(TokenData {
            access_token: access_token.to_string(),
            // El refresh_token viene del exchange PKCE inicial; el caller debe
            // persistirlo por separado. validate_and_store_token solo recibe el
            // access_token ya resuelto, así que lo dejamos None aquí.
            refresh_token: None,
            token_type: token_type.to_string(),
            expires_at: Utc::now()
                .checked_add_signed(chrono::Duration::days(30))
                .unwrap_or_else(Utc::now)
                .to_rfc3339(),
            tracker_user_id: user_data.id.to_string(),
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
        let endpoint = match content_type {
            ContentType::Anime => "anime",
            ContentType::Manga => "manga",
            _ => return Ok(vec![]),
        };

        let mut url = format!("{}/{}?limit={}", JIKAN_BASE_URL, endpoint, limit);

        if let Some(q) = query {
            url.push_str(&format!("&q={}", q));
        }
        if let Some(s) = sort {
            url.push_str(&format!("&order_by={}&sort=desc", s));
        }
        if let Some(g) = genre {
            url.push_str(&format!("&genres={}", g));
        }
        if let Some(f) = format {
            url.push_str(&format!("&type={}", f));
        }
        if let Some(is_nsfw) = nsfw {
            if !is_nsfw {
                url.push_str("&sfw=true");
            }
        }

        let res = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        let jikan_res: JikanSearchResponse = res
            .json()
            .await
            .map_err(|e| CoreError::Parse(e.to_string()))?;

        Ok(jikan_res
            .data
            .into_iter()
            .map(|item| item.into_tracker_media(content_type.clone()))
            .collect())
    }

    // tracker_id tiene el formato "anime:123" o "manga:456".
    // parse_media_id extrae el tipo y el id numérico, igual que update_entry y
    // delete_entry, por lo que no hace falta cambiar la firma del trait.
    async fn get_by_id(&self, tracker_id: &str) -> CoreResult<Option<TrackerMedia>> {
        let (media_type, id) = Self::parse_media_id(tracker_id);
        let url = format!("{}/{}/{}/full", JIKAN_BASE_URL, media_type, id);

        let res = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        if res.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        let jikan_res: JikanSingleResponse = res
            .json()
            .await
            .map_err(|e| CoreError::Parse(e.to_string()))?;

        let c_type = if media_type == "manga" {
            ContentType::Manga
        } else {
            ContentType::Anime
        };

        Ok(Some(jikan_res.data.into_tracker_media(c_type)))
    }

    async fn get_home(&self) -> CoreResult<HashMap<String, Vec<TrackerMedia>>> {
        let mut home = HashMap::new();

        let top_url = format!("{}/top/anime?limit=10", JIKAN_BASE_URL);
        if let Ok(res) = self.client.get(&top_url).send().await {
            if let Ok(j_res) = res.json::<JikanSearchResponse>().await {
                home.insert(
                    "Top Anime".to_string(),
                    j_res
                        .data
                        .into_iter()
                        .map(|i| i.into_tracker_media(ContentType::Anime))
                        .collect(),
                );
            }
        }

        let top_manga_url = format!("{}/top/manga?limit=10", JIKAN_BASE_URL);
        if let Ok(res) = self.client.get(&top_manga_url).send().await {
            if let Ok(j_res) = res.json::<JikanSearchResponse>().await {
                home.insert(
                    "Top Manga".to_string(),
                    j_res
                        .data
                        .into_iter()
                        .map(|i| i.into_tracker_media(ContentType::Manga))
                        .collect(),
                );
            }
        }

        Ok(home)
    }

    // --- API OFICIAL MAL: LISTAS DE USUARIO ---
    // Llama tanto a animelist como a mangalist y combina los resultados.
    async fn get_user_list(
        &self,
        access_token: &str,
        _tracker_user_id: &str,
    ) -> CoreResult<Vec<UserListEntry>> {
        let anime_url = format!(
            "{}/users/@me/animelist?fields=list_status,num_episodes,mean,status&limit=1000",
            MAL_API_BASE_URL
        );
        let manga_url = format!(
            "{}/users/@me/mangalist?fields=list_status,num_chapters,num_volumes,mean,status&limit=1000",
            MAL_API_BASE_URL
        );

        let mut entries: Vec<UserListEntry> = Vec::new();

        // Anime
        let anime_res = self
            .client
            .get(&anime_url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        let anime_list: MalListResponse = anime_res
            .json()
            .await
            .map_err(|e| CoreError::Parse(e.to_string()))?;

        for node in anime_list.data {
            let media = node.node;
            let status = node.list_status;
            entries.push(UserListEntry {
                tracker_media_id: format!("anime:{}", media.id),
                title: media.title,
                poster: media.main_picture.map(|p| p.large.unwrap_or(p.medium)),
                content_type: ContentType::Anime,
                format: None,
                status: Some(status.status),
                progress: status.num_episodes_watched.unwrap_or(0),
                score: status.score.map(|s| s as f64),
                start_date: status.start_date,
                end_date: status.finish_date,
                repeat_count: status.num_times_rewatched.unwrap_or(0),
                notes: status.comments,
                is_private: false,
                total_episodes: media.num_episodes,
                total_chapters: None,
                media: None,
            });
        }

        // Manga
        let manga_res = self
            .client
            .get(&manga_url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        let manga_list: MalListResponse = manga_res
            .json()
            .await
            .map_err(|e| CoreError::Parse(e.to_string()))?;

        for node in manga_list.data {
            let media = node.node;
            let status = node.list_status;
            entries.push(UserListEntry {
                tracker_media_id: format!("manga:{}", media.id),
                title: media.title,
                poster: media.main_picture.map(|p| p.large.unwrap_or(p.medium)),
                content_type: ContentType::Manga,
                format: None,
                status: Some(status.status),
                progress: status.num_chapters_read.unwrap_or(0),
                score: status.score.map(|s| s as f64),
                start_date: status.start_date,
                end_date: status.finish_date,
                repeat_count: status.num_times_rewatched.unwrap_or(0),
                notes: status.comments,
                is_private: false,
                total_episodes: None,
                total_chapters: media.num_chapters,
                media: None,
            });
        }

        Ok(entries)
    }

    async fn update_entry(
        &self,
        access_token: &str,
        params: UpdateEntryParams,
    ) -> CoreResult<()> {
        let (media_type, id) = Self::parse_media_id(&params.media_id);
        let url = format!("{}/{}/{}/my_list_status", MAL_API_BASE_URL, media_type, id);

        // MAL usa x-www-form-urlencoded para los PATCH
        let mut form: Vec<(&str, String)> = Vec::new();

        if let Some(st) = params.status {
            form.push(("status", st));
        }
        if let Some(prog) = params.progress {
            let key = if media_type == "manga" {
                "num_chapters_read"
            } else {
                "num_watched_episodes"
            };
            form.push((key, prog.to_string()));
        }
        if let Some(score) = params.score {
            // MAL acepta enteros 0-10
            form.push(("score", (score.round() as i32).to_string()));
        }
        if let Some(repeat) = params.repeat_count {
            let key = if media_type == "manga" {
                "num_times_reread"
            } else {
                "num_times_rewatched"
            };
            form.push((key, repeat.to_string()));
        }
        if let Some(notes) = params.notes {
            form.push(("comments", notes));
        }

        let res = self
            .client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .form(&form)
            .send()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        if res.status().is_success() {
            Ok(())
        } else {
            Err(CoreError::Network(format!(
                "Error updating MAL: {}",
                res.status()
            )))
        }
    }

    async fn delete_entry(&self, access_token: &str, media_id: &str) -> CoreResult<bool> {
        let (media_type, id) = Self::parse_media_id(media_id);
        let url = format!("{}/{}/{}/my_list_status", MAL_API_BASE_URL, media_type, id);

        let res = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        Ok(res.status().is_success() || res.status() == reqwest::StatusCode::NOT_FOUND)
    }

    fn to_core_metadata(&self, cid: &str, media: &TrackerMedia) -> ContentMetadata {
        let now = Utc::now().timestamp();

        let count = match media.content_type {
            ContentType::Anime => media.episode_count.unwrap_or(0),
            _ => media.chapter_count.unwrap_or(0),
        };

        let status = media
            .status
            .as_deref()
            .map(Self::normalize_status);

        ContentMetadata {
            id: None,
            cid: cid.to_string(),
            source_name: self.name().to_string(),
            source_id: Some(media.tracker_id.clone()),
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
            release_date: media.release_date.clone(),
            end_date: media.end_date.clone(),
            rating: media.rating,
            trailer_url: media.trailer_url.clone(),
            characters: media.characters.clone(),
            studio: media.studio.clone(),
            staff: media.staff.clone(),
            external_ids: json!({}),
            created_at: now,
            updated_at: now,
        }
    }
}

// ==========================================
// STRUCTS PARA DESERIALIZACIÓN (MAL API)
// ==========================================

#[derive(Debug, Deserialize)]
struct MalUserResponse {
    id: i32,
    name: String,
}

#[derive(Debug, Deserialize)]
struct MalListResponse {
    data: Vec<MalListNodeWrapper>,
}

#[derive(Debug, Deserialize)]
struct MalListNodeWrapper {
    node: MalMediaNode,
    list_status: MalListStatus,
}

#[derive(Debug, Deserialize)]
struct MalMediaNode {
    id: i32,
    title: String,
    main_picture: Option<MalPicture>,
    num_episodes: Option<i32>,
    num_chapters: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct MalListStatus {
    status: String,
    score: Option<i32>,
    // Anime
    num_episodes_watched: Option<i32>,
    num_times_rewatched: Option<i32>,
    // Manga
    num_chapters_read: Option<i32>,
    // Compartidos
    comments: Option<String>,
    start_date: Option<String>,
    finish_date: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MalPicture {
    medium: String,
    large: Option<String>,
}

// ==========================================
// STRUCTS PARA DESERIALIZACIÓN (JIKAN)
// ==========================================

#[derive(Debug, Deserialize)]
struct JikanSearchResponse {
    data: Vec<JikanMedia>,
}

#[derive(Debug, Deserialize)]
struct JikanSingleResponse {
    data: JikanMedia,
}

#[derive(Debug, Deserialize)]
struct JikanMedia {
    mal_id: i32,
    url: String,
    images: JikanImages,
    title: String,
    title_english: Option<String>,
    title_japanese: Option<String>,
    title_synonyms: Option<Vec<String>>,
    #[serde(rename = "type")]
    media_type: Option<String>,
    episodes: Option<i32>,
    chapters: Option<i32>,
    status: Option<String>,
    score: Option<f32>,
    rating: Option<String>,
    synopsis: Option<String>,
    genres: Option<Vec<JikanEntity>>,
    explicit_genres: Option<Vec<JikanEntity>>,
    studios: Option<Vec<JikanEntity>>,
    trailer: Option<JikanTrailer>,
    aired: Option<JikanDateRange>,
    published: Option<JikanDateRange>,
    relations: Option<Vec<JikanRelation>>,
}

#[derive(Debug, Deserialize)]
struct JikanImages {
    jpg: JikanImageFormat,
}

#[derive(Debug, Deserialize)]
struct JikanImageFormat {
    large_image_url: Option<String>,
    image_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct JikanTrailer {
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct JikanDateRange {
    from: Option<String>,
    to: Option<String>,
}

#[derive(Debug, Deserialize)]
struct JikanRelation {
    relation: String,
    entry: Vec<JikanEntity>,
}

#[derive(Debug, Deserialize)]
struct JikanEntity {
    mal_id: i32,
    #[serde(rename = "type")]
    entity_type: String,
    name: String,
}

impl JikanMedia {
    fn into_tracker_media(self, content_type: ContentType) -> TrackerMedia {
        let prefix = if content_type == ContentType::Manga {
            "manga"
        } else {
            "anime"
        };

        let mut alt_titles = Vec::new();
        if let Some(t) = self.title_english {
            alt_titles.push(t);
        }
        if let Some(t) = self.title_japanese {
            alt_titles.push(t);
        }
        if let Some(synonyms) = self.title_synonyms {
            alt_titles.extend(synonyms);
        }

        let (release_date, end_date) = if let Some(aired) = self.aired {
            (aired.from, aired.to)
        } else if let Some(published) = self.published {
            (published.from, published.to)
        } else {
            (None, None)
        };

        let studio = self
            .studios
            .and_then(|mut s| if s.is_empty() { None } else { Some(s.remove(0).name) });

        let mut all_genres = Vec::new();
        if let Some(g) = self.genres {
            all_genres.extend(g.into_iter().map(|e| e.name));
        }
        if let Some(eg) = self.explicit_genres {
            all_genres.extend(eg.into_iter().map(|e| e.name));
        }

        let rating_str = self.rating.as_deref().unwrap_or("").to_lowercase();
        let is_nsfw = rating_str.contains("rx")
            || rating_str.contains("hentai")
            || all_genres.iter().any(|g| {
            let gl = g.to_lowercase();
            gl == "hentai" || gl == "erotica"
        });

        let mut relations = Vec::new();
        if let Some(jikan_rels) = self.relations {
            for rel in jikan_rels {
                for entry in rel.entry {
                    let c_type = if entry.entity_type.to_lowercase() == "manga" {
                        ContentType::Manga
                    } else {
                        ContentType::Anime
                    };
                    let rel_prefix = if c_type == ContentType::Manga {
                        "manga"
                    } else {
                        "anime"
                    };

                    let related_media = TrackerMedia {
                        tracker_id: format!("{}:{}", rel_prefix, entry.mal_id),
                        tracker_url: None,
                        cross_ids: HashMap::new(),
                        content_type: c_type,
                        title: entry.name,
                        alt_titles: vec![],
                        synopsis: None,
                        cover_image: None,
                        banner_image: None,
                        episode_count: None,
                        chapter_count: None,
                        status: None,
                        genres: vec![],
                        tags: vec![],
                        nsfw: false,
                        release_date: None,
                        end_date: None,
                        rating: None,
                        trailer_url: None,
                        format: Some(entry.entity_type),
                        studio: None,
                        characters: vec![],
                        staff: vec![],
                        relations: vec![],
                    };

                    relations.push(TrackerRelation {
                        relation_type: rel.relation.clone(),
                        media: related_media,
                    });
                }
            }
        }

        TrackerMedia {
            tracker_id: format!("{}:{}", prefix, self.mal_id),
            tracker_url: Some(self.url),
            cross_ids: HashMap::new(),
            content_type,
            title: self.title,
            alt_titles,
            synopsis: self.synopsis,
            cover_image: self.images.jpg.large_image_url.or(self.images.jpg.image_url),
            banner_image: None,
            episode_count: self.episodes,
            chapter_count: self.chapters,
            status: self.status,
            genres: all_genres,
            tags: vec![],
            nsfw: is_nsfw,
            release_date,
            end_date,
            rating: self.score,
            trailer_url: self.trailer.and_then(|t| t.url),
            format: self.media_type,
            studio,
            characters: vec![],
            staff: vec![],
            relations,
        }
    }
}