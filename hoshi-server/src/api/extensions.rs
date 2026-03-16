use crate::error::AppResult;
use axum::{
    extract::{Path, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

use hoshi_core::{
    extensions::{Extension, ExtensionType},
    state::AppState,
};

#[derive(Serialize)]
struct ExtensionsResponse<T> {
    extensions: T,
}

#[derive(Deserialize)]
struct InstallRequest {
    manifest_url: String,
}

#[derive(Deserialize)]
struct UpdateSettingsRequest {
    settings: HashMap<String, Value>,
}

pub fn extensions_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/extensions", get(get_extensions))
        .route("/extensions/anime", get(get_anime_extensions))
        .route("/extensions/manga", get(get_manga_extensions))
        .route("/extensions/novel", get(get_novel_extensions))
        .route("/extensions/install", post(install_extension))
        .route("/extensions/:id/uninstall", delete(uninstall_extension))
        .route("/extensions/:id/settings", get(get_extension_settings))
        .route("/extensions/:id/settings", put(update_extension_settings))
        .route("/extensions/:name/filters", get(get_extension_filters))
}

async fn get_extensions(State(state): State<Arc<AppState>>) -> AppResult<Json<ExtensionsResponse<Vec<Extension>>>> {
    let manager = state.extension_manager.read().await;
    let list = manager
        .list_extensions()
        .iter()
        .map(|e| (*e).clone())
        .collect();

    Ok(Json(ExtensionsResponse { extensions: list }))
}

async fn get_anime_extensions(State(state): State<Arc<AppState>>) -> AppResult<Json<ExtensionsResponse<Vec<String>>>> {
    let manager = state.extension_manager.read().await;
    let list = manager.get_extensions_by_type(ExtensionType::Anime);
    Ok(Json(ExtensionsResponse { extensions: list }))
}

async fn get_manga_extensions(State(state): State<Arc<AppState>>) -> AppResult<Json<ExtensionsResponse<Vec<String>>>> {
    let manager = state.extension_manager.read().await;
    let list = manager.get_extensions_by_type(ExtensionType::Manga);
    Ok(Json(ExtensionsResponse { extensions: list }))
}

async fn get_novel_extensions(State(state): State<Arc<AppState>>) -> AppResult<Json<ExtensionsResponse<Vec<String>>>> {
    let manager = state.extension_manager.read().await;
    let list = manager.get_extensions_by_type(ExtensionType::Novel);
    Ok(Json(ExtensionsResponse { extensions: list }))
}

async fn install_extension(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<InstallRequest>,
) -> AppResult<Json<Value>> {
    let mut manager = state.extension_manager.write().await;
    let extension = manager.install_extension(&payload.manifest_url).await?;
    Ok(Json(json!({
        "ok": true,
        "extension": extension,
    })))
}

async fn uninstall_extension(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Value>> {
    let mut manager = state.extension_manager.write().await;
    manager.uninstall_extension(&id).await?;
    Ok(Json(json!({ "ok": true, "id": id })))
}

async fn get_extension_settings(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Value>> {
    let manager = state.extension_manager.read().await;

    let settings = manager
        .call_extension_function(&id, "getSettings", vec![])
        .await
        .unwrap_or_else(|_| {
            json!({
                "episodeServers": ["default"],
                "supportsDub": false
            })
        });

    Ok(Json(settings))
}

async fn update_extension_settings(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpdateSettingsRequest>,
) -> AppResult<Json<Value>> {
    let mut manager = state.extension_manager.write().await;
    manager.update_extension_settings(&id, payload.settings).await?;
    Ok(Json(json!({ "ok": true, "id": id })))
}

async fn get_extension_filters(
    Path(name): Path<String>,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Value>> {
    let manager = state.extension_manager.read().await;

    match manager.call_extension_function(&name, "getFilters", vec![]).await {
        Ok(filters) => Ok(Json(json!({ "filters": filters }))),
        Err(_) => Ok(Json(json!({ "filters": {} }))),
    }
}