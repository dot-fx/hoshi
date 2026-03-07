mod sandbox;

use crate::error::{CoreError, CoreResult};
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

pub struct ExtensionManager {
    extensions: HashMap<String, Extension>,
    extensions_dir: PathBuf,
}

impl ExtensionManager {
    pub fn new(paths: &AppPaths) -> CoreResult<Self> {
        let extensions_dir = paths.base_dir.join("extensions");
        Ok(Self {
            extensions: HashMap::new(),
            extensions_dir,
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

            let extension = Extension {
                id: manifest.id.clone(),
                name: manifest.name,
                version: manifest.version,
                author: manifest.author.unwrap_or_else(|| "Unknown".to_string()),
                icon: manifest.icon,
                ext_type: manifest.ext_type,
                script_path,
                language: manifest.language,
            };

            self.extensions.insert(manifest.id, extension);
        }

        tracing::info!("Loaded {} extensions", self.extensions.len());
        Ok(())
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
        sandbox::execute_in_quickjs(extension_code, function_name.to_string(), args).await
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