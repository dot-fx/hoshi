use crate::config;
use crate::error::{CoreError, CoreResult};
use serde_json::Value;

pub struct ConfigService;

impl ConfigService {

    pub fn get_public_config_with_schema() -> CoreResult<Value> {
        let cfg = config::load_config()?;
        let schema = Self::get_config_schema();

        let public_values = serde_json::json!({
            "library": cfg.library,
            "paths": cfg.paths,
        });

        Ok(serde_json::json!({
            "values": public_values,
            "schema": schema
        }))
    }

    pub fn get_config_section(section: &str) -> CoreResult<Value> {
        let cfg = config::load_config()?;

        let section_data = match section {
            "library" => serde_json::to_value(&cfg.library)?,
            "paths" => serde_json::to_value(&cfg.paths)?,
            _ => return Err(CoreError::NotFound(format!("Section '{}' not found", section))),
        };

        Ok(serde_json::json!({
            section: section_data
        }))
    }

    pub fn update_config_partial(updates: Value) -> CoreResult<Value> {
        let mut cfg = config::load_config()?;

        if let Some(lib_updates) = updates.get("library") {
            if let Some(val) = lib_updates.get("anime") {
                cfg.library.anime = Self::json_to_opt_string(val);
            }
            if let Some(val) = lib_updates.get("manga") {
                cfg.library.manga = Self::json_to_opt_string(val);
            }
            if let Some(val) = lib_updates.get("novels") {
                cfg.library.novels = Self::json_to_opt_string(val);
            }
        }

        if let Some(path_updates) = updates.get("paths") {
            if let Some(val) = path_updates.get("mpv") {
                cfg.paths.mpv = Self::json_to_opt_string(val);
            }
            if let Some(val) = path_updates.get("ffmpeg") {
                cfg.paths.ffmpeg = Self::json_to_opt_string(val);
            }
            if let Some(val) = path_updates.get("ffprobe") {
                cfg.paths.ffprobe = Self::json_to_opt_string(val);
            }
            if let Some(val) = path_updates.get("cloudflared") {
                cfg.paths.cloudflared = Self::json_to_opt_string(val);
            }
        }

        config::save_config(&cfg)?;

        Ok(serde_json::json!({
            "library": cfg.library,
            "paths": cfg.paths,
        }))
    }

    pub fn update_config_section(section: &str, updates: Value) -> CoreResult<Value> {
        let wrapper = serde_json::json!({
            section: updates
        });

        let full_updated = Self::update_config_partial(wrapper)?;

        match full_updated.get(section) {
            Some(section_data) => Ok(serde_json::json!({
                section: section_data
            })),
            None => Err(CoreError::Internal(
                "Failed to retrieve updated section".into(),
            )),
        }
    }

    fn get_config_schema() -> Value {
        serde_json::json!({
            "library": {
                "anime": { "description": "Path where anime is stored" },
                "manga": { "description": "Path where manga is stored" },
                "novels": { "description": "Path where novels are stored" }
            },
            "paths": {
                "mpv": { "description": "Required to open anime episodes in mpv on desktop version." },
                "ffmpeg": { "description": "Required for downloading anime episodes." },
                "ffprobe": { "description": "Required for watching local anime episodes." },
                "cloudflared": { "description": "Required for creating pubic rooms." }
            }
        })
    }

    fn json_to_opt_string(val: &Value) -> Option<String> {
        if val.is_null() {
            None
        } else {
            val.as_str().map(|s| s.to_string())
        }
    }
}