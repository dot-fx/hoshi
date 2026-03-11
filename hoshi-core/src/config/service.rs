use crate::config::model::UserConfig;
use crate::config::repository::ConfigRepo;
use crate::error::{CoreError, CoreResult};
use crate::state::AppState;
use serde_json::Value;

pub struct ConfigService;

impl ConfigService {
    pub fn get_config(state: &AppState, user_id: i32) -> CoreResult<UserConfig> {
        let conn = state.db.connection();
        let conn_lock = conn
            .lock()
            .map_err(|_| CoreError::Internal("DB Lock error".into()))?;

        ConfigRepo::get_config(&conn_lock, user_id)
    }

    pub fn patch_config(state: &AppState, user_id: i32, patch: Value) -> CoreResult<UserConfig> {
        if !patch.is_object() {
            return Err(CoreError::BadRequest(
                "Config patch must be a JSON object".into(),
            ));
        }

        let conn = state.db.connection();
        let conn_lock = conn
            .lock()
            .map_err(|_| CoreError::Internal("DB Lock error".into()))?;

        ConfigRepo::patch_config(&conn_lock, user_id, &patch)
    }
}