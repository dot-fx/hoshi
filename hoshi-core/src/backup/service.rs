use std::path::PathBuf;
use crate::backup::repository::{BackupRepository, BackupTrigger, ListBackupMeta};
use crate::error::{CoreError, CoreResult};
use crate::list::repository::ListRepo;
use crate::list::service::UpsertEntryBody;
use crate::state::AppState;

pub struct BackupService;

impl BackupService {

    pub fn list_backups(state: &AppState, user_id: i32) -> CoreResult<Vec<ListBackupMeta>> {
        let conn = state.db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;
        BackupRepository::list_backups(&conn_lock, user_id)
    }

    pub fn create_manual(state: &AppState, user_id: i32) -> CoreResult<ListBackupMeta> {
        let conn = state.db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;

        let backup_id = BackupRepository::create_snapshot(
            &conn_lock,
            &state.paths,
            user_id,
            BackupTrigger::Manual,
            None,
        )?;

        BackupRepository::get_backup_meta(&conn_lock, user_id, backup_id)?
            .ok_or_else(|| CoreError::Internal("Backup created but not found".into()))
    }

    pub fn delete_backup(state: &AppState, user_id: i32, backup_id: i64) -> CoreResult<bool> {
        let conn = state.db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;
        BackupRepository::delete_backup(&conn_lock, &state.paths, user_id, backup_id)
    }

    pub fn restore_backup(state: &AppState, user_id: i32, backup_id: i64) -> CoreResult<usize> {
        let conn = state.db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;

        let meta = BackupRepository::get_backup_meta(&conn_lock, user_id, backup_id)?
            .ok_or_else(|| CoreError::NotFound(format!("Backup {} not found", backup_id)))?;

        let entries = BackupRepository::read_snapshot(&state.paths, &meta)?;

        let mut restored = 0;
        for entry in entries {
            let body = UpsertEntryBody {
                cid: entry.cid.clone(),
                status: entry.status.clone(),
                progress: Some(entry.progress),
                score: entry.score,
                start_date: entry.start_date.clone(),
                end_date: entry.end_date.clone(),
                repeat_count: Some(entry.repeat_count),
                notes: entry.notes.clone(),
                is_private: Some(entry.is_private),
            };

            ListRepo::upsert_entry(
                &conn_lock,
                user_id,
                &body,
                &entry.status,
                entry.progress,
                entry.start_date,
                entry.end_date,
            )?;

            restored += 1;
        }

        Ok(restored)
    }

    pub fn read_backup_json(state: &AppState, user_id: i32, backup_id: i64) -> CoreResult<String> {
        let conn = state.db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;

        let meta = BackupRepository::get_backup_meta(&conn_lock, user_id, backup_id)?
            .ok_or_else(|| CoreError::NotFound(format!("Backup {} not found", backup_id)))?;

        let full_path = state.paths.base_dir.join(&meta.file_path);
        std::fs::read_to_string(&full_path)
            .map_err(|e| CoreError::Internal(format!("Could not read backup file: {}", e)))
    }

    pub fn get_backup_path(state: &AppState, user_id: i32, backup_id: i64) -> CoreResult<PathBuf> {
        let conn = state.db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;

        let meta = BackupRepository::get_backup_meta(&conn_lock, user_id, backup_id)?
            .ok_or_else(|| CoreError::NotFound(format!("Backup {} not found", backup_id)))?;

        let full_path = state.paths.base_dir.join(&meta.file_path);

        Ok(full_path)
    }
}