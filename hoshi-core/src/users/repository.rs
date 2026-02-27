use crate::error::{CoreError, CoreResult};
use crate::users::service::{UpdateUserBody, UserResponse};
use rusqlite::{params, Connection, OptionalExtension};

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

    pub fn get_user_by_id(conn: &Connection, id: i32) -> CoreResult<Option<UserModel>> {
        let user = conn.query_row(
            "SELECT id, username, profile_picture_url, password_hash FROM User WHERE id = ?",
            [id],
            |row| {
                Ok(UserModel {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    avatar: row.get(2)?,
                    password_hash: row.get(3)?,
                })
            },
        )
        .optional()?;

        Ok(user)
    }

    pub fn find_auth_data_by_id(conn: &Connection, user_id: i32) -> CoreResult<Option<UserAuthData>> {
        let result = conn.query_row(
            "SELECT username, profile_picture_url, password_hash FROM User WHERE id = ?",
            [user_id],
            |row| {
                Ok(UserAuthData {
                    username: row.get(0)?,
                    avatar: row.get(1)?,
                    password_hash: row.get(2)?,
                })
            },
        )
            .optional()?;

        Ok(result)
    }

    pub fn delete_user(conn: &Connection, id: i32) -> CoreResult<bool> {
        let changes = conn.execute("DELETE FROM User WHERE id = ?", [id])?;
        Ok(changes > 0)
    }

    pub fn get_all_users(conn: &Connection) -> CoreResult<Vec<UserResponse>> {
        let mut stmt = conn.prepare("
            SELECT id, username, profile_picture_url,
            CASE WHEN password_hash IS NOT NULL THEN 1 ELSE 0 END as has_password
            FROM User ORDER BY id
        ")?;

        let users_iter = stmt.query_map([], |row| {
            Ok(UserResponse {
                id: row.get(0)?,
                username: row.get(1)?,
                profile_picture_url: row.get(2)?,
                has_password: row.get::<_, i32>(3)? == 1,
            })
        })?;

        let mut users = Vec::new();
        for user in users_iter {
            users.push(user?);
        }
        Ok(users)
    }

    pub fn create_user(
        conn: &Connection,
        username: &str,
        avatar: Option<String>,
        password_hash: Option<String>
    ) -> CoreResult<i64> {
        let res = conn.execute(
            "INSERT INTO User (username, profile_picture_url, password_hash) VALUES (?, ?, ?)",
            params![username, avatar, password_hash],
        );

        match res {
            Ok(_) => Ok(conn.last_insert_rowid()),
            Err(rusqlite::Error::SqliteFailure(_, Some(msg))) if msg.contains("UNIQUE") => {
                Err(CoreError::Internal("Username already exists".into())) // El Service/Route traducirá a Conflict si es necesario
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

        if let Some(avatar) = &updates.profile_picture_url {
            fields.push("profile_picture_url = ?");
            params.push(Box::new(avatar));
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

        conn.execute(&query, params_refs.as_slice()).map_err(|e| {
            if e.to_string().contains("UNIQUE") {
                CoreError::Internal("Username already exists".into())
            } else {
                CoreError::Database(e)
            }
        })
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
}