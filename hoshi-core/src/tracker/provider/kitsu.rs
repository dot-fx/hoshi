use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;

use crate::content::{Character, ContentMetadata, ContentType, EpisodeData, StaffMember};
use crate::error::{CoreError, CoreResult};

use super::{
    TokenData, TrackerAuthConfig, TrackerMedia, TrackerProvider, TrackerRelation, UpdateEntryParams,
    UserListEntry,
};

// El client_id/secret públicos documentados oficialmente para apps sin registro.
// Kitsu no ha implementado aún el registro de apps, así que estos son los valores
// canónicos que usa la comunidad.
const CLIENT_ID: &str = "dd031b32d2f56c990b1425efe6c42ad847e7fe3ab46bf1299f05ecd856bdb7dd";
const CLIENT_SECRET: &str = "54d7307928f63414defd96399fc31ba847961ceaecef3a5fd93144e960c0e151";
const BASE_URL: &str = "https://kitsu.io/api/edge";
const OAUTH_URL: &str = "https://kitsu.io/api/oauth/token";

// Kitsu usa JSON:API — todos los requests necesitan estos headers.
const ACCEPT: &str = "application/vnd.api+json";
const CONTENT_TYPE: &str = "application/vnd.api+json";

pub struct KitsuProvider {
    client: Client,
}

