use crate::config::model::UserConfig;
use crate::config::repository::ConfigRepo;
use crate::error::{CoreError, CoreResult};
use crate::state::AppState;
use serde_json::Value;
use tracing::{info, warn, instrument};

pub struct ConfigService;

impl ConfigService {
    #[instrument(skip(state))]
    pub fn get_config(state: &AppState, user_id: i32) -> CoreResult<UserConfig> {
        let conn = state.db.connection();
        let conn_lock = conn
            .lock()
            .map_err(|_| CoreError::Internal("error.system.db_lock".into()))?;

        ConfigRepo::get_config(&conn_lock, user_id)
    }

    #[instrument(skip(state, patch))]
    pub fn patch_config(state: &AppState, user_id: i32, patch: Value) -> CoreResult<UserConfig> {
        if !patch.is_object() {
            warn!("Failed to patch config: patch provided is not a JSON object");
            return Err(CoreError::BadRequest(
                "error.config.invalid_patch_format".into(),
            ));
        }

        let conn = state.db.connection();
        let conn_lock = conn
            .lock()
            .map_err(|_| CoreError::Internal("error.system.db_lock".into()))?;

        let new_config = ConfigRepo::patch_config(&conn_lock, user_id, &patch)?;
        info!("User configuration updated successfully");

        Ok(new_config)
    }
}