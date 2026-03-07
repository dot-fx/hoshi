use tauri::{Manager, async_runtime};
use tokio::sync::RwLock;

pub mod commands;

use crate::commands::auth::{login, register, logout, restore_session};
use crate::commands::users::{get_all_users, get_user, get_me, update_me, delete_me, change_password, upload_avatar, delete_avatar};
use crate::commands::content::{get_home_content, create_content, get_content, update_content, search_content, get_content_items, play_content_by_number, add_tracker_mapping, add_extension_source, update_extension_mapping, update_tracker_mapping, delete_tracker_mapping, resolve_by_tracker, resolve_by_extension, link_tracker, resolve_extension_item, search_extension_direct};
use crate::commands::schedule::{get_schedule};
use crate::commands::list::{get_list, get_single_entry, upsert_entry, delete_entry};
use crate::commands::booru::{booru_search, booru_get_info, booru_autocomplete};
use crate::commands::collections::{get_collections, get_collection, create_collection, update_collection, delete_collection, get_collection_images, add_image_to_collection, remove_image_from_collection, reorder_collection};
use crate::commands::proxy::{proxy_fetch_text, proxy_fetch_bytes};
use crate::commands::extensions::{get_extensions, get_anime_extensions, get_booru_extensions, get_manga_extensions, get_novel_extensions, get_extension_filters, get_extension_settings};

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

pub fn run() -> anyhow::Result<()> {
    tauri::Builder::default()
        .setup(|app| {
            async_runtime::block_on(async {
                let state = hoshi_core::build_app_state().await?;
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
            get_home_content, create_content, get_content, update_content, search_content, get_content_items, play_content_by_number, add_tracker_mapping, add_extension_source, update_extension_mapping, update_tracker_mapping, delete_tracker_mapping, resolve_by_tracker, resolve_by_extension, link_tracker, resolve_extension_item, search_extension_direct,
            get_schedule,
            get_list, get_single_entry, upsert_entry, delete_entry,
            booru_search, booru_get_info, booru_autocomplete,
            get_collections, get_collection, create_collection, update_collection, delete_collection, get_collection_images, add_image_to_collection, remove_image_from_collection, reorder_collection,
            proxy_fetch_text, proxy_fetch_bytes,
            get_extensions, get_anime_extensions, get_booru_extensions, get_manga_extensions, get_novel_extensions, get_extension_filters, get_extension_settings
        ])
        .run(tauri::generate_context!())
        .map_err(|e| anyhow::anyhow!("Tauri runtime error: {}", e))?;

    Ok(())
}