impl KitsuProvider {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .expect("Failed to build Kitsu HTTP client");
        Self { client }
    }

    // GET sin autenticación (endpoints públicos)
    async fn get_public(&self, path: &str) -> CoreResult<Value> {
        let res = self
            .client
            .get(format!("{}{}", BASE_URL, path))
            .header("Accept", ACCEPT)
            .send()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        if res.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(CoreError::NotFound("Kitsu: resource not found".into()));
        }
        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(CoreError::Internal(format!("Kitsu HTTP {}: {}", status, text)));
        }

        res.json::<Value>()
            .await
            .map_err(|e| CoreError::Internal(format!("Kitsu JSON parse: {}", e)))
    }

    // GET autenticado
    async fn get_auth(&self, path: &str, token: &str) -> CoreResult<Value> {
        let res = self
            .client
            .get(format!("{}{}", BASE_URL, path))
            .header("Accept", ACCEPT)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(CoreError::Internal(format!("Kitsu HTTP {}: {}", status, text)));
        }

        res.json::<Value>()
            .await
            .map_err(|e| CoreError::Internal(format!("Kitsu JSON parse: {}", e)))
    }

    // POST/PATCH/DELETE autenticado con body JSON:API
    async fn mutate(
        &self,
        method: reqwest::Method,
        path: &str,
        token: &str,
        body: Option<&Value>,
    ) -> CoreResult<Value> {
        let mut req = self
            .client
            .request(method, format!("{}{}", BASE_URL, path))
            .header("Accept", ACCEPT)
            .header("Content-Type", CONTENT_TYPE)
            .header("Authorization", format!("Bearer {}", token));

        if let Some(b) = body {
            req = req.json(b);
        }

        let res = req
            .send()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(CoreError::Internal(format!("Kitsu HTTP {}: {}", status, text)));
        }

        // DELETE devuelve 204 sin body
        if res.status() == reqwest::StatusCode::NO_CONTENT {
            return Ok(json!({}));
        }

        res.json::<Value>()
            .await
            .map_err(|e| CoreError::Internal(format!("Kitsu JSON parse: {}", e)))
    }

    // Convierte el objeto `data` de JSON:API a TrackerMedia.
    // `included` permite resolver relaciones (genres, mediaRelationships, etc.)
    // cuando están presentes.
    fn media_from_data(&self, data: &Value, included: Option<&Vec<Value>>) -> Option<TrackerMedia> {
        let id = data.get("id")?.as_str()?.to_string();
        let attrs = data.get("attributes")?;
        let media_type = data.get("type")?.as_str()?;

        let content_type = match media_type {
            "anime" => ContentType::Anime,
            "manga" => {
                let subtype = attrs.get("subtype").and_then(|v| v.as_str()).unwrap_or("");
                if matches!(subtype, "novel" | "light_novel") {
                    ContentType::Novel
                } else {
                    ContentType::Manga
                }
            }
            _ => return None,
        };

        // Títulos
        let titles = attrs.get("titles");
        let canonical = attrs
            .get("canonicalTitle")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let mut alt_titles: Vec<String> = Vec::new();
        if let Some(t) = titles {
            for key in &["en", "en_jp", "en_us", "ja_jp"] {
                if let Some(s) = t.get(key).and_then(|v| v.as_str()) {
                    if s != canonical && !alt_titles.contains(&s.to_string()) {
                        alt_titles.push(s.to_string());
                    }
                }
            }
        }
        if let Some(abbrevs) = attrs.get("abbreviatedTitles").and_then(|v| v.as_array()) {
            for a in abbrevs {
                if let Some(s) = a.as_str() {
                    if !alt_titles.contains(&s.to_string()) {
                        alt_titles.push(s.to_string());
                    }
                }
            }
        }

        // Imágenes
        let cover_image = attrs
            .get("posterImage")
            .and_then(|p| p.get("large").or(p.get("original")).or(p.get("medium")))
            .and_then(|v| v.as_str())
            .map(String::from);

        let banner_image = attrs
            .get("coverImage")
            .and_then(|c| c.get("large").or(c.get("original")))
            .and_then(|v| v.as_str())
            .map(String::from);

        // Rating: averageRating es un string "0.00"–"100.00" en escala de 100.
        // Lo normalizamos a 0.0–10.0 como el resto de providers.
        let rating = attrs
            .get("averageRating")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .map(|v| (v / 10.0) as f32);

        // Trailer: Kitsu guarda el youtubeVideoId directamente
        let trailer_url = attrs
            .get("youtubeVideoId")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|id| format!("https://www.youtube.com/watch?v={}", id));

        // NSFW
        let nsfw = attrs
            .get("nsfw")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
            || attrs
            .get("ageRating")
            .and_then(|v| v.as_str())
            .map(|r| r == "R18")
            .unwrap_or(false);

        // Status
        let status = attrs
            .get("status")
            .and_then(|v| v.as_str())
            .map(String::from);

        // Conteos
        let episode_count = attrs
            .get("episodeCount")
            .and_then(|v| v.as_i64())
            .map(|i| i as i32);
        let chapter_count = attrs
            .get("chapterCount")
            .and_then(|v| v.as_i64())
            .map(|i| i as i32);

        // Géneros — vienen en `included` como objetos de tipo "categories" o "genres"
        let genres = self.extract_genres(data, included);

        // Relaciones — `mediaRelationships` en included
        let relations = self.extract_relations(data, included);

        // cross_ids: mappings en included (ej. myanimelist, anilist, thetvdb...)
        let cross_ids = self.extract_cross_ids(data, included);

        let tracker_url = Some(format!(
            "https://kitsu.io/{}/{}",
            media_type,
            attrs
                .get("slug")
                .and_then(|v| v.as_str())
                .unwrap_or(&id)
        ));

        Some(TrackerMedia {
            tracker_id: id,
            tracker_url,
            cross_ids,
            content_type,
            title: canonical,
            alt_titles,
            synopsis: attrs
                .get("synopsis")
                .or(attrs.get("description"))
                .and_then(|v| v.as_str())
                .map(String::from),
            cover_image,
            banner_image,
            episode_count,
            chapter_count,
            status,
            genres,
            tags: vec![],
            nsfw,
            release_date: attrs
                .get("startDate")
                .and_then(|v| v.as_str())
                .map(String::from),
            end_date: attrs
                .get("endDate")
                .and_then(|v| v.as_str())
                .map(String::from),
            rating,
            trailer_url,
            format: attrs
                .get("subtype")
                .and_then(|v| v.as_str())
                .map(String::from),
            studio: None, // Kitsu no expone studio directamente en anime attrs
            characters: vec![], // Requeriría include=characters — no lo pedimos en búsqueda
            staff: vec![],
            relations,
        })
    }

    // Extrae géneros del array `included` usando los IDs en relationships.genres/categories
    fn extract_genres(&self, data: &Value, included: Option<&Vec<Value>>) -> Vec<String> {
        let included = match included {
            Some(i) => i,
            None => return vec![],
        };

        // Kitsu expone géneros como "categories" en el modelo edge actual
        let rel_ids: Vec<&str> = data
            .get("relationships")
            .and_then(|r| r.get("categories").or(r.get("genres")))
            .and_then(|c| c.get("data"))
            .and_then(|d| d.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.get("id").and_then(|v| v.as_str()))
                    .collect()
            })
            .unwrap_or_default();

        included
            .iter()
            .filter(|item| {
                let t = item.get("type").and_then(|v| v.as_str()).unwrap_or("");
                let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("");
                (t == "categories" || t == "genres") && rel_ids.contains(&id)
            })
            .filter_map(|item| {
                item.get("attributes")
                    .and_then(|a| a.get("title").or(a.get("name")))
                    .and_then(|v| v.as_str())
                    .map(String::from)
            })
            .collect()
    }

    // Extrae cross_ids desde mappings en included
    fn extract_cross_ids(
        &self,
        data: &Value,
        included: Option<&Vec<Value>>,
    ) -> HashMap<String, String> {
        let mut ids = HashMap::new();
        let included = match included {
            Some(i) => i,
            None => return ids,
        };

        let mapping_ids: Vec<&str> = data
            .get("relationships")
            .and_then(|r| r.get("mappings"))
            .and_then(|m| m.get("data"))
            .and_then(|d| d.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.get("id").and_then(|v| v.as_str()))
                    .collect()
            })
            .unwrap_or_default();

        for item in included {
            let t = item.get("type").and_then(|v| v.as_str()).unwrap_or("");
            let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("");
            if t == "mappings" && mapping_ids.contains(&id) {
                if let Some(attrs) = item.get("attributes") {
                    if let (Some(site), Some(ext_id)) = (
                        attrs.get("externalSite").and_then(|v| v.as_str()),
                        attrs.get("externalId").and_then(|v| v.as_str()),
                    ) {
                        // externalSite tiene valores como "myanimelist/anime", "anilist/anime"
                        // Normalizamos a solo el nombre del tracker.
                        let site_key = site.split('/').next().unwrap_or(site).to_string();
                        ids.insert(site_key, ext_id.to_string());
                    }
                }
            }
        }

        ids
    }

    // Extrae relaciones de mediaRelationships en included
    fn extract_relations(
        &self,
        data: &Value,
        included: Option<&Vec<Value>>,
    ) -> Vec<TrackerRelation> {
        let included = match included {
            Some(i) => i,
            None => return vec![],
        };

        let rel_ids: Vec<&str> = data
            .get("relationships")
            .and_then(|r| r.get("mediaRelationships"))
            .and_then(|m| m.get("data"))
            .and_then(|d| d.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.get("id").and_then(|v| v.as_str()))
                    .collect()
            })
            .unwrap_or_default();

        let mut relations = Vec::new();

        for item in included {
            let t = item.get("type").and_then(|v| v.as_str()).unwrap_or("");
            let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("");
            if t != "mediaRelationships" || !rel_ids.contains(&id) {
                continue;
            }

            let relation_type = item
                .get("attributes")
                .and_then(|a| a.get("role"))
                .and_then(|v| v.as_str())
                .unwrap_or("related")
                .to_string();

            // El media relacionado está en relationships.destination.data
            let dest_id = item
                .get("relationships")
                .and_then(|r| r.get("destination"))
                .and_then(|d| d.get("data"))
                .and_then(|d| d.get("id"))
                .and_then(|v| v.as_str());
            let dest_type = item
                .get("relationships")
                .and_then(|r| r.get("destination"))
                .and_then(|d| d.get("data"))
                .and_then(|d| d.get("type"))
                .and_then(|v| v.as_str())
                .unwrap_or("anime");

            if let Some(dest_id) = dest_id {
                // Buscamos el objeto completo en included si está disponible
                let dest_media = included.iter().find(|inc| {
                    inc.get("id").and_then(|v| v.as_str()) == Some(dest_id)
                        && inc.get("type").and_then(|v| v.as_str()) == Some(dest_type)
                });

                let related = if let Some(dest) = dest_media {
                    self.media_from_data(dest, None)
                } else {
                    // Solo tenemos el ID, construimos un TrackerMedia mínimo
                    let c_type = if dest_type == "manga" {
                        ContentType::Manga
                    } else {
                        ContentType::Anime
                    };
                    Some(TrackerMedia {
                        tracker_id: dest_id.to_string(),
                        tracker_url: None,
                        cross_ids: HashMap::new(),
                        content_type: c_type,
                        title: String::new(),
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
                        format: None,
                        studio: None,
                        characters: vec![],
                        staff: vec![],
                        relations: vec![],
                    })
                };

                if let Some(media) = related {
                    relations.push(TrackerRelation {
                        relation_type,
                        media,
                    });
                }
            }
        }

        relations
    }

    // Convierte un libraryEntry de JSON:API a UserListEntry.
    // `included` debe contener los objetos de anime/manga relacionados.
    fn library_entry_to_user_list(
        &self,
        entry: &Value,
        included: &Vec<Value>,
    ) -> Option<UserListEntry> {
        let attrs = entry.get("attributes")?;

        // Resolver el media relacionado desde included
        let media_rel = entry
            .get("relationships")
            .and_then(|r| r.get("media"))
            .and_then(|m| m.get("data"))?;

        let media_id = media_rel.get("id")?.as_str()?;
        let media_type = media_rel.get("type")?.as_str()?;

        let media_obj = included.iter().find(|inc| {
            inc.get("id").and_then(|v| v.as_str()) == Some(media_id)
                && inc.get("type").and_then(|v| v.as_str()) == Some(media_type)
        })?;

        let media_attrs = media_obj.get("attributes")?;
        let tracker_media = self.media_from_data(media_obj, Some(included));

        let title = media_attrs
            .get("canonicalTitle")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let poster = media_attrs
            .get("posterImage")
            .and_then(|p| p.get("medium").or(p.get("small")))
            .and_then(|v| v.as_str())
            .map(String::from);

        let content_type = match media_type {
            "anime" => ContentType::Anime,
            "manga" => {
                let subtype = media_attrs
                    .get("subtype")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if matches!(subtype, "novel" | "light_novel") {
                    ContentType::Novel
                } else {
                    ContentType::Manga
                }
            }
            _ => ContentType::Anime,
        };

        // Kitsu status: "current", "planned", "completed", "on_hold", "dropped"
        let status = attrs.get("status").and_then(|v| v.as_str()).map(|s| {
            match s {
                "current" => "CURRENT",
                "planned" => "PLANNING",
                "completed" => "COMPLETED",
                "on_hold" => "PAUSED",
                "dropped" => "DROPPED",
                other => other,
            }
                .to_string()
        });

        // ratingTwenty es la escala actual (2–20). Normalizamos a 0.0–10.0.
        let score = attrs
            .get("ratingTwenty")
            .and_then(|v| v.as_i64())
            .map(|r| r as f64 / 2.0)
            .or_else(|| {
                // fallback: rating antigua en escala 0.5–5.0, normalizamos a 10
                attrs
                    .get("rating")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok())
                    .map(|r| r * 2.0)
            });

        let total_episodes = media_attrs
            .get("episodeCount")
            .and_then(|v| v.as_i64())
            .map(|i| i as i32);
        let total_chapters = media_attrs
            .get("chapterCount")
            .and_then(|v| v.as_i64())
            .map(|i| i as i32);

        Some(UserListEntry {
            tracker_media_id: media_id.to_string(),
            title,
            poster,
            content_type,
            format: media_attrs
                .get("subtype")
                .and_then(|v| v.as_str())
                .map(String::from),
            status,
            progress: attrs
                .get("progress")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
            score,
            start_date: attrs
                .get("startedAt")
                .and_then(|v| v.as_str())
                .map(|s| s[..10].to_string()), // ISO8601 → solo fecha
            end_date: attrs
                .get("finishedAt")
                .and_then(|v| v.as_str())
                .map(|s| s[..10].to_string()),
            repeat_count: attrs
                .get("reconsumeCount")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
            notes: attrs
                .get("notes")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(String::from),
            is_private: attrs
                .get("private")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            total_episodes,
            total_chapters,
            media: tracker_media,
        })
    }

    // Carga todas las páginas de libraryEntries para un usuario.
    async fn fetch_all_library(
        &self,
        token: &str,
        user_id: &str,
        kind: &str, // "anime" o "manga"
    ) -> CoreResult<Vec<UserListEntry>> {
        let mut entries = Vec::new();
        // Pedimos media incluido para no hacer N+1 requests
        let include = format!("media,media.categories,media.mappings");
        let mut offset = 0usize;
        let limit = 500; // Kitsu soporta hasta 500 en library-entries

        loop {
            let path = format!(
                "/library-entries?filter[userId]={}&filter[kind]={}&include={}&page[limit]={}&page[offset]={}",
                user_id, kind, include, limit, offset
            );

            let res = self.get_auth(&path, token).await?;

            let data = match res.get("data").and_then(|d| d.as_array()) {
                Some(d) if !d.is_empty() => d.clone(),
                _ => break,
            };

            let included = res
                .get("included")
                .and_then(|i| i.as_array())
                .cloned()
                .unwrap_or_default();

            for entry in &data {
                if let Some(ule) = self.library_entry_to_user_list(entry, &included) {
                    entries.push(ule);
                }
            }

            // Paginación: si hay menos de `limit` resultados ya llegamos al final
            if data.len() < limit {
                break;
            }
            offset += limit;
        }

        Ok(entries)
    }
}

