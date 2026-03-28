mod sandbox;

use crate::error::{CoreError, CoreResult};
use crate::headless::{noop_headless, HeadlessHandle};
use crate::paths::AppPaths;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tracing::{info, warn, error, debug, instrument};

const BASE: &str  = include_str!("base/Base.js");
const ANIME: &str = include_str!("base/Anime.js");
const MANGA: &str = include_str!("base/Manga.js");
const NOVEL: &str = include_str!("base/Novel.js");
const SANDBOX_BOOTSTRAP: &str = include_str!("sandbox_bootstrap.js");


#[derive(Debug, Deserialize)]
pub struct ExtensionManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    #[serde(rename = "type")]
    pub ext_type: ExtensionType,
    pub main: String,
    pub icon: Option<String>,
    pub language: String,
    #[serde(default)]
    pub nsfw: bool,
    #[serde(default)]
    pub skip_default_processing: bool,
    #[serde(default)]
    pub settings: Vec<SettingDefinition>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingDefinition {
    pub key: String,
    pub label: String,
    #[serde(rename = "type")]
    pub setting_type: SettingType,
    pub default: Value,
    #[serde(default)]
    pub options: Vec<SettingOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingOption {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SettingType {
    String,
    Number,
    Boolean,
    Select,
    MultiSelect,
    #[serde(other)]
    Unknown,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extension {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub icon: Option<String>,
    pub ext_type: ExtensionType,
    #[serde(skip)]
    pub script_path: PathBuf,
    pub language: String,
    pub nsfw: bool,
    pub skip_default_processing: bool,
    pub setting_definitions: Vec<SettingDefinition>,
    pub settings: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ExtensionType {
    Anime,
    Manga,
    Novel,
    #[serde(other)]
    Unknown,
}

pub struct ExtensionManager {
    extensions: HashMap<String, Extension>,
    extensions_dir: PathBuf,
    headless: HeadlessHandle,
}

impl ExtensionManager {
    pub fn new(paths: &AppPaths) -> CoreResult<Self> {
        let extensions_dir = paths.base_dir.join("extensions");
        Ok(Self {
            extensions: HashMap::new(),
            extensions_dir,
            headless: noop_headless(),
        })
    }

    #[instrument(skip(self))]
    pub async fn load_extensions(&mut self) -> CoreResult<()> {
        if !self.extensions_dir.exists() {
            debug!(path = %self.extensions_dir.display(), "Extensions directory missing, creating it");
            fs::create_dir_all(&self.extensions_dir).await.map_err(CoreError::Io)?;
        }

        self.extensions.clear();

        let mut entries = fs::read_dir(&self.extensions_dir).await.map_err(CoreError::Io)?;
        let mut loaded_count = 0;

        while let Some(entry) = entries.next_entry().await.map_err(CoreError::Io)? {
            let path = entry.path();
            if !path.is_dir() { continue; }

            let manifest_path = path.join("manifest.yaml");
            if !manifest_path.exists() { continue; }

            let yaml_content = match fs::read_to_string(&manifest_path).await {
                Ok(c) => c,
                Err(e) => {
                    warn!(path = %manifest_path.display(), error = ?e, "Could not read manifest file");
                    continue;
                }
            };

            let manifest: ExtensionManifest = match serde_yaml::from_str(&yaml_content) {
                Ok(m) => m,
                Err(e) => {
                    error!(path = %manifest_path.display(), error = ?e, "Invalid YAML format in manifest");
                    continue;
                }
            };

            let script_path = path.join(&manifest.main);
            if !script_path.exists() {
                error!(ext = %manifest.id, expected_path = %script_path.display(), "Main JS file declared in manifest is missing");
                continue;
            }

            match script_path.extension().and_then(|e| e.to_str()) {
                Some("js") => {}
                _ => {
                    warn!(ext = %manifest.id, script = %script_path.display(), "Only .js extension scripts are supported");
                    continue;
                }
            }

            let settings = load_settings(&path, &manifest.settings).await;

            let extension = Extension {
                id: manifest.id.clone(),
                name: manifest.name,
                version: manifest.version,
                author: manifest.author.unwrap_or_else(|| "Unknown".to_string()),
                icon: manifest.icon,
                ext_type: manifest.ext_type,
                script_path,
                language: manifest.language,
                nsfw: manifest.nsfw,
                skip_default_processing: manifest.skip_default_processing,
                setting_definitions: manifest.settings,
                settings,
            };

            self.extensions.insert(manifest.id, extension);
            loaded_count += 1;
        }

        info!(count = loaded_count, "Extensions loaded from disk successfully");
        Ok(())
    }

    #[instrument(skip(self, manifest_url))]
    pub async fn install_extension(&mut self, manifest_url: &str) -> CoreResult<Extension> {
        info!(url = %manifest_url, "Starting extension installation");

        let response = reqwest::get(manifest_url).await
            .map_err(|e| {
                error!(error = ?e, "Failed to connect to manifest URL");
                CoreError::Network("error.extension.install_network_failed".into())
            })?;

        if !response.status().is_success() {
            error!(status = %response.status(), url = %manifest_url, "Manifest server returned HTTP error");
            return Err(CoreError::Network("error.extension.install_network_failed".into()));
        }

        let manifest_bytes = response.bytes().await
            .map_err(|_e| CoreError::Network("error.extension.install_network_failed".into()))?;

        let manifest: ExtensionManifest = serde_yaml::from_slice(&manifest_bytes)
            .map_err(|e| {
                error!(error = ?e, "Downloaded manifest contains invalid YAML");
                CoreError::Parse("error.extension.invalid_manifest".into())
            })?;

        if manifest.ext_type == ExtensionType::Unknown {
            warn!(ext = %manifest.id, "Extension rejected: Unsupported type declared");
            return Err(CoreError::Validation("error.extension.unsupported_type".into()));
        }

        if !manifest.main.ends_with(".js") {
            warn!(ext = %manifest.id, "Extension rejected: Main script is not .js");
            return Err(CoreError::Validation("error.extension.invalid_script".into()));
        }

        let script_url = if manifest.main.starts_with("http://") || manifest.main.starts_with("https://") {
            manifest.main.clone()
        } else {
            let base = manifest_url.rsplit_once('/').map(|(b, _)| b).unwrap_or(manifest_url);
            format!("{}/{}", base, manifest.main)
        };

        debug!(ext = %manifest.id, url = %script_url, "Downloading extension script");
        let script_response = reqwest::get(&script_url).await
            .map_err(|e| {
                error!(error = ?e, "Failed to connect to script URL");
                CoreError::Network("error.extension.install_network_failed".into())
            })?;

        if !script_response.status().is_success() {
            error!(status = %script_response.status(), url = %script_url, "Script server returned HTTP error");
            return Err(CoreError::Network("error.extension.install_network_failed".into()));
        }

        let script_bytes = script_response.bytes().await
            .map_err(|_| CoreError::Network("error.extension.install_network_failed".into()))?;

        let ext_dir = self.extensions_dir.join(&manifest.id);

        fs::create_dir_all(&ext_dir).await.map_err(|e| {
            error!(error = ?e, path = %ext_dir.display(), "Failed to create extension directory");
            CoreError::Io(e)
        })?;

        fs::write(ext_dir.join("manifest.yaml"), &manifest_bytes).await.map_err(CoreError::Io)?;

        let script_filename = manifest.main.rsplit('/').next().unwrap_or("index.js");
        let script_path = ext_dir.join(script_filename);
        fs::write(&script_path, &script_bytes).await.map_err(CoreError::Io)?;

        let settings = load_settings(&ext_dir, &manifest.settings).await;
        persist_settings(&ext_dir, &settings).await;

        let extension = Extension {
            id: manifest.id.clone(),
            name: manifest.name,
            version: manifest.version,
            author: manifest.author.unwrap_or_else(|| "Unknown".to_string()),
            icon: manifest.icon,
            ext_type: manifest.ext_type,
            script_path,
            language: manifest.language,
            nsfw: manifest.nsfw,
            skip_default_processing: manifest.skip_default_processing,
            setting_definitions: manifest.settings,
            settings,
        };

        self.extensions.insert(manifest.id.clone(), extension.clone());
        info!(ext = %extension.id, "Extension installed and loaded successfully");

        Ok(extension)
    }

    #[instrument(skip(self))]
    pub async fn uninstall_extension(&mut self, id: &str) -> CoreResult<()> {
        if !self.extensions.contains_key(id) {
            warn!(ext = %id, "Attempted to uninstall a non-existent extension");
            return Err(CoreError::NotFound("error.extension.not_found".into()));
        }

        let ext_dir = self.extensions_dir.join(id);
        if ext_dir.exists() {
            debug!(path = %ext_dir.display(), "Removing extension directory from disk");
            fs::remove_dir_all(&ext_dir).await.map_err(|e| {
                error!(error = ?e, "Failed to delete extension directory");
                CoreError::Io(e)
            })?;
        }

        self.extensions.remove(id);
        info!(ext = %id, "Extension uninstalled successfully");
        Ok(())
    }

    #[instrument(skip(self, updates))]
    pub async fn update_extension_settings(
        &mut self,
        id: &str,
        updates: HashMap<String, Value>,
    ) -> CoreResult<()> {
        let extension = self.extensions.get_mut(id).ok_or_else(|| {
            warn!(ext = %id, "Attempted to update settings for a non-existent extension");
            CoreError::NotFound("error.extension.not_found".into())
        })?;

        for (key, value) in updates {
            extension.settings.insert(key, value);
        }

        let ext_dir = self.extensions_dir.join(id);
        persist_settings(&ext_dir, &extension.settings).await;

        debug!(ext = %id, "Extension settings updated successfully");
        Ok(())
    }

    pub fn set_headless(&mut self, headless: HeadlessHandle) {
        self.headless = headless;
    }

    #[instrument(skip(self, args))]
    pub async fn call_extension_function(
        &self,
        extension_id: &str,
        function_name: &str,
        args: Vec<Value>,
    ) -> CoreResult<Value> {
        let extension = self.extensions.get(extension_id).ok_or_else(|| {
            error!(ext = %extension_id, func = %function_name, "Attempted to call function on unloaded extension");
            CoreError::NotFound("error.extension.not_found".into())
        })?;

        if !extension.script_path.exists() {
            error!(ext = %extension_id, path = %extension.script_path.display(), "Extension script file missing from disk");
            return Err(CoreError::NotFound("error.extension.script_missing".into()));
        }

        debug!(ext = %extension_id, func = %function_name, "Reading script and executing sandbox");
        let extension_code = fs::read_to_string(&extension.script_path).await.map_err(CoreError::Io)?;

        sandbox::execute_in_quickjs(
            extension_code,
            function_name.to_string(),
            args,
            self.headless.clone(),
            extension.settings.clone(),
        ).await
    }

    pub fn list_extensions(&self) -> Vec<&Extension> {
        self.extensions.values().collect()
    }

    pub fn get_extensions_by_type(&self, target_type: ExtensionType) -> Vec<String> {
        self.extensions.values()
            .filter(|e| e.ext_type == target_type)
            .map(|e| e.id.clone())
            .collect()
    }
}

async fn load_settings(
    ext_dir: &PathBuf,
    definitions: &[SettingDefinition],
) -> HashMap<String, Value> {
    let mut settings: HashMap<String, Value> = definitions
        .iter()
        .map(|d| (d.key.clone(), d.default.clone()))
        .collect();

    let settings_path = ext_dir.join("settings.json");
    if settings_path.exists() {
        if let Ok(raw) = fs::read_to_string(&settings_path).await {
            if let Ok(Value::Object(map)) = serde_json::from_str::<Value>(&raw) {
                for def in definitions {
                    if let Some(user_value) = map.get(&def.key) {
                        settings.insert(def.key.clone(), user_value.clone());
                    }
                }
            }
        }
    }

    settings
}

async fn persist_settings(ext_dir: &PathBuf, settings: &HashMap<String, Value>) {
    let path = ext_dir.join("settings.json");
    match serde_json::to_string_pretty(settings) {
        Ok(json) => {
            if let Err(e) = fs::write(&path, json).await {
                warn!("Could not write settings.json to {:?}: {}", path, e);
            }
        }
        Err(e) => warn!("Could not serialise settings for {:?}: {}", path, e),
    }
}