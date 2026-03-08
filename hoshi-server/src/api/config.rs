use crate::error::AppResult;
use axum::{
    extract::State,
    routing::{get, patch},
    Extension, Json, Router,
};
use hoshi_core::{
    state::AppState,
    config::service::ConfigService,
};
use serde_json::Value;
use std::sync::Arc;

pub fn config_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/config", get(get_config).patch(update_config))
}

async fn get_config(
    Extension(user_id): Extension<i32>,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Value>> {
    let config = ConfigService::get_config(&state, user_id)?;
    Ok(Json(config))
}

async fn update_config(
    Extension(user_id): Extension<i32>,
    State(state): State<Arc<AppState>>,
    Json(patch): Json<Value>,
) -> AppResult<Json<Value>> {
    let updated = ConfigService::patch_config(&state, user_id, patch)?;
    Ok(Json(updated))
}