#[async_trait]
impl TrackerProvider for KitsuProvider {
    fn name(&self) -> &'static str {
        "kitsu"
    }

    fn display_name(&self) -> &'static str {
        "Kitsu"
    }

    fn icon_url(&self) -> &'static str {
        "https://kitsu.io/favicon.ico"
    }

    fn supported_types(&self) -> Vec<ContentType> {
        vec![ContentType::Anime, ContentType::Manga, ContentType::Novel]
    }

    // Kitsu usa OAuth2 Password Grant (usuario introduce email + contraseña
    // directamente en tu app). No hay redirect/callback como en AniList o MAL.
    // El token se obtiene via POST a /api/oauth/token y se renueva con refresh_token.
    fn auth_config(&self) -> TrackerAuthConfig {
        TrackerAuthConfig {
            oauth_flow: "password".to_string(),
            auth_url: OAUTH_URL.to_string(),
            token_url: Some(OAUTH_URL.to_string()),
            client_id: Some(CLIENT_ID.to_string()),
            scopes: vec![],
        }
    }

    // El access_token ya viene resuelto (obtenido via password grant por el caller).
    // Validamos llamando a /users?filter[self]=true para obtener el user_id.
    async fn validate_and_store_token(
        &self,
        access_token: &str,
        token_type: &str,
    ) -> CoreResult<TokenData> {
        let res = self
            .get_auth("/users?filter[self]=true", access_token)
            .await?;

        let user_id = res
            .get("data")
            .and_then(|d| d.as_array())
            .and_then(|arr| arr.first())
            .and_then(|u| u.get("id"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| CoreError::AuthError("Invalid Kitsu token".into()))?
            .to_string();

        let expires_at = Utc::now()
            .checked_add_signed(chrono::Duration::days(30))
            .unwrap_or_else(Utc::now)
            .to_rfc3339();

        Ok(TokenData {
            access_token: access_token.to_string(),
            refresh_token: None, // El caller gestiona el refresh_token
            token_type: token_type.to_string(),
            expires_at,
            tracker_user_id: user_id,
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
            ContentType::Manga | ContentType::Novel => "manga",
            _ => return Ok(vec![]),
        };

        let page_limit = limit.min(20); // Kitsu máx 20 por página
        let mut path = format!(
            "/{endpoint}?page[limit]={page_limit}&include=categories,mappings,mediaRelationships"
        );

        if let Some(q) = query.filter(|q| !q.trim().is_empty()) {
            path.push_str(&format!("&filter[text]={}", urlencoding::encode(q)));
        }
        if let Some(s) = sort {
            path.push_str(&format!("&sort={}", s));
        }
        if let Some(g) = genre {
            path.push_str(&format!("&filter[categories]={}", g));
        }
        if let Some(f) = format {
            path.push_str(&format!("&filter[subtype]={}", f));
        }
        // nsfw: Kitsu filtra R18 automáticamente para requests sin token.
        // Si se pide NSFW explícitamente, no hay un param público para activarlo;
        // requeriría token de usuario con NSFW habilitado en su cuenta.

        let res = self.get_public(&path).await?;

        let data = res
            .get("data")
            .and_then(|d| d.as_array())
            .ok_or_else(|| CoreError::NotFound("Kitsu search: no data".into()))?;

        let included = res
            .get("included")
            .and_then(|i| i.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(data
            .iter()
            .filter_map(|item| self.media_from_data(item, Some(&included)))
            .collect())
    }

    // tracker_id es un número string simple (ej. "1", "12345").
    // Para saber si es anime o manga necesitamos intentar ambos o confiar en que
    // el llamador prefija con "anime:" o "manga:". Aquí aceptamos ambas formas.
    async fn get_by_id(&self, tracker_id: &str) -> CoreResult<Option<TrackerMedia>> {
        let (endpoint, id) = if let Some(stripped) = tracker_id.strip_prefix("anime:") {
            ("anime", stripped)
        } else if let Some(stripped) = tracker_id.strip_prefix("manga:") {
            ("manga", stripped)
        } else {
            // Sin prefijo: asumimos anime (igual que MAL)
            ("anime", tracker_id)
        };

        let path = format!(
            "/{}/{}?include=categories,mappings,mediaRelationships",
            endpoint, id
        );

        match self.get_public(&path).await {
            Ok(res) => {
                let included = res
                    .get("included")
                    .and_then(|i| i.as_array())
                    .cloned()
                    .unwrap_or_default();
                Ok(res
                    .get("data")
                    .and_then(|d| self.media_from_data(d, Some(&included))))
            }
            Err(CoreError::NotFound(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    async fn get_home(&self) -> CoreResult<HashMap<String, Vec<TrackerMedia>>> {
        let mut home = HashMap::new();

        // Trending anime (por popularidad descendente)
        if let Ok(res) = self
            .get_public("/anime?sort=-popularityRank&page[limit]=20&include=categories")
            .await
        {
            let included = res
                .get("included")
                .and_then(|i| i.as_array())
                .cloned()
                .unwrap_or_default();
            if let Some(data) = res.get("data").and_then(|d| d.as_array()) {
                home.insert(
                    "Trending Anime".to_string(),
                    data.iter()
                        .filter_map(|i| self.media_from_data(i, Some(&included)))
                        .collect(),
                );
            }
        }

        // Top rated anime (por rating descendente)
        if let Ok(res) = self
            .get_public("/anime?sort=-averageRating&page[limit]=20&include=categories")
            .await
        {
            let included = res
                .get("included")
                .and_then(|i| i.as_array())
                .cloned()
                .unwrap_or_default();
            if let Some(data) = res.get("data").and_then(|d| d.as_array()) {
                home.insert(
                    "Top Rated Anime".to_string(),
                    data.iter()
                        .filter_map(|i| self.media_from_data(i, Some(&included)))
                        .collect(),
                );
            }
        }

        // Trending manga
        if let Ok(res) = self
            .get_public("/manga?sort=-popularityRank&page[limit]=20&include=categories")
            .await
        {
            let included = res
                .get("included")
                .and_then(|i| i.as_array())
                .cloned()
                .unwrap_or_default();
            if let Some(data) = res.get("data").and_then(|d| d.as_array()) {
                home.insert(
                    "Trending Manga".to_string(),
                    data.iter()
                        .filter_map(|i| self.media_from_data(i, Some(&included)))
                        .collect(),
                );
            }
        }

        Ok(home)
    }

    async fn get_user_list(
        &self,
        access_token: &str,
        tracker_user_id: &str,
    ) -> CoreResult<Vec<UserListEntry>> {
        // Cargamos anime y manga en paralelo
        let (anime, manga) = tokio::try_join!(
            self.fetch_all_library(access_token, tracker_user_id, "anime"),
            self.fetch_all_library(access_token, tracker_user_id, "manga"),
        )?;

        let mut all = anime;
        all.extend(manga);
        Ok(all)
    }

    async fn update_entry(
        &self,
        access_token: &str,
        params: UpdateEntryParams,
    ) -> CoreResult<()> {
        // Primero buscamos si ya existe una libraryEntry para este media y usuario
        let user_res = self
            .get_auth("/users?filter[self]=true", access_token)
            .await?;
        let user_id = user_res
            .get("data")
            .and_then(|d| d.as_array())
            .and_then(|arr| arr.first())
            .and_then(|u| u.get("id"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| CoreError::AuthError("Could not resolve Kitsu user ID".into()))?
            .to_string();

        // Determinar el tipo de media (anime/manga) del ID
        let (media_type, raw_id) = if let Some(s) = params.media_id.strip_prefix("anime:") {
            ("anime", s.to_string())
        } else if let Some(s) = params.media_id.strip_prefix("manga:") {
            ("manga", s.to_string())
        } else {
            ("anime", params.media_id.clone())
        };

        // Buscar entrada existente
        let existing_path = format!(
            "/library-entries?filter[userId]={}&filter[mediaId]={}&filter[kind]={}",
            user_id, raw_id, media_type
        );
        let existing = self.get_auth(&existing_path, access_token).await?;
        let entry_id = existing
            .get("data")
            .and_then(|d| d.as_array())
            .and_then(|arr| arr.first())
            .and_then(|e| e.get("id"))
            .and_then(|v| v.as_str())
            .map(String::from);

        // Mapeo de status
        let kitsu_status = params.status.as_deref().map(|s| match s {
            "CURRENT" => "current",
            "PLANNING" => "planned",
            "COMPLETED" => "completed",
            "PAUSED" => "on_hold",
            "DROPPED" => "dropped",
            other => other,
        });

        // ratingTwenty: escala 2–20 (Kitsu) = score * 2
        let rating_twenty = params
            .score
            .map(|s| (s * 2.0).round().clamp(2.0, 20.0) as i64);

        let mut entry_attrs = json!({});
        if let Some(st) = kitsu_status {
            entry_attrs["status"] = json!(st);
        }
        if let Some(prog) = params.progress {
            entry_attrs["progress"] = json!(prog);
        }
        if let Some(r) = rating_twenty {
            entry_attrs["ratingTwenty"] = json!(r);
        }
        if let Some(repeat) = params.repeat_count {
            entry_attrs["reconsumeCount"] = json!(repeat);
        }
        if let Some(notes) = params.notes {
            entry_attrs["notes"] = json!(notes);
        }
        if let Some(private) = params.is_private {
            entry_attrs["private"] = json!(private);
        }

        if let Some(eid) = entry_id {
            // PATCH — actualizar entrada existente
            let body = json!({
                "data": {
                    "id": eid,
                    "type": "libraryEntries",
                    "attributes": entry_attrs
                }
            });
            self.mutate(
                reqwest::Method::PATCH,
                &format!("/library-entries/{}", eid),
                access_token,
                Some(&body),
            )
                .await?;
        } else {
            // POST — crear entrada nueva
            let body = json!({
                "data": {
                    "type": "libraryEntries",
                    "attributes": entry_attrs,
                    "relationships": {
                        "user": {
                            "data": { "id": user_id, "type": "users" }
                        },
                        "media": {
                            "data": { "id": raw_id, "type": media_type }
                        }
                    }
                }
            });
            self.mutate(
                reqwest::Method::POST,
                "/library-entries",
                access_token,
                Some(&body),
            )
                .await?;
        }

        Ok(())
    }

    async fn delete_entry(&self, access_token: &str, media_id: &str) -> CoreResult<bool> {
        // media_id puede ser el ID de la libraryEntry directamente o el mediaId.
        // Aquí asumimos que es el ID de la libraryEntry (igual que AniList).
        // Si necesitas buscar por mediaId, haría falta el mismo lookup que en update_entry.
        match self
            .mutate(
                reqwest::Method::DELETE,
                &format!("/library-entries/{}", media_id),
                access_token,
                None,
            )
            .await
        {
            Ok(_) => Ok(true),
            Err(CoreError::NotFound(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }

    fn to_core_metadata(&self, cid: &str, media: &TrackerMedia) -> ContentMetadata {
        let now = Utc::now().timestamp();

        let count = match media.content_type {
            ContentType::Anime => media.episode_count.unwrap_or(0),
            _ => media.chapter_count.unwrap_or(0),
        };

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
            status: None, // Kitsu status strings son distintos; se puede mappear si hace falta
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