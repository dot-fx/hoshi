use crate::error::AppResult;
use axum::extract::Path;
use axum::{extract::State, routing::{get, put}, Extension, Json, Router};
use hoshi_core::users::service::{UserPrivate, UserPublic, UserResponse};
use hoshi_core::{
    state::AppState,
    users::service::{
        ChangePasswordBody, DeleteUserBody, UpdateUserBody, UserService,
    }
};
use serde::Serialize;
use std::sync::Arc;
use axum::body::Bytes;
use axum::http::{header, HeaderMap};

#[derive(Serialize)]
pub struct UsersListResponse {
    pub users: Vec<UserResponse>,
}

#[derive(Serialize)]
pub struct SuccessResponse {
    pub success: bool,
}

#[derive(Serialize)]
pub struct MessageResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Serialize)]
pub struct SingleUserResponse {
    pub user: UserPublic,
}

pub fn user_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/users", get(get_all_users))
        .route("/users/:id", get(get_user))
        .route("/me", get(get_me).put(update_me).delete(delete_me))
        .route("/me/password", put(change_password))
        .route("/me/avatar", put(upload_avatar).delete(delete_avatar))
}

pub async fn get_all_users(
    State(state): State<Arc<AppState>>
) -> AppResult<Json<UsersListResponse>> {
    let users = UserService::get_all_users(&state)?;
    Ok(Json(UsersListResponse { users }))
}

pub async fn get_user(
    Path(id): Path<i32>,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<SingleUserResponse>> {
    let user = UserService::get_user_public(&state, id)?;
    Ok(Json(SingleUserResponse { user }))
}

pub async fn get_me(
    Extension(user_id): Extension<i32>,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<UserPrivate>> {
    let user = UserService::get_me(&state, user_id)?;
    Ok(Json(user))
}

pub async fn update_me(
    Extension(user_id): Extension<i32>,
    State(state): State<Arc<AppState>>,
    Json(updates): Json<UpdateUserBody>,
) -> AppResult<Json<MessageResponse>> {
    UserService::update_user(&state, user_id, updates)?;

    Ok(Json(MessageResponse {
        success: true,
        message: "User updated successfully".into(),
    }))
}

pub async fn delete_me(
    Extension(user_id): Extension<i32>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<DeleteUserBody>,
) -> AppResult<Json<SuccessResponse>> {
    UserService::delete_user(&state, user_id, body)?;

    Ok(Json(SuccessResponse {
        success: true,
    }))
}

pub async fn change_password(
    Extension(user_id): Extension<i32>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<ChangePasswordBody>,
) -> AppResult<Json<MessageResponse>> {
    let has_new =
        UserService::change_password(&state, user_id, body)?;

    Ok(Json(MessageResponse {
        success: true,
        message: if has_new {
            "Password updated successfully".into()
        } else {
            "Password removed successfully".into()
        },
    }))
}

pub async fn upload_avatar(
    Extension(user_id): Extension<i32>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> AppResult<Json<SuccessResponse>> {
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    UserService::upload_avatar(&state, user_id, body.to_vec(), content_type)?;
    Ok(Json(SuccessResponse { success: true }))
}

pub async fn delete_avatar(
    Extension(user_id): Extension<i32>,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<SuccessResponse>> {
    UserService::delete_avatar(&state, user_id)?;
    Ok(Json(SuccessResponse { success: true }))
}