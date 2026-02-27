use axum::{
    extract::State,
    routing::post,
    Json, Router,
};
use serde::Serialize;
use std::sync::Arc;

use crate::error::AppResult;
use hoshi_core::{
    state::AppState,
    rpc::service::RpcData,
    error::CoreError,
};

#[derive(Serialize)]
pub struct SuccessResponse {
    pub success: bool,
}

pub fn rpc_routes() -> Router<Arc<AppState>> {
    Router::new().route("/rpc", post(update_rpc_activity))
}

async fn update_rpc_activity(
    State(state): State<Arc<AppState>>,
    Json(data): Json<RpcData>,
) -> AppResult<Json<SuccessResponse>> {
    let mut rpc_manager = state.rpc_manager.lock().map_err(|_| {
        CoreError::Internal("Failed to lock RPC manager".into())
    })?;

    rpc_manager.set_activity(data)?;

    Ok(Json(SuccessResponse { success: true }))
}