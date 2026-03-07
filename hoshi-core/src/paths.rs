use std::path::PathBuf;
use crate::error::CoreResult;

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub base_dir: PathBuf,
    pub config_path: PathBuf,
    pub database_path: PathBuf,
    pub images_path: PathBuf,
}

impl AppPaths {
    pub fn from_base(base: PathBuf) -> Self {
        Self {
            config_path: base.join("config.yaml"),
            database_path: base.join("app.db"),
            images_path: base.join("images"),
            base_dir: base,
        }
    }

    pub fn ensure_dirs(&self) -> CoreResult<()> {
        ensure_dir(&self.base_dir)?;
        ensure_dir(&self.images_path)?;
        Ok(())
    }
}

fn ensure_dir(path: &PathBuf) -> CoreResult<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}