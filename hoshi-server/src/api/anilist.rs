use axum::{
    extract::State,
    routing::post,
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

use crate::error::AppResult;
use hoshi_core::error::CoreError;
use hoshi_core::state::AppState;

pub fn anilist_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/anilist/store", post(store_token))
}

#[derive(Debug, Deserialize)]
struct StoreTokenBody {
    pub user_id: i32,
    pub access_token: String,
    #[serde(default = "default_token_type")]
    pub token_type: String,
}

fn default_token_type() -> String {
    "Bearer".to_string()
}

async fn store_token(
    State(state): State<Arc<AppState>>,
    Json(body): Json<StoreTokenBody>,
) -> AppResult<Json<serde_json::Value>> {

    let provider = state
        .tracker_registry
        .get("anilist")
        .ok_or_else(|| CoreError::Internal("AniList provider not registered".into()))?;

    let token_data = provider
        .validate_and_store_token(&body.access_token, &body.token_type)
        .await
        .map_err(|e| CoreError::AuthError(e.to_string()))?;

    let conn = state.db.connection();
    let conn_lock = conn
        .lock()
        .map_err(|_| CoreError::Internal("DB lock error".into()))?;

    hoshi_core::tracker::repository::TrackerRepo::save_integration(
        &conn_lock,
        body.user_id,
        "anilist",
        &token_data.tracker_user_id,
        &token_data.access_token,
        token_data.refresh_token.as_deref(),
        &token_data.token_type,
        &token_data.expires_at,
    )?;

    Ok(Json(json!({
        "ok": true,
        "tracker": "anilist",
        "tracker_user_id": token_data.tracker_user_id,
        "expires_at": token_data.expires_at,
    })))
}