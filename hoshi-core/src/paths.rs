use std::path::PathBuf;
use crate::error::CoreResult;

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub base_dir: PathBuf,
    pub config_path: PathBuf,
    pub database_path: PathBuf,
    pub images_path: PathBuf,
    pub backups_path: PathBuf,
}

impl AppPaths {
    pub fn from_base(base: PathBuf) -> Self {
        Self {
            config_path: base.join("config.yaml"),
            database_path: base.join("app.db"),
            images_path: base.join("images"),
            backups_path: base.join("backups"),
            base_dir: base,
        }
    }

    pub fn ensure_dirs(&self) -> CoreResult<()> {
        ensure_dir(&self.base_dir)?;
        ensure_dir(&self.images_path)?;
        ensure_dir(&self.backups_path)?;
        Ok(())
    }

    pub fn user_backups_dir(&self, user_id: i32) -> PathBuf {
        self.backups_path.join(user_id.to_string())
    }

    pub fn pre_import_backup_path(&self, user_id: i32, tracker_name: &str) -> PathBuf {
        self.user_backups_dir(user_id)
            .join(format!("pre_import_{}.json", tracker_name))
    }

    pub fn manual_backup_path(&self, user_id: i32, timestamp: i64) -> PathBuf {
        self.user_backups_dir(user_id)
            .join(format!("manual_{}.json", timestamp))
    }

    pub fn relative_backup_path(&self, full_path: &PathBuf) -> String {
        full_path
            .strip_prefix(&self.base_dir)
            .unwrap_or(full_path)
            .to_string_lossy()
            .to_string()
    }
}

fn ensure_dir(path: &PathBuf) -> CoreResult<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}