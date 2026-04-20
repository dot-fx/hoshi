use tauri::AppHandle;
use tauri::Manager;
use tauri_plugin_fs::FsExt;

#[tauri::command]
pub async fn load_locale(app: AppHandle, lang: String) -> Result<serde_json::Value, String> {
    if !lang.chars().all(|c| c.is_alphanumeric() || c == '-') {
        return Err(format!("Invalid language code: {}", lang));
    }

    let resource_path = app
        .path()
        .resolve(
            format!("locales/{}.json", lang),
            tauri::path::BaseDirectory::Resource,
        )
        .map_err(|e| e.to_string())?;

    let content = app
        .fs()
        .read_to_string(&resource_path)
        .map_err(|e| format!("Locale '{}' not found: {}", lang, e))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("Invalid JSON in locale '{}': {}", lang, e))
}