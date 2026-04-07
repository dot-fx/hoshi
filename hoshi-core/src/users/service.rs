use crate::error::{CoreError, CoreResult};
use crate::state::AppState;
use crate::users::repository::UserRepo;
use bcrypt::{hash, verify, DEFAULT_COST};
use tracing::{error, info, instrument, warn};
use crate::users::types::{ChangePasswordBody, DeleteUserBody, UpdateUserBody, UserPrivate, UserPublic, UserResponse};

pub struct UserService;

impl UserService {
    pub fn get_all_users(state: &AppState) -> CoreResult<Vec<UserResponse>> {
        let db = &state.db;
        let conn = db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("error.system.db_lock".into()))?;
        UserRepo::get_all_users(&conn_lock)
    }

    #[instrument(skip(state))]
    pub fn get_me(state: &AppState, user_id: i32) -> CoreResult<UserPrivate> {
        let conn = state.db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("error.system.db_lock".into()))?;

        let user = UserRepo::get_user_by_id(&conn_lock, user_id)?
            .ok_or_else(|| {
                warn!("User not found in session");
                CoreError::NotFound("error.user.not_found".into())
            })?;

        Ok(UserPrivate {
            id: user.id,
            username: user.username,
            avatar: user.avatar,
            has_password: user.password_hash.is_some(),
        })
    }

    #[instrument(skip(state, updates), fields(id = id))]
    pub fn update_user(state: &AppState, id: i32, updates: UpdateUserBody) -> CoreResult<()> {
        let password_hash_update = if let Some(password) = &updates.password {
            if password.is_empty() {
                Some(None)
            } else {
                let h = hash(password.trim(), DEFAULT_COST)
                    .map_err(|e| {
                        error!(error = ?e, "Failed to hash new password during update");
                        CoreError::Internal("error.user.hashing_failed".into())
                    })?;
                Some(Some(h))
            }
        } else {
            None
        };

        let db = &state.db;
        let conn = db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("error.system.db_lock".into()))?;

        if updates.username.is_none() && password_hash_update.is_none() {
            return Err(CoreError::BadRequest("error.user.no_updates_provided".into()));
        }

        let changes = UserRepo::update_user(&conn_lock, id, &updates, password_hash_update)?;

        if changes > 0 {
            info!("User profile updated successfully");
            Ok(())
        } else {
            Err(CoreError::NotFound("error.user.not_found".into()))
        }
    }

    #[instrument(skip(state, body), fields(id = id))]
    pub fn delete_user(state: &AppState, id: i32, body: DeleteUserBody) -> CoreResult<()> {
        let conn = state.db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("error.system.db_lock".into()))?;

        let (_, password_hash) = UserRepo::get_user_credentials(&conn_lock, id)?
            .ok_or_else(|| CoreError::NotFound("error.user.not_found".into()))?;

        if let Some(hash_str) = password_hash {
            let pass = body.password.ok_or_else(|| {
                warn!("Account deletion rejected: password required");
                CoreError::AuthError("error.user.password_required".into())
            })?;

            if !verify(&pass, &hash_str).unwrap_or(false) {
                warn!("Account deletion rejected: incorrect password");
                return Err(CoreError::AuthError("error.user.incorrect_password".into()));
            }
        }

        let success = UserRepo::delete_user(&conn_lock, id)?;
        if success {
            info!("User account deleted successfully");
            Ok(())
        } else {
            error!("Failed to delete user record from database");
            Err(CoreError::Internal("error.user.delete_failed".into()))
        }
    }

    #[instrument(skip(state, body), fields(id = id))]
    pub fn change_password(state: &AppState, id: i32, body: ChangePasswordBody) -> CoreResult<bool> {
        let conn = state.db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("error.system.db_lock".into()))?;

        let password_hash = UserRepo::get_password_hash(&conn_lock, id)?;

        if let Some(hash_str) = password_hash {
            if let Some(curr_pass) = &body.current_password {
                if !verify(curr_pass, &hash_str).unwrap_or(false) {
                    warn!("Password change rejected: current password incorrect");
                    return Err(CoreError::AuthError("error.user.current_password_incorrect".into()));
                }
            } else {
                return Err(CoreError::AuthError("error.user.current_password_required".into()));
            }
        }

        let has_new_password = body.new_password.is_some();
        let new_hash = match &body.new_password {
            Some(pass) if !pass.is_empty() => {
                Some(hash(pass.trim(), DEFAULT_COST)
                    .map_err(|e| {
                        error!(error = ?e, "Failed to hash new password during change");
                        CoreError::Internal("error.user.hashing_failed".into())
                    })?)
            }
            _ => None
        };

        UserRepo::update_password(&conn_lock, id, new_hash)?;
        info!("User password changed successfully");
        Ok(has_new_password)
    }

    #[instrument(skip(state, data), fields(id = id, mime = %mime))]
    pub fn upload_avatar(state: &AppState, id: i32, data: Vec<u8>, mime: String) -> CoreResult<()> {
        match mime.as_str() {
            "image/jpeg" | "image/png" | "image/webp" | "image/gif" => {}
            _ => return Err(CoreError::BadRequest("error.user.unsupported_image_format".into())),
        }

        if data.len() > 20 * 1024 * 1024 {
            warn!("Avatar upload rejected: file size {} exceeds 20MB", data.len());
            return Err(CoreError::BadRequest("error.user.image_too_large".into()));
        }

        let conn = state.db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("error.system.db_lock".into()))?;
        UserRepo::update_avatar(&conn_lock, id, Some(data), Some(mime))?;
        info!("User avatar updated successfully");
        Ok(())
    }

    #[instrument(skip(state))]
    pub fn get_user_public(state: &AppState, id: i32) -> CoreResult<UserPublic> {
        let conn = state.db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("error.system.db_lock".into()))?;

        let user = UserRepo::get_user_by_id(&conn_lock, id)?
            .ok_or_else(|| {
                warn!(user_id = id, "Public user profile not found");
                CoreError::NotFound("error.user.not_found".into())
            })?;

        Ok(UserPublic {
            id: user.id,
            username: user.username,
            avatar: user.avatar,
        })
    }

    #[instrument(skip(state))]
    pub fn delete_avatar(state: &AppState, id: i32) -> CoreResult<()> {
        let conn = state.db.connection();
        let conn_lock = conn.lock()
            .map_err(|_| CoreError::Internal("error.system.db_lock".into()))?;

        UserRepo::update_avatar(&conn_lock, id, None, None)?;

        info!(user_id = id, "User avatar deleted successfully");
        Ok(())
    }
}