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
    pub expires_at: i64,
    pub sync_enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddIntegrationRequest {
    pub tracker_name: String,
    pub access_token: String,
}

pub struct IntegrationCredentials {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub tracker_user_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackerMapping {
    pub cid: String,
    pub tracker_name: String,
    pub tracker_id: String,
    pub tracker_url: Option<String>,
    pub sync_enabled: bool,
    pub last_synced: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

pub struct TrackerRepository;

impl TrackerRepository {

    pub fn save_integration(
        conn: &Connection,
        user_id: i32,
        tracker_name: &str,
        tracker_user_id: &str,
        access_token: &str,
        refresh_token: Option<&str>,
        token_type: &str,
        expires_at: i64,
    ) -> CoreResult<()> {
        let sql = r#"
            INSERT INTO UserIntegration
            (user_id, tracker_name, tracker_user_id, access_token, refresh_token, token_type, expires_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(user_id, tracker_name) DO UPDATE SET
                tracker_user_id = excluded.tracker_user_id,
                access_token    = excluded.access_token,
                refresh_token   = excluded.refresh_token,
                token_type      = excluded.token_type,
                expires_at      = excluded.expires_at,
                updated_at      = strftime('%s', 'now')
        "#;

        conn.execute(
            sql,
            params![
                user_id,
                tracker_name,
                tracker_user_id,
                access_token,
                refresh_token,
                token_type,
                expires_at,
            ],
        )?;

        Ok(())
    }

    pub fn get_credentials(
        conn: &Connection,
        user_id: i32,
        tracker_name: &str,
    ) -> CoreResult<Option<IntegrationCredentials>> {
        let res = conn.query_row(
            "SELECT access_token, refresh_token, tracker_user_id
             FROM UserIntegration
             WHERE user_id = ?1 AND tracker_name = ?2",
            params![user_id, tracker_name],
            |row| {
                Ok(IntegrationCredentials {
                    access_token: row.get(0)?,
                    refresh_token: row.get(1)?,
                    tracker_user_id: row.get(2)?,
                })
            },
        )
            .optional()?;

        Ok(res)
    }

    pub fn delete_integration(
        conn: &Connection,
        user_id: i32,
        tracker_name: &str,
    ) -> CoreResult<bool> {
        let count = conn.execute(
            "DELETE FROM UserIntegration WHERE user_id = ?1 AND tracker_name = ?2",
            params![user_id, tracker_name],
        )?;
        Ok(count > 0)
    }

    pub fn get_user_integrations(
        conn: &Connection,
        user_id: i32,
    ) -> CoreResult<Vec<TrackerIntegration>> {
        let mut stmt =
            conn.prepare("SELECT * FROM UserIntegration WHERE user_id = ?1")?;
        let rows = stmt.query_map(params![user_id], |row| {
            Ok(TrackerIntegration {
                user_id: row.get("user_id")?,
                tracker_name: row.get("tracker_name")?,
                tracker_user_id: row.get("tracker_user_id")?,
                access_token: row.get("access_token")?,
                refresh_token: row.get("refresh_token")?,
                token_type: row.get("token_type")?,
                expires_at: row.get("expires_at")?,
                sync_enabled: row.get::<_, i32>("sync_enabled")? == 1,
                created_at: {
                    match row.get::<_, i64>("created_at") {
                        Ok(v) => v,
                        Err(_) => row.get::<_, String>("created_at")?
                            .parse::<i64>().unwrap_or(0),
                    }
                },
                updated_at: {
                    match row.get::<_, i64>("updated_at") {
                        Ok(v) => v,
                        Err(_) => row.get::<_, String>("updated_at")?
                            .parse::<i64>().unwrap_or(0),
                    }
                },
            })
        })?;

        let mut result = Vec::new();
        for r in rows {
            result.push(r?);
        }
        Ok(result)
    }

    pub fn set_sync_enabled(
        conn: &Connection,
        user_id: i32,
        tracker_name: &str,
        enabled: bool,
    ) -> CoreResult<()> {
        conn.execute(
            "UPDATE UserIntegration
             SET sync_enabled = ?1, updated_at = strftime('%s', 'now')
             WHERE user_id = ?2 AND tracker_name = ?3",
            params![if enabled { 1 } else { 0 }, user_id, tracker_name],
        )?;
        Ok(())
    }
    
    pub fn add_mapping(conn: &Connection, mapping: &TrackerMapping) -> CoreResult<()> {
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            r#"
            INSERT OR REPLACE INTO tracker_mappings
            (cid, tracker_name, tracker_id, tracker_url, sync_enabled, last_synced, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                mapping.cid,
                mapping.tracker_name,
                mapping.tracker_id,
                mapping.tracker_url,
                if mapping.sync_enabled { 1 } else { 0 },
                mapping.last_synced,
                mapping.created_at,
                now,
            ],
        )?;
        Ok(())
    }

    pub fn update_mapping_id(
        conn: &Connection,
        cid: &str,
        tracker_name: &str,
        new_id: &str,
    ) -> CoreResult<usize> {
        let now = chrono::Utc::now().timestamp();
        let count = conn.execute(
            "UPDATE tracker_mappings
             SET tracker_id = ?1, updated_at = ?2
             WHERE cid = ?3 AND tracker_name = ?4",
            params![new_id, now, cid, tracker_name],
        )?;
        Ok(count)
    }

    pub fn delete_mapping(
        conn: &Connection,
        cid: &str,
        tracker_name: &str,
    ) -> CoreResult<usize> {
        let count = conn.execute(
            "DELETE FROM tracker_mappings WHERE cid = ?1 AND tracker_name = ?2",
            params![cid, tracker_name],
        )?;
        Ok(count)
    }

    pub fn get_mappings_by_cid(
        conn: &Connection,
        cid: &str,
    ) -> CoreResult<Vec<TrackerMapping>> {
        let mut stmt =
            conn.prepare("SELECT * FROM tracker_mappings WHERE cid = ?1")?;
        let rows = stmt.query_map(params![cid], |row| {
            Ok(TrackerMapping {
                cid: row.get(0)?,
                tracker_name: row.get(1)?,
                tracker_id: row.get(2)?,
                tracker_url: row.get(3)?,
                sync_enabled: row.get::<_, i32>(4)? == 1,
                last_synced: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn find_cid_by_tracker(
        conn: &Connection,
        tracker_name: &str,
        tracker_id: &str,
    ) -> CoreResult<Option<String>> {
        conn.query_row(
            "SELECT cid FROM tracker_mappings WHERE tracker_name = ?1 AND tracker_id = ?2",
            params![tracker_name, tracker_id],
            |row| row.get(0),
        )
            .optional()
            .map_err(Into::into)
    }

    pub fn has_canonical_mapping(conn: &Connection, cid: &str) -> CoreResult<bool> {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM tracker_mappings WHERE cid = ?1",
            params![cid],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }
}