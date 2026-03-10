use std::sync::Arc;
use tokio::sync::RwLock;

use crate::db::DatabaseManager;
use crate::extensions::ExtensionManager;
use crate::headless::HeadlessHandle;
use crate::paths::AppPaths;
use crate::tracker::provider::TrackerRegistry;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseManager>,
    pub extension_manager: Arc<RwLock<ExtensionManager>>,
    pub tracker_registry: Arc<TrackerRegistry>,
    pub paths: Arc<AppPaths>,
    pub headless:          HeadlessHandle,
}