use axum::{
    extract::{Path, State},
    routing::{delete, get, post},
    Extension, Json, Router,
};
use std::sync::Arc;

use crate::error::AppResult;
use hoshi_core::{
    state::AppState,
    tracker::{
        repository::TrackerIntegration,
        service::{IntegrationService, SuccessResponse, TrackerInfoResponse},
    },
};

pub fn integration_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/trackers", get(list_trackers))
        .route("/integrations", post(add_integration))
        .route("/integrations/:tracker_name", delete(remove_integration))
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
    Json(body): Json<TrackerIntegration>,
) -> AppResult<Json<SuccessResponse>> {
    let result = IntegrationService::add_integration(&state, user_id, body)?;
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