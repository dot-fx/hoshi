pub mod service;

use crate::error::{CoreError, CoreResult};
use crate::paths::{ensure_dir, get_base_path, get_config_path};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryPaths {
    pub anime: Option<String>,
    pub manga: Option<String>,
    pub novels: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryPaths {
    pub mpv: Option<String>,
    pub ffmpeg: Option<String>,
    pub ffprobe: Option<String>,
    pub cloudflared: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub setup_completed: bool,
    pub library: LibraryPaths,
    pub paths: BinaryPaths,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            setup_completed: false,
            library: LibraryPaths {
                anime: None,
                manga: None,
                novels: None,
            },
            paths: BinaryPaths {
                mpv: None,
                ffmpeg: None,
                ffprobe: None,
                cloudflared: None,
            },
        }
    }
}

pub fn ensure_config_file() -> CoreResult<()> {
    let base_path = get_base_path();
    ensure_dir(&base_path)?;

    let config_path = get_config_path();

    if !config_path.exists() {
        let default_config = Config::default();
        save_config(&default_config)?;
        tracing::info!("Created default config at: {}", config_path.display());
    }

    Ok(())
}

pub fn load_config() -> CoreResult<Config> {
    let config_path = get_config_path();

    if !config_path.exists() {
        return Ok(Config::default());
    }

    let contents = fs::read_to_string(&config_path)?;
    let config: Config = serde_yaml::from_str(&contents)
        .map_err(|e| CoreError::Config(format!("Failed to parse config: {}", e)))?;

    Ok(config)
}

pub fn save_config(config: &Config) -> CoreResult<()> {
    let config_path = get_config_path();
    let yaml = serde_yaml::to_string(config)
        .map_err(|e| CoreError::Config(format!("Failed to serialize config: {}", e)))?;

    fs::write(&config_path, yaml)?;
    tracing::info!("Config saved to: {}", config_path.display());
    Ok(())
}