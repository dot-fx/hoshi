use sqlx::SqlitePool;
use crate::backup::types::{BackupTrigger, ListBackupMeta, ListEntrySnapshot};
use crate::error::{CoreError, CoreResult};
use crate::list::repository::ListRepository;
use crate::paths::AppPaths;
use crate::tracker::provider::UserListEntry;

pub struct BackupRepository;

impl BackupRepository {

    pub async fn save_remote_list(
        pool: &SqlitePool,
        paths: &AppPaths,
        user_id: i32,
        tracker_name: &str,
        entries: &[UserListEntry],
    ) -> CoreResult<i64> {
        let now = chrono::Utc::now().timestamp();
        let json = serde_json::to_string_pretty(entries)?;

        let file_path = paths.base_dir
            .join("backups")
            .join(user_id.to_string())
            .join(format!("remote_{}_{}.json", tracker_name, now));

        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&file_path, &json)?;

        let relative_path = file_path
            .strip_prefix(&paths.base_dir)
            .unwrap_or(&file_path)
            .to_string_lossy()
            .to_string();

        let entry_count = entries.len() as i32;

        let existing_id: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM ListBackup
             WHERE user_id = ? AND trigger = 'REMOTE_SYNC' AND tracker_name = ?",
        )
            .bind(user_id)
            .bind(tracker_name)
            .fetch_optional(pool)
            .await?;

        if let Some(id) = existing_id {
            sqlx::query(
                "UPDATE ListBackup SET file_path = ?, entry_count = ?, created_at = ? WHERE id = ?",
            )
                .bind(&relative_path)
                .bind(entry_count)
                .bind(now)
                .bind(id)
                .execute(pool)
                .await?;
            Ok(id)
        } else {
            let id = sqlx::query_scalar::<_, i64>(
                "INSERT INTO ListBackup (user_id, trigger, tracker_name, file_path, entry_count, created_at)
                 VALUES (?, ?, ?, ?, ?, ?) RETURNING id",
            )
                .bind(user_id)
                .bind(BackupTrigger::RemoteSync.as_str())
                .bind(tracker_name)
                .bind(&relative_path)
                .bind(entry_count)
                .bind(now)
                .fetch_one(pool)
                .await?;
            Ok(id)
        }
    }

    pub async fn create_snapshot(
        pool: &SqlitePool,
        paths: &AppPaths,
        user_id: i32,
        trigger: BackupTrigger,
        tracker_name: Option<&str>,
    ) -> CoreResult<i64> {
        let entries = ListRepository::get_entries(pool, user_id, None).await?;
        if entries.is_empty() && trigger == BackupTrigger::PreImport {
            return Ok(0);
        }

        let snapshots: Vec<ListEntrySnapshot> = entries
            .into_iter()
            .map(|e| ListEntrySnapshot {
                cid:          e.cid,
                status:       e.status,
                progress:     e.progress,
                score:        e.score,
                start_date:   e.start_date,
                end_date:     e.end_date,
                repeat_count: e.repeat_count,
                notes:        e.notes,
                is_private:   e.is_private,
            })
            .collect();

        let now = chrono::Utc::now().timestamp();
        let json = serde_json::to_string_pretty(&snapshots)?;
        let entry_count = snapshots.len() as i32;

        let file_path = match trigger {
            BackupTrigger::PreImport => {
                let name = tracker_name.ok_or_else(|| {
                    CoreError::Internal("error.backup.missing_tracker_name".into())
                })?;
                paths.pre_import_backup_path(user_id, name)
            }
            BackupTrigger::Manual    => paths.manual_backup_path(user_id, now),
            BackupTrigger::RemoteSync => return Err(CoreError::Internal("error.backup.invalid_trigger".into())),
        };

        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&file_path, &json)?;

        let relative_path = paths.relative_backup_path(&file_path);

        let backup_id = match trigger {
            BackupTrigger::PreImport => {
                let existing_id: Option<i64> = sqlx::query_scalar(
                    "SELECT id FROM ListBackup
                     WHERE user_id = ? AND trigger = 'PRE_IMPORT' AND tracker_name = ?",
                )
                    .bind(user_id)
                    .bind(tracker_name)
                    .fetch_optional(pool)
                    .await?;

                if let Some(id) = existing_id {
                    sqlx::query(
                        "UPDATE ListBackup SET file_path = ?, entry_count = ?, created_at = ? WHERE id = ?",
                    )
                        .bind(&relative_path)
                        .bind(entry_count)
                        .bind(now)
                        .bind(id)
                        .execute(pool)
                        .await?;
                    id
                } else {
                    sqlx::query_scalar::<_, i64>(
                        "INSERT INTO ListBackup (user_id, trigger, tracker_name, file_path, entry_count, created_at)
                         VALUES (?, ?, ?, ?, ?, ?) RETURNING id",
                    )
                        .bind(user_id)
                        .bind(BackupTrigger::PreImport.as_str())
                        .bind(tracker_name)
                        .bind(&relative_path)
                        .bind(entry_count)
                        .bind(now)
                        .fetch_one(pool)
                        .await?
                }
            }
            BackupTrigger::Manual => {
                sqlx::query_scalar::<_, i64>(
                    "INSERT INTO ListBackup (user_id, trigger, tracker_name, file_path, entry_count, created_at)
                     VALUES (?, ?, NULL, ?, ?, ?) RETURNING id",
                )
                    .bind(user_id)
                    .bind(BackupTrigger::Manual.as_str())
                    .bind(&relative_path)
                    .bind(entry_count)
                    .bind(now)
                    .fetch_one(pool)
                    .await?
            }
            BackupTrigger::RemoteSync => unreachable!(),
        };

        Ok(backup_id)
    }

    pub async fn list_backups(pool: &SqlitePool, user_id: i32) -> CoreResult<Vec<ListBackupMeta>> {
        let rows = sqlx::query_as::<_, ListBackupMeta>(
            "SELECT id, user_id, trigger, tracker_name, file_path, entry_count, created_at
             FROM ListBackup WHERE user_id = ? ORDER BY created_at DESC",
        )
            .bind(user_id)
            .fetch_all(pool)
            .await?;
        Ok(rows)
    }

    pub async fn get_backup_meta(
        pool: &SqlitePool,
        user_id: i32,
        backup_id: i64,
    ) -> CoreResult<Option<ListBackupMeta>> {
        let row = sqlx::query_as::<_, ListBackupMeta>(
            "SELECT id, user_id, trigger, tracker_name, file_path, entry_count, created_at
             FROM ListBackup WHERE id = ? AND user_id = ?",
        )
            .bind(backup_id)
            .bind(user_id)
            .fetch_optional(pool)
            .await?;
        Ok(row)
    }

    pub fn read_snapshot(paths: &AppPaths, meta: &ListBackupMeta) -> CoreResult<Vec<ListEntrySnapshot>> {
        let full_path = paths.base_dir.join(&meta.file_path);
        let json = std::fs::read_to_string(&full_path)?;
        Ok(serde_json::from_str(&json)?)
    }

    pub async fn delete_backup(
        pool: &SqlitePool,
        paths: &AppPaths,
        user_id: i32,
        backup_id: i64,
    ) -> CoreResult<bool> {
        let Some(meta) = Self::get_backup_meta(pool, user_id, backup_id).await? else {
            return Ok(false);
        };

        let full_path = paths.base_dir.join(&meta.file_path);
        if full_path.exists() {
            std::fs::remove_file(&full_path)?;
        }

        let rows = sqlx::query(
            "DELETE FROM ListBackup WHERE id = ? AND user_id = ?",
        )
            .bind(backup_id)
            .bind(user_id)
            .execute(pool)
            .await?
            .rows_affected();

        Ok(rows > 0)
    }
}