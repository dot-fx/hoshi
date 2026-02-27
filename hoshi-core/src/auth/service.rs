use crate::auth::repository::{AuthRepo, Session};
use crate::db::DatabaseManager;
use crate::error::{CoreError, CoreResult};
use crate::state::AppState;
use crate::users::repository::UserRepo;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub user_id: i32,
    pub password: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    pub username: String,
    pub profile_picture_url: Option<String>,
    pub password: Option<String>,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub success: bool,
    pub user: UserInfo,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub id: i32,
    pub username: String,
    pub avatar: Option<String>,
}

pub struct AuthService;

impl AuthService {

    pub fn login(state: &AppState, payload: LoginRequest) -> CoreResult<(UserInfo, String)> {
        let db = &state.db;
        if payload.user_id <= 0 {
            return Err(CoreError::BadRequest("Invalid userId provided".into()));
        }

        let auth_data = {
            let conn = db.connection();
            let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;

            UserRepo::find_auth_data_by_id(&conn_lock, payload.user_id)?
                .ok_or_else(|| CoreError::NotFound("User not found".into()))?
        };

        let username = auth_data.username;
        let avatar = auth_data.avatar;
        let password_hash = auth_data.password_hash;

        if let Some(hash_str) = password_hash {
            let password_input = payload.password.ok_or_else(|| {
                CoreError::AuthError("Password required".into())
            })?;

            let is_valid = verify(&password_input, &hash_str)
                .map_err(|_| CoreError::Internal("Password verification failed".into()))?;

            if !is_valid {
                return Err(CoreError::AuthError("Incorrect password".into()));
            }
        }

        let session_id = Self::create_session_internal(db, payload.user_id)?;

        let user_info = UserInfo {
            id: payload.user_id,
            username,
            avatar,
        };

        Ok((user_info, session_id))
    }

    pub fn register(state: &AppState, payload: RegisterRequest) -> CoreResult<(UserInfo, String)> {
        let db = &state.db;

        if payload.username.is_empty() {
            return Err(CoreError::BadRequest("Username is required".into()));
        }

        let password_hash = if let Some(pass) = &payload.password {
            if !pass.trim().is_empty() {
                Some(hash(pass.trim(), DEFAULT_COST)
                    .map_err(|_| CoreError::Internal("Hashing failed".into()))?)
            } else {
                None
            }
        } else {
            None
        };

        let user_id = {
            let conn = db.connection();
            let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;
            UserRepo::create_user(&conn_lock, &payload.username, payload.profile_picture_url.clone(), password_hash)?
        };

        let session_id = Self::create_session_internal(db, user_id as i32)?;

        let user_info = UserInfo {
            id: user_id as i32,
            username: payload.username,
            avatar: payload.profile_picture_url,
        };

        Ok((user_info, session_id))
    }

    pub fn logout(state: &AppState, session_id: &str) -> CoreResult<()> {
        let db = &state.db;
        let conn = db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;
        AuthRepo::delete_session(&conn_lock, session_id)
    }

    pub fn get_session(state: &AppState, session_id: &str) -> CoreResult<Option<Session>> {
        let conn = state.db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;
        AuthRepo::get_session(&conn_lock, session_id)
    }

    pub fn delete_session(state: &AppState, session_id: &str) -> CoreResult<()> {
        let conn = state.db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;
        AuthRepo::delete_session(&conn_lock, session_id)
    }

    pub fn cleanup_expired_sessions(state: &AppState) -> CoreResult<()> {
        let conn = state.db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;
        AuthRepo::cleanup_expired_sessions(&conn_lock)
    }

    fn create_session_internal(db: &DatabaseManager, user_id: i32) -> CoreResult<String> {
        let session = Session {
            session_id: Uuid::new_v4().to_string(),
            user_id,
            expires_at: Utc::now() + Duration::days(7),
        };

        let conn = db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;
        AuthRepo::create_session(&conn_lock, &session)?;

        Ok(session.session_id)
    }
}