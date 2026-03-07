use hoshi_core::{
    booru::service::{AutocompleteQuery, BooruService, ImageInfo, InfoQuery, SearchQuery, SearchResponse},
    state::AppState,
};
use serde_json::Value;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn booru_search(
    state: State<'_, Arc<AppState>>,
    params: SearchQuery,
) -> Result<SearchResponse, String> {
    BooruService::search_in_extension(&state.extension_manager, params)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn booru_get_info(
    state: State<'_, Arc<AppState>>,
    id: String,
    provider: String,
) -> Result<ImageInfo, String> {
    BooruService::get_info(&state.extension_manager, id, Option::from(provider))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn booru_autocomplete(
    state: State<'_, Arc<AppState>>,
    provider: String,
    q: Option<String>,
) -> Result<Value, String> {
    BooruService::get_autocomplete(&state.extension_manager, provider, q)
        .await
        .map_err(|e| e.to_string())
}

// serve_local_image no tiene command — en Tauri se sirve via custom protocol.
// Registrar en src-tauri/src/main.rs:
//
// .register_uri_scheme_protocol("booru", |_app, req| {
//     let path = req.uri().path(); // "/provider/filename"
//     let parts: Vec<&str> = path.trim_start_matches('/').splitn(2, '/').collect();
//     let (provider, filename) = (parts[0], parts[1]);
//     let (content_type, bytes) = BooruService::serve_local_image(provider, filename)
//         .await
//         .unwrap_or_default();
//     ResponseBuilder::new()
//         .header("Content-Type", content_type)
//         .header("Cache-Control", "public, max-age=31536000")
//         .body(bytes)
// })
//
// Y usar booruApi.localImageUrl() que devuelve "booru://local/provider/filename" en Tauri.