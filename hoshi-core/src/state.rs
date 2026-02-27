use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;

use crate::config::Config;
use crate::rpc::service::RpcManager;
use crate::db::DatabaseManager;
use crate::extensions::ExtensionManager;
use crate::tracker::provider::TrackerRegistry;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<RwLock<Config>>,
    pub rpc_manager: Arc<Mutex<RpcManager>>,
    pub db: Arc<DatabaseManager>,
    pub extension_manager: Arc<ExtensionManager>,
    pub tracker_registry: Arc<TrackerRegistry>,
}