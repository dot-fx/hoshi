use hoshi_core::error::CoreError;
use hoshi_core::logs::LogEntry;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize)]
pub struct LogFileInfo {
    pub name: String,
    pub size_bytes: u64,
    pub created_at: i64,
}

fn logs_dir(state: &tauri::State<'_, std::sync::Arc<hoshi_core::state::AppState>>) -> PathBuf {
    state.paths.logs_path.clone()
}

#[tauri::command]
pub async fn get_system_logs(
    state: tauri::State<'_, std::sync::Arc<hoshi_core::state::AppState>>,
) -> Result<Vec<LogEntry>, CoreError> {
    let logs = {
        let lock = state.log_store.read().unwrap();
        lock.iter().cloned().collect()
    };
    Ok(logs)
}

#[tauri::command]
pub async fn list_log_files(
    state: tauri::State<'_, std::sync::Arc<hoshi_core::state::AppState>>,
) -> Result<Vec<LogFileInfo>, CoreError> {
    let dir = logs_dir(&state);
    let mut files = vec![];

    let entries = std::fs::read_dir(&dir).map_err(CoreError::Io)?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("log") {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let meta = std::fs::metadata(&path).map_err(CoreError::Io)?;
        let size_bytes = meta.len();

        let created_at = {
            chrono::NaiveDateTime::parse_from_str(
                name.trim_end_matches(".log"),
                "%Y-%m-%dT%H-%M-%S"
            )
                .map(|dt| dt.and_utc().timestamp_millis())
                .unwrap_or(0)
        };

        files.push(LogFileInfo { name, size_bytes, created_at });
    }

    files.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(files)
}

#[tauri::command]
pub async fn get_log_file(
    state: tauri::State<'_, std::sync::Arc<hoshi_core::state::AppState>>,
    name: String,
) -> Result<String, CoreError> {
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err(CoreError::NotFound("Invalid log file name".into()));
    }
    let path = logs_dir(&state).join(&name);
    std::fs::read_to_string(&path).map_err(CoreError::Io)
}

#[tauri::command]
pub async fn delete_log_file(
    state: tauri::State<'_, std::sync::Arc<hoshi_core::state::AppState>>,
    name: String,
) -> Result<(), CoreError> {
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err(CoreError::NotFound("Invalid log file name".into()));
    }
    let path = logs_dir(&state).join(&name);
    std::fs::remove_file(&path).map_err(CoreError::Io)
}