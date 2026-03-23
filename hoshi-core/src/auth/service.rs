use crate::auth::repository::AuthRepo;
use crate::error::{CoreError, CoreResult};
use crate::state::AppState;
use crate::users::repository::UserRepo;
use bcrypt::{hash, verify, DEFAULT_COST};
use serde::{Deserialize, Serialize};

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
    pub password: Option<String>,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub success: bool,
    pub user: UserInfo,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub id: i32,
    pub username: String,
    pub avatar: Option<String>,
}

pub struct AuthService;

impl AuthService {
    pub fn login(state: &AppState, payload: LoginRequest) -> CoreResult<UserInfo> {
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

        if let Some(hash_str) = auth_data.password_hash {
            let password_input = payload.password.ok_or_else(|| {
                CoreError::AuthError("Password required".into())
            })?;

            let is_valid = verify(&password_input, &hash_str)
                .map_err(|_| CoreError::Internal("Password verification failed".into()))?;

            if !is_valid {
                return Err(CoreError::AuthError("Incorrect password".into()));
            }
        }

        let user_info = UserInfo {
            id: payload.user_id,
            username: auth_data.username,
            avatar: auth_data.avatar,
        };

        Self::set_active_user(state, Some(payload.user_id))?;

        Ok(user_info)
    }

    pub fn register(state: &AppState, payload: RegisterRequest) -> CoreResult<UserInfo> {
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
            UserRepo::create_user(&conn_lock, &payload.username, password_hash)?
        };

        let user_info = UserInfo {
            id: user_id as i32,
            username: payload.username,
            avatar: None,
        };

        Self::set_active_user(state, Some(user_id as i32))?;

        Ok(user_info)
    }

    pub fn logout(state: &AppState) -> CoreResult<()> {
        Self::set_active_user(state, None)
    }

    pub fn get_active_user(state: &AppState) -> CoreResult<Option<UserInfo>> {
        let db = &state.db;
        let conn = db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;

        if let Some(user_id) = AuthRepo::get_active_user(&conn_lock)? {
            if let Some(auth_data) = UserRepo::find_auth_data_by_id(&conn_lock, user_id)? {
                return Ok(Some(UserInfo {
                    id: user_id,
                    username: auth_data.username,
                    avatar: auth_data.avatar,
                }));
            }
        }
        Ok(None)
    }

    fn set_active_user(state: &AppState, user_id: Option<i32>) -> CoreResult<()> {
        let db = &state.db;
        let conn = db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;
        AuthRepo::set_active_user(&conn_lock, user_id)
    }
}