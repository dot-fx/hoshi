use crate::{require_auth, TauriSession};
use hoshi_core::{
    backup::repository::ListBackupMeta,
    backup::service::BackupService,
    error::CoreError,
    state::AppState,
    tracker::service::SuccessResponse,
};
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn list_backups(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
) -> Result<Vec<ListBackupMeta>, CoreError> {
    let user_id = require_auth(&session_state).await?;

    BackupService::list_backups(&state, user_id)
}

#[tauri::command]
pub async fn create_manual_backup(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
) -> Result<ListBackupMeta, CoreError> {
    let user_id = require_auth(&session_state).await?;

    BackupService::create_manual(&state, user_id)
}

#[tauri::command]
pub async fn delete_backup(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    backup_id: i64,
) -> Result<SuccessResponse, CoreError> {
    let user_id = require_auth(&session_state).await?;

    let deleted = BackupService::delete_backup(&state, user_id, backup_id)?;

    Ok(SuccessResponse { success: deleted })
}

#[tauri::command]
pub async fn restore_backup(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    backup_id: i64,
) -> Result<SuccessResponse, CoreError> {
    let user_id = require_auth(&session_state).await?;

    BackupService::restore_backup(&state, user_id, backup_id)?;

    Ok(SuccessResponse { success: true })
}

#[tauri::command]
pub async fn download_backup(
    #[allow(unused_variables)] app_handle: tauri::AppHandle,
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    backup_id: i64,
) -> Result<SuccessResponse, CoreError> {
    let user_id = require_auth(&session_state).await?;

    #[cfg(not(target_os = "android"))]
    {
        let backup_path = BackupService::get_backup_path(&state, user_id, backup_id)?;

        if backup_path.exists() {
            reveal_in_folder(&backup_path);
            Ok(SuccessResponse { success: true })
        } else {
            Err(CoreError::NotFound("error.backup.file_not_found".into()))
        }
    }

    #[cfg(target_os = "android")]
    {
        let json = BackupService::read_backup_json(&state, user_id, backup_id)?;

        let download_dir = app_handle.path().download_dir()
            .map_err(|_| CoreError::Internal("error.system.io".into()))?;

        if !download_dir.exists() {
            std::fs::create_dir_all(&download_dir)?;
        }

        let file_path = download_dir.join(format!("hoshi_backup_{}.json", backup_id));
        std::fs::write(&file_path, json)?;

        Ok(SuccessResponse { success: true })
    }
}

#[cfg(not(target_os = "android"))]
fn reveal_in_folder(path: &std::path::Path) {
    use std::process::Command;

    #[cfg(target_os = "windows")]
    {
        // /select permite resaltar el archivo específicamente
        Command::new("explorer")
            .arg("/select,")
            .arg(path)
            .spawn()
            .ok();
    }

    #[cfg(target_os = "macos")]
    {
        // -R hace el "reveal" en el Finder
        Command::new("open")
            .arg("-R")
            .arg(path)
            .spawn()
            .ok();
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(parent) = path.parent() {
            Command::new("xdg-open")
                .arg(parent)
                .spawn()
                .ok();
        }
    }
}