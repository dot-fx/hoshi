use hoshi_core::{
    extensions::Extension,
    state::AppState,
    error::CoreError,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;

#[derive(Serialize)]
pub struct ExtensionsResponse<T> {
    extensions: T,
}

#[tauri::command]
pub async fn get_extensions(
    state: State<'_, Arc<AppState>>,
) -> Result<ExtensionsResponse<Vec<Extension>>, CoreError> {
    let manager = state.inner().extension_manager.read().await;
    let list: Vec<Extension> = manager
        .list_extensions()
        .iter()
        .map(|e| (*e).clone())
        .collect();

    Ok(ExtensionsResponse { extensions: list })
}

#[tauri::command]
pub async fn install_extension(
    state: State<'_, Arc<AppState>>,
    manifest_url: String,
) -> Result<Value, CoreError> {
    let mut manager = state.inner().extension_manager.write().await;
    let extension = manager.install_extension(&manifest_url).await?;
    Ok(json!({ "ok": true, "extension": extension }))
}

#[tauri::command]
pub async fn uninstall_extension(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<Value, CoreError> {
    let mut manager = state.inner().extension_manager.write().await;
    manager.uninstall_extension(&id).await?;
    Ok(json!({ "ok": true, "id": id }))
}

#[tauri::command]
pub async fn update_extension_settings(
    state: State<'_, Arc<AppState>>,
    id: String,
    settings: HashMap<String, Value>,
) -> Result<Value, CoreError> {
    let mut manager = state.inner().extension_manager.write().await;
    manager.update_extension_settings(&id, settings).await?;
    Ok(json!({ "ok": true, "id": id }))
}

#[tauri::command]
pub async fn get_extension_settings(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<Value, CoreError> {
    let manager = state.inner().extension_manager.read().await;
    Ok(manager
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
) -> Result<Value, CoreError> {
    let manager = state.inner().extension_manager.read().await;
    let filters = manager
        .call_extension_function(&name, "getFilters", vec![])
        .await
        .unwrap_or_else(|_| json!({}));
    Ok(json!({ "filters": filters }))
}