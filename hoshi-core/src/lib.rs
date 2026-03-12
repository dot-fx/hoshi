pub mod state;
pub mod db;
pub mod extensions;
pub mod users;
pub mod booru;
pub mod auth;
pub mod list;
pub mod content;
pub mod tracker;
pub mod paths;
pub mod error;
pub mod collections;
pub mod schedule;
pub mod config;
pub mod headless;
pub mod proxy;
pub mod progress;

use headless::{HeadlessHandle, noop_headless};
use state::AppState;
use paths::AppPaths;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracker::provider::build_registry;

pub async fn build_app_state(paths: AppPaths) -> anyhow::Result<Arc<AppState>> {
    build_app_state_with_headless(paths, noop_headless()).await
}

pub async fn build_app_state_with_headless(
    paths: AppPaths,
    headless: HeadlessHandle,
) -> anyhow::Result<Arc<AppState>> {
    paths.ensure_dirs()?;

    db::init_all_databases(&paths)?;
    let db_manager = db::DatabaseManager::new(&paths)?;
    let db = Arc::new(db_manager);

    let mut extension_manager = extensions::ExtensionManager::new(&paths)?;
    extension_manager.load_extensions().await?;
    extension_manager.set_headless(headless.clone());
    let ext_manager_arc = Arc::new(RwLock::new(extension_manager));

    let tracker_registry = Arc::new(build_registry());

    let state = Arc::new(AppState {
        db,
        extension_manager: ext_manager_arc,
        tracker_registry,
        paths: Arc::new(paths),
        headless,
    });

    Ok(state)
}