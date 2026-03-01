use axum::{
    extract::{Extension, Query, State},
    routing::get,
    Json, Router,
};
use serde::Serialize;
use std::sync::Arc;

use crate::error::AppResult;
use hoshi_core::{
    schedule::{
        repository::{AiringEntryEnriched, ScheduleWindow},
        service::ScheduleService,
    },
    state::AppState,
};


#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleResponse {
    pub success: bool,
    pub data:    Vec<AiringEntryEnriched>,
    pub total:   usize,
}


pub fn schedule_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/schedule", get(get_schedule))
}

async fn get_schedule(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<i32>,
    Query(window): Query<ScheduleWindow>,
) -> AppResult<Json<ScheduleResponse>> {
    let data = ScheduleService::get_schedule(state, user_id, window).await?;
    let total = data.len();

    Ok(Json(ScheduleResponse {
        success: true,
        data,
        total,
    }))
}