use crate::config::model::UserConfig;
use crate::error::CoreResult;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::Value;

pub struct ConfigRepo;

impl ConfigRepo {
    pub fn get_config(conn: &Connection, user_id: i32) -> CoreResult<UserConfig> {
        let result = conn
            .query_row(
                "SELECT config FROM UserConfig WHERE user_id = ?",
                [user_id],
                |row| row.get::<_, String>(0),
            )
            .optional()?;

        match result {
            Some(raw) => Ok(serde_json::from_str(&raw).unwrap_or_default()),
            None => Ok(UserConfig::default()),
        }
    }

    pub fn set_config(conn: &Connection, user_id: i32, config: &UserConfig) -> CoreResult<()> {
        let raw = serde_json::to_string(config)?;

        let now = Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO UserConfig (user_id, config, updated_at)
             VALUES (?, ?, ?)
             ON CONFLICT(user_id) DO UPDATE SET
                config = excluded.config,
                updated_at = excluded.updated_at",
            params![user_id, raw, now],
        )?;

        Ok(())
    }

    pub fn patch_config(
        conn: &Connection,
        user_id: i32,
        patch: &Value,
    ) -> CoreResult<UserConfig> {
        let current = Self::get_config(conn, user_id)?;
        let mut current_value = serde_json::to_value(&current)?;

        merge_json(&mut current_value, patch);

        let merged: UserConfig = serde_json::from_value(current_value).unwrap_or(current);

        Self::set_config(conn, user_id, &merged)?;

        Ok(merged)
    }
}

fn merge_json(base: &mut Value, patch: &Value) {
    match (base, patch) {
        (Value::Object(base_map), Value::Object(patch_map)) => {
            for (key, patch_val) in patch_map {
                let base_val = base_map.entry(key).or_insert(Value::Null);
                merge_json(base_val, patch_val);
            }
        }
        (base, patch) => {
            *base = patch.clone();
        }
    }
}