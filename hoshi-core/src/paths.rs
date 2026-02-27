use crate::error::CoreResult;
use std::path::PathBuf;

pub fn get_base_path() -> PathBuf {
    if cfg!(target_os = "windows") {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("hoshi")
    } else if cfg!(target_os = "macos") {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Library")
            .join("Application Support")
            .join("hoshi")
    } else {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("hoshi")
    }
}

pub fn get_config_path() -> PathBuf {
    get_base_path().join("config.yaml")
}
pub fn get_database_path() -> PathBuf {
    get_base_path().join("app.db")
}

pub fn get_images_path() -> PathBuf {
    get_base_path().join("images")
}

pub fn ensure_dir(path: &PathBuf) -> CoreResult<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}