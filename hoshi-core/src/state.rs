use std::sync::Arc;

use crate::db::DatabaseManager;
use crate::extensions::ExtensionManager;
use crate::paths::AppPaths;
use crate::tracker::provider::TrackerRegistry;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseManager>,
    pub extension_manager: Arc<ExtensionManager>,
    pub tracker_registry: Arc<TrackerRegistry>,
    pub paths: Arc<AppPaths>,
}