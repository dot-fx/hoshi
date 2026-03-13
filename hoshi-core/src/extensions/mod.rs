mod sandbox;

use crate::error::{CoreError, CoreResult};
use crate::headless::{HeadlessHandle, noop_headless};
use crate::paths::AppPaths;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

const BASE: &str  = include_str!("base/Base.js");
const ANIME: &str = include_str!("base/Anime.js");
const MANGA: &str = include_str!("base/Manga.js");
const NOVEL: &str = include_str!("base/Novel.js");
const BOORU: &str = include_str!("base/Booru.js");
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

// ─── Extension ───────────────────────────────────────────────────────────────

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
    /// The definitions declared in the manifest (key / type / label / default).
    pub setting_definitions: Vec<SettingDefinition>,
    /// The current values, merging manifest defaults with any user overrides
    /// persisted in `settings.json`. Always populated after load/install.
    pub settings: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ExtensionType {
    Anime,
    Manga,
    Novel,
    Booru,
    #[serde(other)]
    Unknown,
}

// ─── Manager ─────────────────────────────────────────────────────────────────

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

    pub async fn load_extensions(&mut self) -> CoreResult<()> {
        if !self.extensions_dir.exists() {
            fs::create_dir_all(&self.extensions_dir).await?;
        }

        self.extensions.clear();
        let mut entries = fs::read_dir(&self.extensions_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let manifest_path = path.join("manifest.yaml");
            if !manifest_path.exists() {
                continue;
            }

            let yaml_content = match fs::read_to_string(&manifest_path).await {
                Ok(c) => c,
                Err(_) => {
                    tracing::warn!("Could not read manifest at {:?}", manifest_path);
                    continue;
                }
            };

            let manifest: ExtensionManifest = match serde_yaml::from_str(&yaml_content) {
                Ok(m) => m,
                Err(e) => {
                    tracing::error!("Invalid YAML in {:?}: {}", manifest_path, e);
                    continue;
                }
            };

            let script_path = path.join(&manifest.main);
            if !script_path.exists() {
                tracing::error!("Main file not found: {:?}", script_path);
                continue;
            }

            match script_path.extension().and_then(|e| e.to_str()) {
                Some("js") => {}
                _ => {
                    tracing::warn!("Only .js are supported: {:?}", script_path);
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
        }

        tracing::info!("Loaded {} extensions", self.extensions.len());
        Ok(())
    }

    pub async fn install_extension(&mut self, manifest_url: &str) -> CoreResult<Extension> {
        let response = reqwest::get(manifest_url)
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(CoreError::Network(format!(
                "Failed to fetch manifest (HTTP {}): {}",
                response.status(),
                manifest_url
            )));
        }

        let manifest_bytes = response
            .bytes()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        let manifest: ExtensionManifest = serde_yaml::from_slice(&manifest_bytes)
            .map_err(|e| CoreError::Parse(format!("Invalid manifest YAML: {}", e)))?;

        if manifest.ext_type == ExtensionType::Unknown {
            return Err(CoreError::Validation(
                "Extension type is unknown or unsupported".into(),
            ));
        }

        if !manifest.main.ends_with(".js") {
            return Err(CoreError::Validation(
                "Only .js scripts are supported".into(),
            ));
        }

        let script_url =
            if manifest.main.starts_with("http://") || manifest.main.starts_with("https://") {
                manifest.main.clone()
            } else {
                let base = manifest_url
                    .rsplit_once('/')
                    .map(|(b, _)| b)
                    .unwrap_or(manifest_url);
                format!("{}/{}", base, manifest.main)
            };

        let script_response = reqwest::get(&script_url)
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        if !script_response.status().is_success() {
            return Err(CoreError::Network(format!(
                "Failed to fetch script (HTTP {}): {}",
                script_response.status(),
                script_url
            )));
        }

        let script_bytes = script_response
            .bytes()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        let ext_dir = self.extensions_dir.join(&manifest.id);
        fs::create_dir_all(&ext_dir).await?;

        fs::write(ext_dir.join("manifest.yaml"), &manifest_bytes).await?;

        let script_filename = manifest
            .main
            .rsplit('/')
            .next()
            .unwrap_or("index.js");
        let script_path = ext_dir.join(script_filename);
        fs::write(&script_path, &script_bytes).await?;

        // On fresh install, settings.json is written with all defaults so the
        // file exists and is ready for the user to edit.
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
        tracing::info!("Installed extension '{}'", extension.id);
        Ok(extension)
    }

    pub async fn uninstall_extension(&mut self, id: &str) -> CoreResult<()> {
        if !self.extensions.contains_key(id) {
            return Err(CoreError::NotFound(format!(
                "Extension not found: {}",
                id
            )));
        }

        let ext_dir = self.extensions_dir.join(id);
        if ext_dir.exists() {
            fs::remove_dir_all(&ext_dir).await?;
        }

        self.extensions.remove(id);
        tracing::info!("Uninstalled extension '{}'", id);
        Ok(())
    }

    /// Persist updated settings for an extension. Merges the provided values
    /// over the existing ones, then writes `settings.json` to disk and updates
    /// the in-memory extension entry.
    pub async fn update_extension_settings(
        &mut self,
        id: &str,
        updates: HashMap<String, Value>,
    ) -> CoreResult<()> {
        let extension = self
            .extensions
            .get_mut(id)
            .ok_or_else(|| CoreError::NotFound(format!("Extension not found: {}", id)))?;

        for (key, value) in updates {
            extension.settings.insert(key, value);
        }

        let ext_dir = self.extensions_dir.join(id);
        persist_settings(&ext_dir, &extension.settings).await;
        Ok(())
    }

    pub fn set_headless(&mut self, headless: HeadlessHandle) {
        self.headless = headless;
    }

    pub async fn call_extension_function(
        &self,
        extension_id: &str,
        function_name: &str,
        args: Vec<Value>,
    ) -> CoreResult<Value> {
        let extension = self
            .extensions
            .get(extension_id)
            .ok_or_else(|| CoreError::NotFound(format!("Extension ID not found: {}", extension_id)))?;

        if !extension.script_path.exists() {
            return Err(CoreError::NotFound(format!(
                "Extension source file missing: {:?}",
                extension.script_path
            )));
        }

        let extension_code = fs::read_to_string(&extension.script_path).await?;
        sandbox::execute_in_quickjs(
            extension_code,
            function_name.to_string(),
            args,
            self.headless.clone(),
            extension.settings.clone(),
        )
            .await
    }

    pub fn list_extensions(&self) -> Vec<&Extension> {
        self.extensions.values().collect()
    }

    pub fn get_extensions_by_type(&self, target_type: ExtensionType) -> Vec<String> {
        self.extensions
            .values()
            .filter(|e| e.ext_type == target_type)
            .map(|e| e.id.clone())
            .collect()
    }
}

