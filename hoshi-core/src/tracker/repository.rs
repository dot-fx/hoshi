use crate::error::CoreResult;
use rusqlite::{params, Connection, OptionalExtension};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TrackerIntegration {
    pub user_id: i32,
    pub tracker_name: String,
    pub tracker_user_id: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_at: String,
    pub sync_enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

pub struct IntegrationCredentials {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub tracker_user_id: String,
}

pub struct TrackerRepo;

impl TrackerRepo {
    pub fn save_integration(
        conn: &Connection,
        user_id: i32,
        tracker_name: &str,
        tracker_user_id: &str,
        access_token: &str,
        refresh_token: Option<&str>,
        token_type: &str,
        expires_at: &str
    ) -> CoreResult<()> {
        let sql = r#"
            INSERT INTO UserIntegration
            (user_id, tracker_name, tracker_user_id, access_token, refresh_token, token_type, expires_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(user_id, tracker_name) DO UPDATE SET
                tracker_user_id = excluded.tracker_user_id,
                access_token = excluded.access_token,
                refresh_token = excluded.refresh_token,
                token_type = excluded.token_type,
                expires_at = excluded.expires_at,
                updated_at = CURRENT_TIMESTAMP
        "#;

        conn.execute(
            sql,
            params![
                user_id,
                tracker_name,
                tracker_user_id,
                access_token,
                refresh_token.unwrap_or(""),
                token_type,
                expires_at
            ],
        )?;

        Ok(())
    }

    pub fn get_credentials(
        conn: &Connection,
        user_id: i32,
        tracker_name: &str
    ) -> CoreResult<Option<IntegrationCredentials>> {
        let res = conn.query_row(
            "SELECT access_token, refresh_token, tracker_user_id FROM UserIntegration WHERE user_id = ? AND tracker_name = ?",
            params![user_id, tracker_name],
            |row| {
                Ok(IntegrationCredentials {
                    access_token: row.get(0)?,
                    refresh_token: row.get(1)?,
                    tracker_user_id: row.get(2)?,
                })
            }
        ).optional()?;

        Ok(res)
    }

    pub fn delete_integration(conn: &Connection, user_id: i32, tracker_name: &str) -> CoreResult<bool> {
        let count = conn.execute(
            "DELETE FROM UserIntegration WHERE user_id = ? AND tracker_name = ?",
            params![user_id, tracker_name]
        )?;
        Ok(count > 0)
    }

    pub fn get_user_integrations(conn: &Connection, user_id: i32) -> CoreResult<Vec<TrackerIntegration>> {
        let mut stmt = conn.prepare("SELECT * FROM UserIntegration WHERE user_id = ?")?;
        let rows = stmt.query_map([user_id], |row| {
            Ok(TrackerIntegration {
                user_id: row.get("user_id")?,
                tracker_name: row.get("tracker_name")?,
                tracker_user_id: row.get("tracker_user_id")?,
                access_token: row.get("access_token")?,
                refresh_token: row.get("refresh_token").ok(),
                token_type: row.get("token_type")?,
                expires_at: row.get("expires_at")?,
                sync_enabled: row.get::<_, i32>("sync_enabled")? == 1,
                created_at: row.get("created_at")?,
                updated_at: row.get("updated_at")?,
            })
        })?;

        let mut result = Vec::new();
        for r in rows { result.push(r?); }
        Ok(result)
    }

    pub fn set_sync_enabled(conn: &Connection, user_id: i32, tracker_name: &str, enabled: bool) -> CoreResult<()> {
        conn.execute(
            "UPDATE UserIntegration SET sync_enabled = ? WHERE user_id = ? AND tracker_name = ?",
            params![if enabled { 1 } else { 0 }, user_id, tracker_name]
        )?;
        Ok(())
    }
}