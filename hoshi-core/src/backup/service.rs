use std::path::PathBuf;
use tracing::{error, info, instrument, warn};
use crate::backup::repository::BackupRepository;
use crate::backup::types::{BackupTrigger, ListBackupMeta};
use crate::error::{CoreError, CoreResult};
use crate::list::repository::ListRepo;
use crate::list::types::UpsertEntryBody;
use crate::state::AppState;

pub struct BackupService;

impl BackupService {

    #[instrument(skip(state))]
    pub fn list_backups(state: &AppState, user_id: i32) -> CoreResult<Vec<ListBackupMeta>> {
        let conn = state.db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("error.system.db_lock".into()))?;
        BackupRepository::list_backups(&conn_lock, user_id)
    }

    #[instrument(skip(state))]
    pub fn create_manual(state: &AppState, user_id: i32) -> CoreResult<ListBackupMeta> {
        info!("Starting manual backup creation");
        let conn = state.db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("error.system.db_lock".into()))?;

        let backup_id = BackupRepository::create_snapshot(
            &conn_lock,
            &state.paths,
            user_id,
            BackupTrigger::Manual,
            None,
        )?;

        let meta = BackupRepository::get_backup_meta(&conn_lock, user_id, backup_id)?
            .ok_or_else(|| {
                error!("Backup was created but metadata could not be found");
                CoreError::Internal("error.backup.creation_failed".into())
            })?;

        info!(backup_id = backup_id, "Manual backup created successfully");
        Ok(meta)
    }

    #[instrument(skip(state))]
    pub fn delete_backup(state: &AppState, user_id: i32, backup_id: i64) -> CoreResult<bool> {
        info!("Deleting backup from system");
        let conn = state.db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("error.system.db_lock".into()))?;

        let deleted = BackupRepository::delete_backup(&conn_lock, &state.paths, user_id, backup_id)?;
        if deleted {
            info!("Backup deleted successfully");
        } else {
            warn!("Attempted to delete a backup that does not exist");
        }

        Ok(deleted)
    }

    #[instrument(skip(state))]
    pub fn restore_backup(state: &AppState, user_id: i32, backup_id: i64) -> CoreResult<usize> {
        info!("Starting backup restoration process");
        let conn = state.db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("error.system.db_lock".into()))?;

        let meta = BackupRepository::get_backup_meta(&conn_lock, user_id, backup_id)?
            .ok_or_else(|| {
                warn!("Backup not found for restoration");
                CoreError::NotFound("error.backup.not_found".into())
            })?;

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

        info!(restored_entries = restored, "Backup restored successfully");
        Ok(restored)
    }

    #[instrument(skip(state))]
    pub fn read_backup_json(state: &AppState, user_id: i32, backup_id: i64) -> CoreResult<String> {
        let conn = state.db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("error.system.db_lock".into()))?;

        let meta = BackupRepository::get_backup_meta(&conn_lock, user_id, backup_id)?
            .ok_or_else(|| CoreError::NotFound("error.backup.not_found".into()))?;

        let full_path = state.paths.base_dir.join(&meta.file_path);
        std::fs::read_to_string(&full_path).map_err(Into::into)
    }

    #[instrument(skip(state))]
    pub fn get_backup_path(state: &AppState, user_id: i32, backup_id: i64) -> CoreResult<PathBuf> {
        let conn = state.db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("error.system.db_lock".into()))?;

        let meta = BackupRepository::get_backup_meta(&conn_lock, user_id, backup_id)?
            .ok_or_else(|| CoreError::NotFound("error.backup.not_found".into()))?;

        let full_path = state.paths.base_dir.join(&meta.file_path);

        Ok(full_path)
    }
}