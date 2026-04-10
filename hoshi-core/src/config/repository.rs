use chrono::Utc;
use serde_json::Value;
use sqlx::SqlitePool;

use crate::config::model::UserConfig;
use crate::error::CoreResult;

pub struct ConfigRepository;

impl ConfigRepository {
    pub async fn get_config(pool: &SqlitePool, user_id: i32) -> CoreResult<UserConfig> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT config FROM UserConfig WHERE user_id = ?",
        )
            .bind(user_id)
            .fetch_optional(pool)
            .await?;

        Ok(match row {
            Some((raw,)) => serde_json::from_str(&raw).unwrap_or_default(),
            None => UserConfig::default(),
        })
    }

    pub async fn set_config(pool: &SqlitePool, user_id: i32, config: &UserConfig) -> CoreResult<()> {
        let raw = serde_json::to_string(config)?;
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO UserConfig (user_id, config, updated_at)
             VALUES (?, ?, ?)
             ON CONFLICT(user_id) DO UPDATE SET
                config     = excluded.config,
                updated_at = excluded.updated_at",
        )
            .bind(user_id)
            .bind(&raw)
            .bind(&now)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn patch_config(pool: &SqlitePool, user_id: i32, patch: &Value) -> CoreResult<UserConfig> {
        let current = Self::get_config(pool, user_id).await?;
        let mut current_value = serde_json::to_value(&current)?;
        merge_json(&mut current_value, patch);
        let merged: UserConfig = serde_json::from_value(current_value).unwrap_or(current);
        Self::set_config(pool, user_id, &merged).await?;
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
        (base, patch) => *base = patch.clone(),
    }
}