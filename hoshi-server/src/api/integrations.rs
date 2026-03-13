use axum::{
    extract::{Path, State},
    routing::{delete, get, patch, post},
    Extension, Json, Router,
};
use std::sync::Arc;

use crate::error::AppResult;
use hoshi_core::tracker::repository::AddIntegrationRequest;
use hoshi_core::{
    state::AppState,
    tracker::service::{IntegrationService, SuccessResponse, TrackerInfoResponse, SetSyncEnabledRequest},
};

pub fn integration_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/trackers", get(list_trackers))
        .route("/integrations", post(add_integration))
        .route("/integrations/:tracker_name", delete(remove_integration))
        .route("/integrations/:tracker_name/sync", patch(set_sync_enabled))
}

async fn list_trackers(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<i32>,
) -> AppResult<Json<Vec<TrackerInfoResponse>>> {
    let result = IntegrationService::list_trackers(&state, user_id)?;
    Ok(Json(result))
}

async fn add_integration(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<i32>,
    Json(body): Json<AddIntegrationRequest>,
) -> AppResult<Json<SuccessResponse>> {
    let result = IntegrationService::add_integration(state, user_id, body).await?;
    Ok(Json(result))
}

async fn remove_integration(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<i32>,
    Path(tracker_name): Path<String>,
) -> AppResult<Json<SuccessResponse>> {
    let result = IntegrationService::remove_integration(&state, user_id, &tracker_name)?;
    Ok(Json(result))
}

async fn set_sync_enabled(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<i32>,
    Path(tracker_name): Path<String>,
    Json(body): Json<SetSyncEnabledRequest>,
) -> AppResult<Json<SuccessResponse>> {
    let result = IntegrationService::set_sync_enabled(&state, user_id, &tracker_name, body.enabled)?;
    Ok(Json(result))
}