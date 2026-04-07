use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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