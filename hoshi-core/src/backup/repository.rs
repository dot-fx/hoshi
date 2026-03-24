use crate::error::{CoreError, CoreResult};
use crate::list::repository::ListRepo;
use crate::paths::AppPaths;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Tipos
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackupTrigger {
    PreImport,
    Manual,
    RemoteSync
}

impl BackupTrigger {
    fn as_str(&self) -> &'static str {
        match self {
            BackupTrigger::PreImport => "PRE_IMPORT",
            BackupTrigger::Manual => "MANUAL",
            BackupTrigger::RemoteSync => "REMOTE_SYNC",
        }
    }
}

/// Metadatos del backup (lo que vive en DB).
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListBackupMeta {
    pub id: i64,
    pub user_id: i32,
    pub trigger: String,
    pub tracker_name: Option<String>,
    pub file_path: String,
    pub entry_count: i32,
    pub created_at: i64,
}

/// Subconjunto de ListEntry que se persiste en el fichero JSON.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListEntrySnapshot {
    pub cid: String,
    pub status: String,
    pub progress: i32,
    pub score: Option<f64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub repeat_count: i32,
    pub notes: Option<String>,
    pub is_private: bool,
}

// ---------------------------------------------------------------------------
// Repositorio
// ---------------------------------------------------------------------------

pub struct BackupRepository;
use crate::tracker::provider::UserListEntry;
impl BackupRepository {


