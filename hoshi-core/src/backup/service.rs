use std::path::PathBuf;
use tracing::{error, info, instrument, warn};

use crate::backup::repository::BackupRepository;
use crate::backup::types::{BackupTrigger, ListBackupMeta};
use crate::error::{CoreError, CoreResult};
use crate::list::repository::ListRepository;
use crate::list::types::UpsertEntryBody;
use crate::state::AppState;

pub struct BackupService;

impl BackupService {

    #[instrument(skip(state))]
    pub async fn list_backups(state: &AppState, user_id: i32) -> CoreResult<Vec<ListBackupMeta>> {
        BackupRepository::list_backups(&state.pool, user_id).await
    }

    #[instrument(skip(state))]
    pub async fn create_manual(state: &AppState, user_id: i32) -> CoreResult<ListBackupMeta> {
        info!("Starting manual backup creation");

        let backup_id = BackupRepository::create_snapshot(
            &state.pool,
            &state.paths,
            user_id,
            BackupTrigger::Manual,
            None,
        ).await?;

        let meta = BackupRepository::get_backup_meta(&state.pool, user_id, backup_id)
            .await?
            .ok_or_else(|| {
                error!("Backup created but metadata not found");
                CoreError::Internal("error.backup.creation_failed".into())
            })?;

        info!(backup_id = backup_id, "Manual backup created");
        Ok(meta)
    }

    #[instrument(skip(state))]
    pub async fn delete_backup(state: &AppState, user_id: i32, backup_id: i64) -> CoreResult<bool> {
        let deleted = BackupRepository::delete_backup(
            &state.pool, &state.paths, user_id, backup_id,
        ).await?;

        if deleted {
            info!(backup_id = backup_id, "Backup deleted");
        } else {
            warn!(backup_id = backup_id, "Backup not found for deletion");
        }
        Ok(deleted)
    }

    #[instrument(skip(state))]
    pub async fn restore_backup(state: &AppState, user_id: i32, backup_id: i64) -> CoreResult<usize> {
        info!("Starting backup restoration");

        let meta = BackupRepository::get_backup_meta(&state.pool, user_id, backup_id)
            .await?
            .ok_or_else(|| {
                warn!("Backup not found for restoration");
                CoreError::NotFound("error.backup.not_found".into())
            })?;

        let entries = BackupRepository::read_snapshot(&state.paths, &meta)?;
        let mut restored = 0;

        for entry in entries {
            ListRepository::upsert_entry(
                &state.pool,
                user_id,
                &UpsertEntryBody {
                    cid:          entry.cid,
                    status:       entry.status.clone(),
                    progress:     Some(entry.progress),
                    score:        entry.score,
                    start_date:   entry.start_date.clone(),
                    end_date:     entry.end_date.clone(),
                    repeat_count: Some(entry.repeat_count),
                    notes:        entry.notes,
                    is_private:   Some(entry.is_private),
                },
                &entry.status,
                entry.progress,
                entry.start_date,
                entry.end_date,
            ).await?;
            restored += 1;
        }

        info!(restored = restored, "Backup restored");
        Ok(restored)
    }

    #[instrument(skip(state))]
    pub async fn read_backup_json(state: &AppState, user_id: i32, backup_id: i64) -> CoreResult<String> {
        let meta = BackupRepository::get_backup_meta(&state.pool, user_id, backup_id)
            .await?
            .ok_or_else(|| CoreError::NotFound("error.backup.not_found".into()))?;

        let full_path = state.paths.base_dir.join(&meta.file_path);
        std::fs::read_to_string(&full_path).map_err(Into::into)
    }

    #[instrument(skip(state))]
    pub async fn get_backup_path(state: &AppState, user_id: i32, backup_id: i64) -> CoreResult<PathBuf> {
        let meta = BackupRepository::get_backup_meta(&state.pool, user_id, backup_id)
            .await?
            .ok_or_else(|| CoreError::NotFound("error.backup.not_found".into()))?;

        Ok(state.paths.base_dir.join(&meta.file_path))
    }
}