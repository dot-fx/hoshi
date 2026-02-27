use crate::error::{CoreError, CoreResult};
use crate::state::AppState;
use crate::users::repository::UserRepo;
use bcrypt::{hash, verify, DEFAULT_COST};
use serde::{Deserialize, Serialize};


#[derive(Serialize)]
pub struct UserPublic {
    pub id: i32,
    pub username: String,
    pub avatar: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPrivate {
    pub id: i32,
    pub username: String,
    pub avatar: Option<String>,
    pub has_password: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserBody {
    pub username: String,
    pub profile_picture_url: Option<String>,
    pub password: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserBody {
    pub username: Option<String>,
    pub profile_picture_url: Option<String>,
    pub password: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub id: i32,
    pub username: String,
    pub profile_picture_url: Option<String>,
    pub has_password: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangePasswordBody {
    pub current_password: Option<String>,
    pub new_password: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteUserBody {
    pub password: Option<String>,
}

pub struct UserService;

impl UserService {

    pub fn get_all_users(state: &AppState) -> CoreResult<Vec<UserResponse>> {
        let db = &state.db;
        let conn = db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("Database lock failed".into()))?;
        UserRepo::get_all_users(&conn_lock)
    }

    pub fn get_me(state: &AppState, user_id: i32) -> CoreResult<UserPrivate> {
        let conn = state.db.connection();
        let conn_lock = conn.lock()
            .map_err(|_| CoreError::Internal("Database lock failed".into()))?;

        let user = UserRepo::get_user_by_id(&conn_lock, user_id)?
            .ok_or_else(|| CoreError::NotFound("User not found".into()))?;

        Ok(UserPrivate {
            id: user.id,
            username: user.username,
            avatar: user.avatar,
            has_password: user.password_hash.is_some(),
        })
    }

    pub fn get_user_public(state: &AppState, id: i32) -> CoreResult<UserPublic> {
        let conn = state.db.connection();
        let conn_lock = conn.lock()
            .map_err(|_| CoreError::Internal("Database lock failed".into()))?;

        let user = UserRepo::get_user_by_id(&conn_lock, id)?
            .ok_or_else(|| CoreError::NotFound("User not found".into()))?;

        Ok(UserPublic {
            id: user.id,
            username: user.username,
            avatar: user.avatar,
        })
    }

    pub fn update_user(state: &AppState, id: i32, updates: UpdateUserBody) -> CoreResult<()> {
        let password_hash_update = if let Some(password) = &updates.password {
            if password.is_empty() {
                Some(None)
            } else {
                let h = hash(password.trim(), DEFAULT_COST)
                    .map_err(|_| CoreError::Internal("Hashing failed".into()))?;
                Some(Some(h))
            }
        } else {
            None
        };

        let db = &state.db;
        let conn = db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("Database lock failed".into()))?;

        if updates.username.is_none() && updates.profile_picture_url.is_none() && password_hash_update.is_none() {
            return Err(CoreError::BadRequest("No update fields provided".into()));
        }

        let changes = UserRepo::update_user(&conn_lock, id, &updates, password_hash_update)?;

        if changes > 0 {
            Ok(())
        } else {
            Err(CoreError::NotFound("User not found or nothing to update".into()))
        }
    }

    pub fn delete_user(state: &AppState, id: i32, body: DeleteUserBody) -> CoreResult<()> {
        let conn = state.db.connection();
        let conn_lock = conn
            .lock()
            .map_err(|_| CoreError::Internal("Database lock failed".into()))?;

        let (_, password_hash) =
            UserRepo::get_user_credentials(&conn_lock, id)?
                .ok_or_else(|| CoreError::NotFound("User not found".into()))?;

        if let Some(hash_str) = password_hash {
            let pass = body.password
                .ok_or_else(|| CoreError::AuthError("Password required".into()))?;

            if !verify(&pass, &hash_str).unwrap_or(false) {
                return Err(CoreError::AuthError("Incorrect password".into()));
            }
        }

        let success = UserRepo::delete_user(&conn_lock, id)?;

        if success {
            Ok(())
        } else {
            Err(CoreError::Internal("Failed to delete user".into()))
        }
    }

    pub fn change_password(state: &AppState, id: i32, body: ChangePasswordBody) -> CoreResult<bool> {
        let conn = state.db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("Database lock failed".into()))?;

        let password_hash = UserRepo::get_password_hash(&conn_lock, id)?;

        if let Some(hash_str) = password_hash {
            if let Some(curr_pass) = &body.current_password {
                if !verify(curr_pass, &hash_str).unwrap_or(false) {
                    return Err(CoreError::AuthError("Current password is incorrect".into()));
                }
            } else {
                return Err(CoreError::AuthError("Current password required".into()));
            }
        }

        let has_new_password = body.new_password.is_some();

        let new_hash = match &body.new_password {
            Some(pass) if !pass.is_empty() => {
                Some(hash(pass.trim(), DEFAULT_COST)
                    .map_err(|_| CoreError::Internal("Hashing failed".into()))?)
            }
            _ => None
        };

        UserRepo::update_password(&conn_lock, id, new_hash)?;

        Ok(has_new_password)
    }
}