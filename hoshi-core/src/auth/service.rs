use crate::auth::repository::AuthRepo;
use crate::error::{CoreError, CoreResult};
use crate::state::AppState;
use crate::users::repository::UserRepo;
use bcrypt::{hash, verify, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use tracing::{error, info, instrument, warn};
use crate::auth::types::{LoginRequest, RegisterRequest, UserInfo};

pub struct AuthService;

impl AuthService {
    #[instrument(skip(state, payload), fields(user_id = payload.user_id))]
    pub fn login(state: &AppState, payload: LoginRequest) -> CoreResult<UserInfo> {
        let db = &state.db;

        if payload.user_id <= 0 {
            warn!("Login rejected: invalid user ID");
            return Err(CoreError::BadRequest("error.auth.invalid_user_id".into()));
        }

        let auth_data = {
            let conn = db.connection();
            let conn_lock = conn.lock().map_err(|_| CoreError::Internal("error.system.db_lock".into()))?;

            UserRepo::find_auth_data_by_id(&conn_lock, payload.user_id)?
                .ok_or_else(|| {
                    warn!("Login rejected: user not found in database");
                    CoreError::NotFound("error.auth.user_not_found".into())
                })?
        };

        if let Some(hash_str) = auth_data.password_hash {
            let password_input = payload.password.ok_or_else(|| {
                warn!("Login rejected: password required but not provided");
                CoreError::AuthError("error.auth.password_required".into())
            })?;

            let is_valid = verify(&password_input, &hash_str)
                .map_err(|e| {
                    error!(error = ?e, "Failed to verify password hash");
                    CoreError::Internal("error.auth.password_verification_failed".into())
                })?;

            if !is_valid {
                warn!("Login rejected: incorrect password");
                return Err(CoreError::AuthError("error.auth.incorrect_password".into()));
            }
        }

        let user_info = UserInfo {
            id: payload.user_id,
            username: auth_data.username,
            avatar: auth_data.avatar,
        };

        Self::set_active_user(state, Some(payload.user_id))?;

        info!("User logged in successfully");
        Ok(user_info)
    }

    #[instrument(skip(state, payload), fields(username = %payload.username))]
    pub fn register(state: &AppState, payload: RegisterRequest) -> CoreResult<UserInfo> {
        let db = &state.db;

        if payload.username.is_empty() {
            warn!("Registration rejected: empty username");
            return Err(CoreError::BadRequest("error.auth.username_required".into()));
        }

        let password_hash = if let Some(pass) = &payload.password {
            if !pass.trim().is_empty() {
                Some(hash(pass.trim(), DEFAULT_COST)
                    .map_err(|e| {
                        error!(error = ?e, "Failed to hash password during registration");
                        CoreError::Internal("error.auth.hashing_failed".into())
                    })?)
            } else {
                None
            }
        } else {
            None
        };

        let user_id = {
            let conn = db.connection();
            let conn_lock = conn.lock().map_err(|_| CoreError::Internal("error.system.db_lock".into()))?;
            UserRepo::create_user(&conn_lock, &payload.username, password_hash)?
        };

        info!(user_id = user_id, "New user registered successfully");

        let user_info = UserInfo {
            id: user_id as i32,
            username: payload.username,
            avatar: None,
        };

        Self::set_active_user(state, Some(user_id as i32))?;

        Ok(user_info)
    }

    #[instrument(skip(state))]
    pub fn logout(state: &AppState) -> CoreResult<()> {
        info!("User logged out");
        Self::set_active_user(state, None)
    }

    pub fn get_active_user(state: &AppState) -> CoreResult<Option<UserInfo>> {
        let db = &state.db;
        let conn = db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("error.system.db_lock".into()))?;

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
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("error.system.db_lock".into()))?;
        AuthRepo::set_active_user(&conn_lock, user_id)
    }
}