    pub fn save_remote_list(
        conn: &Connection,
        paths: &AppPaths,
        user_id: i32,
        tracker_name: &str,
        entries: &[UserListEntry]
    ) -> CoreResult<i64> {
        let entry_count = entries.len() as i32;

        let json = serde_json::to_string_pretty(entries)
            .map_err(|e| CoreError::Internal(e.to_string()))?;

        let now = chrono::Utc::now().timestamp();

        let file_name = format!("remote_{}_{}.json", tracker_name, now);
        let file_path = paths.base_dir.join("backups").join(user_id.to_string()).join(&file_name);

        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| CoreError::Internal(format!("Could not create remote backup dir: {}", e)))?;
        }

        std::fs::write(&file_path, &json)
            .map_err(|e| CoreError::Internal(format!("Could not write remote backup file: {}", e)))?;

        let relative_path = file_path.strip_prefix(&paths.base_dir)
            .unwrap_or(&file_path)
            .to_string_lossy()
            .to_string();

        let existing_id: Option<i64> = conn.query_row(
            "SELECT id FROM ListBackup
             WHERE user_id = ?1 AND trigger = 'REMOTE_SYNC' AND tracker_name = ?2",
            params![user_id, tracker_name],
            |row| row.get(0),
        ).optional()?;

        if let Some(id) = existing_id {
            // SOBRESCRIBIR
            conn.execute(
                "UPDATE ListBackup
                 SET file_path = ?1, entry_count = ?2, created_at = ?3
                 WHERE id = ?4",
                params![relative_path, entry_count, now, id],
            )?;
            Ok(id)
        } else {
            // CREAR NUEVO
            conn.execute(
                "INSERT INTO ListBackup (user_id, trigger, tracker_name, file_path, entry_count, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![user_id, BackupTrigger::RemoteSync.as_str(), tracker_name, relative_path, entry_count, now],
            )?;
            Ok(conn.last_insert_rowid())
        }
    }

    /// Crea un snapshot de la lista del usuario:
    /// - Escribe el JSON en disco.
    /// - Guarda/actualiza los metadatos en DB.
    ///
    /// Para `PreImport`: si ya existe un backup previo del mismo tracker,
    /// sobreescribe el fichero y actualiza el registro en DB (1 por tracker).
    /// Para `Manual`: siempre crea un fichero y registro nuevos.
    pub fn create_snapshot(
        conn: &Connection,
        paths: &AppPaths,
        user_id: i32,
        trigger: BackupTrigger,
        tracker_name: Option<&str>,
    ) -> CoreResult<i64> {
        // 1. Leer la lista actual
        let entries = ListRepo::get_entries(conn, user_id, None)?;
        let snapshots: Vec<ListEntrySnapshot> = entries
            .into_iter()
            .map(|e| ListEntrySnapshot {
                cid: e.cid,
                status: e.status,
                progress: e.progress,
                score: e.score,
                start_date: e.start_date,
                end_date: e.end_date,
                repeat_count: e.repeat_count,
                notes: e.notes,
                is_private: e.is_private,
            })
            .collect();

        let entry_count = snapshots.len() as i32;
        let json = serde_json::to_string_pretty(&snapshots)
            .map_err(|e| CoreError::Internal(e.to_string()))?;

        let now = chrono::Utc::now().timestamp();

        // 2. Resolver la ruta del fichero
        let file_path = match trigger {
            BackupTrigger::PreImport => {
                let name = tracker_name.ok_or_else(|| {
                    CoreError::Internal("tracker_name required for PRE_IMPORT backup".into())
                })?;
                paths.pre_import_backup_path(user_id, name)
            }
            BackupTrigger::Manual => paths.manual_backup_path(user_id, now),
            BackupTrigger::RemoteSync => return Err(CoreError::Internal("RemoteSync use save_remote_list instead".into())),
        };

        // 3. Asegurar que existe el directorio del usuario
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| CoreError::Internal(format!("Could not create backup dir: {}", e)))?;
        }

        // 4. Escribir el fichero (sobreescribe si existe)
        std::fs::write(&file_path, &json)
            .map_err(|e| CoreError::Internal(format!("Could not write backup file: {}", e)))?;

        let relative_path = paths.relative_backup_path(&file_path);

        // 5. Upsert en DB
        let backup_id = match trigger {
            BackupTrigger::PreImport => {
                // Buscar si ya existe un registro para este tracker
                let existing_id: Option<i64> = conn.query_row(
                    "SELECT id FROM ListBackup
                     WHERE user_id = ?1 AND trigger = 'PRE_IMPORT' AND tracker_name = ?2",
                    params![user_id, tracker_name],
                    |row| row.get(0),
                )
                    .optional()?;

                if let Some(id) = existing_id {
                    conn.execute(
                        "UPDATE ListBackup
                         SET file_path = ?1, entry_count = ?2, created_at = ?3
                         WHERE id = ?4",
                        params![relative_path, entry_count, now, id],
                    )?;
                    id
                } else {
                    conn.execute(
                        "INSERT INTO ListBackup (user_id, trigger, tracker_name, file_path, entry_count, created_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                        params![user_id, BackupTrigger::PreImport.as_str(), tracker_name, relative_path, entry_count, now],
                    )?;
                    conn.last_insert_rowid()
                }
            }
            BackupTrigger::Manual => {
                conn.execute(
                    "INSERT INTO ListBackup (user_id, trigger, tracker_name, file_path, entry_count, created_at)
                     VALUES (?1, ?2, NULL, ?3, ?4, ?5)",
                    params![user_id, BackupTrigger::Manual.as_str(), relative_path, entry_count, now],
                )?;
                conn.last_insert_rowid()
            }
            BackupTrigger::RemoteSync => unreachable!(),
        };

        Ok(backup_id)
    }


    pub fn list_backups(conn: &Connection, user_id: i32) -> CoreResult<Vec<ListBackupMeta>> {
        let mut stmt = conn.prepare(
            "SELECT id, user_id, trigger, tracker_name, file_path, entry_count, created_at
             FROM ListBackup
             WHERE user_id = ?1
             ORDER BY created_at DESC",
        )?;

        let rows = stmt.query_map(params![user_id], |row| {
            Ok(ListBackupMeta {
                id: row.get(0)?,
                user_id: row.get(1)?,
                trigger: row.get(2)?,
                tracker_name: row.get(3)?,
                file_path: row.get(4)?,
                entry_count: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?;

        let mut result = Vec::new();
        for r in rows {
            result.push(r?);
        }
        Ok(result)
    }

    pub fn get_backup_meta(
        conn: &Connection,
        user_id: i32,
        backup_id: i64,
    ) -> CoreResult<Option<ListBackupMeta>> {
        conn.query_row(
            "SELECT id, user_id, trigger, tracker_name, file_path, entry_count, created_at
             FROM ListBackup
             WHERE id = ?1 AND user_id = ?2",
            params![backup_id, user_id],
            |row| {
                Ok(ListBackupMeta {
                    id: row.get(0)?,
                    user_id: row.get(1)?,
                    trigger: row.get(2)?,
                    tracker_name: row.get(3)?,
                    file_path: row.get(4)?,
                    entry_count: row.get(5)?,
                    created_at: row.get(6)?,
                })
            },
        )
            .optional()
            .map_err(Into::into)
    }

    /// Lee el fichero JSON y devuelve las entradas del snapshot.
    pub fn read_snapshot(
        paths: &AppPaths,
        meta: &ListBackupMeta,
    ) -> CoreResult<Vec<ListEntrySnapshot>> {
        let full_path = paths.base_dir.join(&meta.file_path);
        let json = std::fs::read_to_string(&full_path)
            .map_err(|e| CoreError::Internal(format!("Could not read backup file: {}", e)))?;
        let entries: Vec<ListEntrySnapshot> = serde_json::from_str(&json)
            .map_err(|e| CoreError::Internal(format!("Could not parse backup file: {}", e)))?;
        Ok(entries)
    }

    pub fn delete_backup(
        conn: &Connection,
        paths: &AppPaths,
        user_id: i32,
        backup_id: i64,
    ) -> CoreResult<bool> {
        let meta = Self::get_backup_meta(conn, user_id, backup_id)?;
        let Some(meta) = meta else { return Ok(false) };

        let full_path = paths.base_dir.join(&meta.file_path);
        if full_path.exists() {
            std::fs::remove_file(&full_path)
                .map_err(|e| CoreError::Internal(format!("Could not delete backup file: {}", e)))?;
        }

        let count = conn.execute(
            "DELETE FROM ListBackup WHERE id = ?1 AND user_id = ?2",
            params![backup_id, user_id],
        )?;

        Ok(count > 0)
    }
}