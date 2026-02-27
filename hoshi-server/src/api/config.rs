use axum::{extract::Path, extract::Json, routing::get, Router};
use serde_json::Value;
use std::sync::Arc;

use crate::error::AppResult;

use hoshi_core::{
    state::AppState,
    config::service::ConfigService
};

pub fn config_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/config", get(get_full_config).post(update_config))
        .route("/config/:section", get(get_config_section).post(update_config_section))
}

async fn get_full_config() -> AppResult<Json<Value>> {
    let data = ConfigService::get_public_config_with_schema()?;
    Ok(Json(data))
}

async fn get_config_section(Path(section): Path<String>) -> AppResult<Json<Value>> {
    let data = ConfigService::get_config_section(&section)?;
    Ok(Json(data))
}

async fn update_config(Json(body): Json<Value>) -> AppResult<Json<Value>> {
    let updated = ConfigService::update_config_partial(body)?;
    Ok(Json(updated))
}

async fn update_config_section(
    Path(section): Path<String>,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    let updated = ConfigService::update_config_section(&section, body)?;
    Ok(Json(updated))
}