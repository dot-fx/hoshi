use hoshi_core::{
    extensions::ExtensionType,
    state::AppState,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn get_extensions(
    state: State<'_, Arc<AppState>>,
) -> Result<Value, String> {
    let list: Vec<String> = state.inner().extension_manager // Usamos .inner()
        .list_extensions()
        .iter()
        .map(|e| e.name.clone())
        .collect();
    // Devolvemos el objeto unificado
    Ok(json!({ "extensions": list }))
}

#[tauri::command]
pub async fn get_anime_extensions(
    state: State<'_, Arc<AppState>>,
) -> Result<Value, String> {
    let list = state.inner().extension_manager.get_extensions_by_type(ExtensionType::Anime);
    Ok(json!({ "extensions": list }))
}

#[tauri::command]
pub async fn get_manga_extensions(
    state: State<'_, Arc<AppState>>,
) -> Result<Value, String> {
    let list = state.inner().extension_manager.get_extensions_by_type(ExtensionType::Manga);
    Ok(json!({ "extensions": list }))
}

#[tauri::command]
pub async fn get_novel_extensions(
    state: State<'_, Arc<AppState>>,
) -> Result<Value, String> {
    let list = state.inner().extension_manager.get_extensions_by_type(ExtensionType::Novel);
    Ok(json!({ "extensions": list }))
}

#[tauri::command]
pub async fn get_booru_extensions(
    state: State<'_, Arc<AppState>>,
) -> Result<Value, String> {
    let list = state.inner().extension_manager.get_extensions_by_type(ExtensionType::Booru);
    Ok(json!({ "extensions": list }))
}

#[tauri::command]
pub async fn get_extension_settings(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<Value, String> {
    Ok(state.inner().extension_manager
        .call_extension_function(&id, "getSettings", vec![])
        .await
        .unwrap_or_else(|_| json!({
            "episodeServers": ["default"],
            "supportsDub": false
        })))
}

#[tauri::command]
pub async fn get_extension_filters(
    state: State<'_, Arc<AppState>>,
    name: String,
) -> Result<Value, String> {
    Ok(state.inner().extension_manager
        .call_extension_function(&name, "getFilters", vec![])
        .await
        .unwrap_or_else(|_| json!({})))
}