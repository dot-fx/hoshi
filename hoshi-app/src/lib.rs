use tauri::{Manager, async_runtime, Listener, Emitter};
use tokio::sync::RwLock;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use hoshi_core::error::CoreError;
use tracing::{info, error, debug};
use hoshi_core::logs::{new_log_store, MemoryLogLayer, LogEntry};

pub mod commands;
pub mod headless;
pub mod headless_sync;

#[cfg(mobile)]
mod headless_plugin;
#[cfg(mobile)]
use headless_plugin::{init as headless_plugin_init, notify_done};

use crate::commands::auth::{login, register, logout, get_current_profile};
use crate::commands::users::{get_all_users, get_user, get_me, update_me, delete_me, change_password, upload_avatar, delete_avatar};
use crate::commands::content::{get_trending, get_home_content, get_content, update_content, search_content, get_content_items, play_content_by_number, add_tracker_mapping, add_extension_source, update_extension_mapping, update_tracker_mapping, delete_tracker_mapping, resolve_by_tracker, resolve_by_extension, link_tracker, resolve_extension_item, search_extension_direct};
use crate::commands::schedule::{get_schedule};
use crate::commands::list::{get_list, get_single_entry, upsert_entry, delete_entry, get_stats};
use crate::commands::proxy::{proxy_fetch_text, proxy_fetch_bytes};
use crate::commands::extensions::{get_extensions, get_extension_filters, get_extension_settings, install_extension, uninstall_extension, update_extension_settings};
use crate::commands::config::{get_user_config, patch_user_config};
use crate::commands::progress::{get_content_progress, get_continue_watching, update_anime_progress, update_chapter_progress};
use crate::commands::intergations::{list_trackers, add_integration, remove_integration, set_sync_enabled};
use crate::commands::backups::{list_backups, create_manual_backup, delete_backup, restore_backup, download_backup};
use crate::commands::logs::get_system_logs;

#[cfg(feature = "watchparty")]
use crate::commands::watchparty::{
    start_watchparty, stop_watchparty, watchparty_status,
    create_watchparty_room, delete_watchparty_room,
    list_watchparty_rooms, get_watchparty_room, join_watchparty_room,
};

#[cfg(feature = "discord-rpc")]
use crate::commands::discord::{set_activity, clear_activity};

#[derive(Default)]
pub struct TauriSession {
    pub user_id: RwLock<Option<String>>,
}


pub async fn require_auth(session_state: &TauriSession) -> Result<i32, CoreError> {
    let user = session_state.user_id.read().await;

    let uid_str = match &*user {
        Some(uid) => uid.clone(),
        None => return Err(CoreError::AuthError("error.auth.unauthorized".into())),
    };

    uid_str.parse::<i32>().map_err(|_| {
        CoreError::AuthError("error.auth.invalid_user_id".into())
    })
}

pub fn run_inner() -> anyhow::Result<()> {
    let log_store = new_log_store();

    let memory_layer = MemoryLogLayer {
        store: log_store.clone(),
        limit: 1000,
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "hoshi_app=debug,hoshi_core=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(memory_layer)
        .init();

    info!("Starting Hoshi Desktop Application...");

    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_deep_link::init());

    #[cfg(mobile)]
    {
        builder = builder.plugin(headless_plugin_init());
    }

    builder
        .setup(move |app| {
            let base_dir = app.path().app_data_dir()
                .map_err(|e| anyhow::anyhow!("Failed to obtain app_data_dir: {}", e))?;

            let paths = hoshi_core::paths::AppPaths::from_base(base_dir);
            let headless = std::sync::Arc::new(headless::TauriHeadless::new(app.handle().clone()));
            let handle = app.handle().clone();

            app.listen_any("repository://deep-link", move |event| {
                let url = event.payload();
                if !url.is_empty() {
                    debug!(url = %url, "Deep link received");
                    let _ = handle.emit("auth-callback", url);
                }
            });

            async_runtime::block_on(async {
                let state = hoshi_core::build_app_state(paths, headless, log_store).await
                    .map_err(|e| {
                        error!(error = ?e, "FATAL: Failed to build AppState during startup");
                        anyhow::anyhow!("AppState Error: {}", e)
                    })?;

                app.manage(state);
                app.manage(TauriSession::default());

                #[cfg(feature = "watchparty")]
                app.manage(hoshi_watchparty::WatchPartyServerState::new());

                Ok::<(), anyhow::Error>(())
            })?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_system_logs,
            login, register, logout,
            get_current_profile, get_all_users, get_user, get_me, update_me, delete_me, change_password, upload_avatar, delete_avatar,
            get_trending, get_home_content, get_content, update_content, search_content, get_content_items, play_content_by_number, add_tracker_mapping, add_extension_source, update_extension_mapping, update_tracker_mapping, delete_tracker_mapping, resolve_by_tracker, resolve_by_extension, link_tracker, resolve_extension_item, search_extension_direct,
            get_schedule,
            get_list, get_single_entry, upsert_entry, delete_entry, get_stats,
            proxy_fetch_text, proxy_fetch_bytes,
            get_extensions, get_extension_filters, get_extension_settings, install_extension, uninstall_extension, update_extension_settings,
            get_user_config, patch_user_config,
            get_content_progress, get_continue_watching, update_anime_progress, update_chapter_progress,
            list_trackers, add_integration, remove_integration, set_sync_enabled,
            list_backups, create_manual_backup, delete_backup, restore_backup, download_backup,
            #[cfg(feature = "watchparty")]
            start_watchparty,
            #[cfg(feature = "watchparty")]
            stop_watchparty,
            #[cfg(feature = "watchparty")]
            watchparty_status,
            #[cfg(feature = "watchparty")]
            create_watchparty_room,
            #[cfg(feature = "watchparty")]
            delete_watchparty_room,
            #[cfg(feature = "watchparty")]
            list_watchparty_rooms,
            #[cfg(feature = "watchparty")]
            get_watchparty_room,
            #[cfg(feature = "watchparty")]
            join_watchparty_room,
            #[cfg(feature = "discord-rpc")]
            set_activity,
            #[cfg(feature = "discord-rpc")]
            clear_activity,
            #[cfg(mobile)]
            notify_done,
        ])
        .run(tauri::generate_context!())
        .map_err(|e| anyhow::anyhow!("Tauri runtime error: {}", e))?;

    Ok(())
}

#[cfg(not(mobile))]
pub fn run() -> anyhow::Result<()> {
    run_inner()
}

#[cfg(mobile)]
#[tauri::mobile_entry_point]
pub fn run() {
    run_inner().expect("failed to run mobile app");
}