use hoshi_core::error::CoreError;
use hoshi_core::logs::LogEntry;

#[tauri::command]
pub async fn get_system_logs(
    state: tauri::State<'_, std::sync::Arc<hoshi_core::state::AppState>>,
) -> Result<Vec<LogEntry>, CoreError> {
    let logs = {
        let lock = state.inner().log_store.read().unwrap();
        lock.iter().cloned().collect()
    };
    Ok(logs)
}