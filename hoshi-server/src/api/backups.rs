use axum::{
    extract::{Path, State},
    http::header,
    response::IntoResponse,
    routing::{delete, get, post},
    Extension, Json, Router,
};
use std::sync::Arc;

use crate::error::AppResult;
use hoshi_core::{
    state::AppState,
    backup::repository::ListBackupMeta,
    backup::service::BackupService,
    tracker::service::SuccessResponse,
};

pub fn backup_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/backups", get(list_backups))
        .route("/backups", post(create_manual_backup))
        .route("/backups/:id", delete(delete_backup))
        .route("/backups/:id/restore", post(restore_backup))
        .route("/backups/:id/download", get(download_backup))
}

async fn list_backups(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<i32>,
) -> AppResult<Json<Vec<ListBackupMeta>>> {
    let result = BackupService::list_backups(&state, user_id)?;
    Ok(Json(result))
}

async fn create_manual_backup(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<i32>,
) -> AppResult<Json<ListBackupMeta>> {
    let result = BackupService::create_manual(&state, user_id)?;
    Ok(Json(result))
}

async fn delete_backup(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<i32>,
    Path(backup_id): Path<i64>,
) -> AppResult<Json<SuccessResponse>> {
    let deleted = BackupService::delete_backup(&state, user_id, backup_id)?;
    Ok(Json(SuccessResponse { success: deleted }))
}

async fn restore_backup(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<i32>,
    Path(backup_id): Path<i64>,
) -> AppResult<Json<SuccessResponse>> {
    BackupService::restore_backup(&state, user_id, backup_id)?;
    Ok(Json(SuccessResponse { success: true }))
}

async fn download_backup(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<i32>,
    Path(backup_id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    let json = BackupService::read_backup_json(&state, user_id, backup_id)?;
    let filename = format!("backup_{}.json", backup_id);

    let response = (
        [
            (header::CONTENT_TYPE, "application/json"),
            (
                header::CONTENT_DISPOSITION,
                Box::leak(format!("attachment; filename=\"{}\"", filename).into_boxed_str()),
            ),
        ],
        json,
    );

    Ok(response)
}