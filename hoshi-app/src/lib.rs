use tauri::{Manager, async_runtime};
use tokio::sync::RwLock;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub mod commands;
pub mod headless;

use crate::commands::auth::{login, register, logout, restore_session};
use crate::commands::users::{get_all_users, get_user, get_me, update_me, delete_me, change_password, upload_avatar, delete_avatar};
use crate::commands::content::{get_trending, get_home_content, create_content, get_content, update_content, search_content, get_content_items, play_content_by_number, add_tracker_mapping, add_extension_source, update_extension_mapping, update_tracker_mapping, delete_tracker_mapping, resolve_by_tracker, resolve_by_extension, link_tracker, resolve_extension_item, search_extension_direct};
use crate::commands::schedule::{get_schedule};
use crate::commands::list::{get_list, get_single_entry, upsert_entry, delete_entry, get_stats};
use crate::commands::booru::{booru_search, booru_get_info, booru_autocomplete};
use crate::commands::collections::{get_collections, get_collection, create_collection, update_collection, delete_collection, get_collection_images, add_image_to_collection, remove_image_from_collection, reorder_collection};
use crate::commands::proxy::{proxy_fetch_text, proxy_fetch_bytes};
use crate::commands::extensions::{get_extensions, get_anime_extensions, get_booru_extensions, get_manga_extensions, get_novel_extensions, get_extension_filters, get_extension_settings};
use crate::commands::config::{get_user_config, patch_user_config};
use crate::commands::progress::{get_content_progress, get_continue_watching, update_anime_progress, update_chapter_progress};
use crate::commands::intergations::{list_trackers, add_integration, remove_integration, set_sync_enabled};
use crate::commands::backups::{list_backups, create_manual_backup, delete_backup, restore_backup, download_backup};

#[derive(Default)]
pub struct TauriSession {
    pub user_id: RwLock<Option<String>>,
}

pub async fn require_auth(session_state: &TauriSession) -> Result<String, String> {
    let user = session_state.user_id.read().await;
    match &*user {
        Some(uid) => Ok(uid.clone()),
        None => Err("Unauthorized".to_string()),
    }
}

pub fn run_inner() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "hoshi_app=debug,hoshi_core=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Hoshi (Tauri)...");

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let base_dir = app.path().app_data_dir()
                .map_err(|e| anyhow::anyhow!("No se pudo obtener app_data_dir: {}", e))?;

            let paths = hoshi_core::paths::AppPaths::from_base(base_dir);
            let headless = std::sync::Arc::new(headless::TauriHeadless::new(app.handle().clone()));

            async_runtime::block_on(async {
                let state = hoshi_core::build_app_state_with_headless(paths, headless).await?;
                app.manage(state);
                app.manage(TauriSession::default());
                Ok::<(), anyhow::Error>(())
            })?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            login,
            register,
            logout,
            restore_session,
            get_all_users,
            get_user,
            get_me,
            update_me,
            delete_me,
            change_password,
            upload_avatar,
            delete_avatar,
            get_trending, get_home_content, create_content, get_content, update_content, search_content, get_content_items, play_content_by_number, add_tracker_mapping, add_extension_source, update_extension_mapping, update_tracker_mapping, delete_tracker_mapping, resolve_by_tracker, resolve_by_extension, link_tracker, resolve_extension_item, search_extension_direct,
            get_schedule,
            get_list, get_single_entry, upsert_entry, delete_entry, get_stats,
            booru_search, booru_get_info, booru_autocomplete,
            get_collections, get_collection, create_collection, update_collection, delete_collection, get_collection_images, add_image_to_collection, remove_image_from_collection, reorder_collection,
            proxy_fetch_text, proxy_fetch_bytes,
            get_extensions, get_anime_extensions, get_booru_extensions, get_manga_extensions, get_novel_extensions, get_extension_filters, get_extension_settings, 
            get_user_config, patch_user_config,
            get_content_progress, get_continue_watching, update_anime_progress, update_chapter_progress,
            list_trackers, add_integration, remove_integration, set_sync_enabled,
            list_backups, create_manual_backup, delete_backup, restore_backup, download_backup
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