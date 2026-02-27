use crate::error::AppResult;
use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::sync::Arc;

use hoshi_core::{
    extensions::ExtensionType
    ,
    state::AppState
};

#[derive(Serialize)]
struct ExtensionsResponse<T> {
    extensions: T,
}

pub fn extensions_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/extensions", get(get_extensions))
        .route("/extensions/anime", get(get_anime_extensions))
        .route("/extensions/manga", get(get_manga_extensions))
        .route("/extensions/novel", get(get_novel_extensions))
        .route("/extensions/booru", get(get_booru_extensions))
        .route("/extensions/:name/settings", get(get_extension_settings))
        .route("/extensions/:name/filters", get(get_extension_filters))
}

async fn get_extensions(State(state): State<Arc<AppState>>) -> AppResult<Json<ExtensionsResponse<Vec<String>>>> {
    let manager = state.extension_manager.clone();
    let list = manager
        .list_extensions()
        .iter()
        .map(|e| e.name.clone())
        .collect();

    Ok(Json(ExtensionsResponse { extensions: list }))
}

async fn get_anime_extensions(State(state): State<Arc<AppState>>) -> AppResult<Json<ExtensionsResponse<Vec<String>>>> {
    let manager = state.extension_manager.clone();
    let list = manager.get_extensions_by_type(ExtensionType::Anime);
    Ok(Json(ExtensionsResponse { extensions: list }))
}

async fn get_manga_extensions(State(state): State<Arc<AppState>>) -> AppResult<Json<ExtensionsResponse<Vec<String>>>> {
    let manager = state.extension_manager.clone();
    let list = manager.get_extensions_by_type(ExtensionType::Manga);
    Ok(Json(ExtensionsResponse { extensions: list }))
}

async fn get_novel_extensions(State(state): State<Arc<AppState>>) -> AppResult<Json<ExtensionsResponse<Vec<String>>>> {
    let manager = state.extension_manager.clone();
    let list = manager.get_extensions_by_type(ExtensionType::Novel);
    Ok(Json(ExtensionsResponse { extensions: list }))
}

async fn get_booru_extensions(State(state): State<Arc<AppState>>) -> AppResult<Json<ExtensionsResponse<Vec<String>>>> {
    let manager = state.extension_manager.clone();
    let list = manager.get_extensions_by_type(ExtensionType::Booru);
    Ok(Json(ExtensionsResponse { extensions: list }))
}

async fn get_extension_settings(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Value>> {
    let manager = state.extension_manager.clone();

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

async fn get_extension_filters(
    Path(name): Path<String>,
    State(state): State<Arc<AppState>>
) -> AppResult<Json<Value>> {
    let manager = state.extension_manager.clone();

    match manager.call_extension_function(&name, "getFilters", vec![]).await {
        Ok(filters) => Ok(Json(json!({ "filters": filters }))),
        Err(_) => {
            Ok(Json(json!({ "filters": {} })))
        }
    }
}