// ─── Settings helpers ─────────────────────────────────────────────────────────

/// Builds the effective settings map for an extension directory.
///
/// Strategy:
/// 1. Start with the defaults declared in the manifest.
/// 2. Overlay any values already persisted in `settings.json` (user overrides).
///
/// Keys present in the JSON file that are *not* in the manifest definitions
/// are silently ignored, so stale keys never accumulate.
async fn load_settings(
    ext_dir: &PathBuf,
    definitions: &[SettingDefinition],
) -> HashMap<String, Value> {
    // Start from manifest defaults.
    let mut settings: HashMap<String, Value> = definitions
        .iter()
        .map(|d| (d.key.clone(), d.default.clone()))
        .collect();

    // Overlay persisted user overrides if the file exists.
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

/// Writes the current settings map to `settings.json` inside the extension
/// directory. Errors are logged but not propagated — a failed settings write
/// should never crash an install or update.
async fn persist_settings(ext_dir: &PathBuf, settings: &HashMap<String, Value>) {
    let path = ext_dir.join("settings.json");
    match serde_json::to_string_pretty(settings) {
        Ok(json) => {
            if let Err(e) = fs::write(&path, json).await {
                tracing::warn!("Could not write settings.json to {:?}: {}", path, e);
            }
        }
        Err(e) => tracing::warn!("Could not serialise settings for {:?}: {}", path, e),
    }
}