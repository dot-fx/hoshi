use crate::error::{CoreError, CoreResult};
use crate::state::AppState;
use crate::config::repository::ConfigRepo;
use serde_json::Value;

pub struct ConfigService;

impl ConfigService {
    /// Devuelve la config completa del usuario. Siempre retorna un objeto JSON válido.
    pub fn get_config(state: &AppState, user_id: i32) -> CoreResult<Value> {
        let conn = state.db.connection();
        let conn_lock = conn
            .lock()
            .map_err(|_| CoreError::Internal("DB Lock error".into()))?;

        ConfigRepo::get_config(&conn_lock, user_id)
    }

    /// Merge parcial de la config. Devuelve la config resultante completa.
    pub fn patch_config(state: &AppState, user_id: i32, patch: Value) -> CoreResult<Value> {
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