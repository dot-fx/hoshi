use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

use hoshi_core::error::CoreError;

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Core(#[from] CoreError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Core(CoreError::NotFound(msg)) => (StatusCode::NOT_FOUND, msg),
            AppError::Core(CoreError::BadRequest(msg)) => (StatusCode::BAD_REQUEST, msg),
            AppError::Core(CoreError::AuthError(msg)) => (StatusCode::UNAUTHORIZED, msg),
            AppError::Core(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;