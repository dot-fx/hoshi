use base64::Engine;
use crate::error::{CoreError, CoreResult};
use crate::users::service::{UpdateUserBody, UserResponse};
use rusqlite::{params, Connection, OptionalExtension};
use tracing::{debug, instrument};

pub struct UserAuthData {
    pub username: String,
    pub avatar: Option<String>,
    pub password_hash: Option<String>,
}

pub struct UserModel {
    pub id: i32,
    pub username: String,
    pub avatar: Option<String>,
    pub password_hash: Option<String>,
}

pub struct UserRepo;

impl UserRepo {

    #[instrument(skip(conn))]
    pub fn get_user_by_id(conn: &Connection, id: i32) -> CoreResult<Option<UserModel>> {
        debug!(id = id, "Fetching user by ID");
        let user = conn.query_row(
            "SELECT id, username, avatar_data, avatar_mime, password_hash FROM User WHERE id = ?",
            [id],
            |row| {
                let avatar_data: Option<Vec<u8>> = row.get(2)?;
                let avatar_mime: Option<String> = row.get(3)?;
                let avatar = match (avatar_data, avatar_mime) {
                    (Some(data), Some(mime)) => Some(format!(
                        "data:{};base64,{}",
                        mime,
                        base64::engine::general_purpose::STANDARD.encode(&data)
                    )),
                    _ => None,
                };
                Ok(UserModel {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    avatar,
                    password_hash: row.get(4)?,
                })
            },
        ).optional()?;
        Ok(user)
    }

    pub fn find_auth_data_by_id(conn: &Connection, user_id: i32) -> CoreResult<Option<UserAuthData>> {
        let result = conn.query_row(
            "SELECT username, avatar_data, avatar_mime, password_hash FROM User WHERE id = ?",
            [user_id],
            |row| {
                let avatar_data: Option<Vec<u8>> = row.get(1)?;
                let avatar_mime: Option<String> = row.get(2)?;
                let avatar = match (avatar_data, avatar_mime) {
                    (Some(data), Some(mime)) => Some(format!(
                        "data:{};base64,{}",
                        mime,
                        base64::engine::general_purpose::STANDARD.encode(&data)
                    )),
                    _ => None,
                };
                Ok(UserAuthData {
                    username: row.get(0)?,
                    avatar,
                    password_hash: row.get(3)?,
                })
            },
        ).optional()?;
        Ok(result)
    }
    pub fn delete_user(conn: &Connection, id: i32) -> CoreResult<bool> {
        let changes = conn.execute("DELETE FROM User WHERE id = ?", [id])?;
        Ok(changes > 0)
    }

    pub fn get_all_users(conn: &Connection) -> CoreResult<Vec<UserResponse>> {
        let mut stmt = conn.prepare("
        SELECT id, username, avatar_data, avatar_mime,
        CASE WHEN password_hash IS NOT NULL THEN 1 ELSE 0 END as has_password
        FROM User ORDER BY id
    ")?;

        let users_iter = stmt.query_map([], |row| {
            let avatar_data: Option<Vec<u8>> = row.get(2)?;
            let avatar_mime: Option<String> = row.get(3)?;
            let avatar = match (avatar_data, avatar_mime) {
                (Some(data), Some(mime)) => Some(format!(
                    "data:{};base64,{}",
                    mime,
                    base64::engine::general_purpose::STANDARD.encode(&data)
                )),
                _ => None,
            };
            Ok(UserResponse {
                id: row.get(0)?,
                username: row.get(1)?,
                avatar,
                has_password: row.get::<_, i32>(4)? == 1,
            })
        })?;

        let mut users = Vec::new();
        for user in users_iter {
            users.push(user?);
        }
        Ok(users)
    }

    #[instrument(skip(conn))]
    pub fn create_user(
        conn: &Connection,
        username: &str,
        password_hash: Option<String>
    ) -> CoreResult<i64> {
        let res = conn.execute(
            "INSERT INTO User (username, password_hash) VALUES (?, ?)",
            params![username, password_hash],
        );

        match res {
            Ok(_) => {
                let id = conn.last_insert_rowid();
                debug!(id = id, "User record created");
                Ok(id)
            },
            Err(rusqlite::Error::SqliteFailure(_, Some(msg))) if msg.contains("UNIQUE") => {
                Err(CoreError::Internal("error.user.already_exists".into()))
            },
            Err(e) => Err(CoreError::Database(e)),
        }
    }

    pub fn update_user(conn: &Connection, id: i32, updates: &UpdateUserBody, new_password_hash: Option<Option<String>>) -> CoreResult<usize> {
        let mut query = "UPDATE User SET ".to_string();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        let mut fields = Vec::new();

        if let Some(username) = &updates.username {
            fields.push("username = ?");
            params.push(Box::new(username));
        }
        
        if let Some(ph_opt) = new_password_hash {
            fields.push("password_hash = ?");
            params.push(Box::new(ph_opt));
        }

        if fields.is_empty() {
            return Ok(0);
        }

        query.push_str(&fields.join(", "));
        query.push_str(" WHERE id = ?");
        params.push(Box::new(id));

        let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        conn.execute(&query, params_refs.as_slice()).map_err(CoreError::Database)
    }

    pub fn get_user_credentials(conn: &Connection, id: i32) -> CoreResult<Option<(String, Option<String>)>> {
        let res = conn.query_row(
            "SELECT username, password_hash FROM User WHERE id = ?",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).optional()?;
        Ok(res)
    }

    pub fn get_password_hash(conn: &Connection, id: i32) -> CoreResult<Option<String>> {
        let res: Option<String> = conn.query_row(
            "SELECT password_hash FROM User WHERE id = ?",
            [id],
            |row| row.get(0),
        ).optional()?.flatten();
        Ok(res)
    }

    pub fn update_password(conn: &Connection, id: i32, new_hash: Option<String>) -> CoreResult<()> {
        conn.execute("UPDATE User SET password_hash = ? WHERE id = ?", params![new_hash, id])?;
        Ok(())
    }

    pub fn update_avatar(conn: &Connection, id: i32, data: Option<Vec<u8>>, mime: Option<String>) -> CoreResult<()> {
        conn.execute(
            "UPDATE User SET avatar_data = ?, avatar_mime = ? WHERE id = ?",
            params![data, mime, id],
        )?;
        Ok(())
    }

    pub fn get_avatar(conn: &Connection, id: i32) -> CoreResult<Option<(Vec<u8>, String)>> {
        let res = conn.query_row(
            "SELECT avatar_data, avatar_mime FROM User WHERE id = ? AND avatar_data IS NOT NULL",
            [id],
            |row| Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, String>(1)?)),
        ).optional()?;
        Ok(res)
    }
}