use crate::{require_auth, TauriSession};
use hoshi_core::{
    state::AppState,
    backup::repository::ListBackupMeta,
    backup::service::BackupService,
    tracker::service::SuccessResponse,
};
use std::sync::Arc;
use tauri::State;
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub async fn list_backups(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
) -> Result<Vec<ListBackupMeta>, String> {
    let user_id = require_auth(&session_state).await?
        .parse::<i32>().map_err(|_| "Invalid user ID")?;

    BackupService::list_backups(&state, user_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_manual_backup(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
) -> Result<ListBackupMeta, String> {
    let user_id = require_auth(&session_state).await?
        .parse::<i32>().map_err(|_| "Invalid user ID")?;

    BackupService::create_manual(&state, user_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_backup(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    backup_id: i64,
) -> Result<SuccessResponse, String> {
    let user_id = require_auth(&session_state).await?
        .parse::<i32>().map_err(|_| "Invalid user ID")?;

    let deleted = BackupService::delete_backup(&state, user_id, backup_id)
        .map_err(|e| e.to_string())?;

    Ok(SuccessResponse { success: deleted })
}

#[tauri::command]
pub async fn restore_backup(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    backup_id: i64,
) -> Result<SuccessResponse, String> {
    let user_id = require_auth(&session_state).await?
        .parse::<i32>().map_err(|_| "Invalid user ID")?;

    BackupService::restore_backup(&state, user_id, backup_id)
        .map_err(|e| e.to_string())?;

    Ok(SuccessResponse { success: true })
}

#[tauri::command]
pub async fn download_backup(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    app_handle: tauri::AppHandle,
    backup_id: i64,
) -> Result<SuccessResponse, String> {
    let user_id = require_auth(&session_state).await?
        .parse::<i32>().map_err(|_| "Invalid user ID")?;

    let json = BackupService::read_backup_json(&state, user_id, backup_id)
        .map_err(|e| e.to_string())?;

    let (tx, rx) = tokio::sync::oneshot::channel();

    app_handle
        .dialog()
        .file()
        .set_title("Guardar backup")
        .set_file_name(format!("backup_{}.json", backup_id))
        .add_filter("JSON", &["json"])
        .save_file(move |path| {
            let _ = tx.send(path);
        });

    match rx.await.map_err(|e| e.to_string())? {
        Some(file_path) => {
            let path = file_path.into_path()
                .map_err(|e| format!("Invalid path: {}", e))?;
            std::fs::write(path, json)
                .map_err(|e| format!("Could not write file: {}", e))?;
            Ok(SuccessResponse { success: true })
        }
        None => Ok(SuccessResponse { success: false }),
    }
}