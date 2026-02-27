pub mod state;
pub mod db;
pub mod extensions;
pub mod config;
pub mod rpc;
pub mod users;
pub mod booru;
pub mod auth;
pub mod list;
pub mod content;
pub mod proxy;
pub mod tracker;
pub mod paths;
pub mod error;
pub mod collections;

use tokio::sync::RwLock;
use std::sync::{Arc, Mutex};
use rpc::service::RpcManager;
use state::AppState;
use tracker::provider::build_registry;

pub async fn build_app_state() -> anyhow::Result<Arc<AppState>> {
    config::ensure_config_file()?;
    let loaded_config = config::load_config()?;

    db::init_all_databases()?;
    let db_manager = db::DatabaseManager::new()?;
    let db = Arc::new(db_manager);

    let mut extension_manager = extensions::ExtensionManager::new()?;
    extension_manager.load_extensions().await?;
    let ext_manager_arc = Arc::new(extension_manager);

    let rpc_manager = RpcManager::new("1461370397734731808".to_string());
    let config_arc = Arc::new(RwLock::new(loaded_config));

    let tracker_registry = Arc::new(build_registry());

    let state = Arc::new(AppState {
        config: config_arc,
        rpc_manager: Arc::new(Mutex::new(rpc_manager)),
        db,
        extension_manager: ext_manager_arc,
        tracker_registry,
    });

    Ok(state)
}