use chrono::Utc;
use serde_json::Value;
use sqlx::SqlitePool;
use tracing::{debug, instrument};

use crate::error::CoreResult;

pub struct CacheRepository;

impl CacheRepository {
    #[instrument(skip(pool, data))]
    pub async fn set(
        pool: &SqlitePool,
        key: &str,
        source: &str,
        query_type: &str,
        data: &Value,
        ttl_seconds: i64,
    ) -> CoreResult<()> {
        let now = Utc::now().timestamp();
        let expires_at = now + ttl_seconds;

        debug!(key = %key, source = %source, "Updating cache entry");

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO cache_metadata (key, source, query_type, data, created_at, expires_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
            .bind(key)
            .bind(source)
            .bind(query_type)
            .bind(data.to_string())
            .bind(now)
            .bind(expires_at)
            .execute(pool)
            .await?;

        Ok(())
    }

    #[instrument(skip(pool))]
    pub async fn get(pool: &SqlitePool, key: &str) -> CoreResult<Option<Value>> {
        let now = Utc::now().timestamp();

        let row: Option<(String,)> = sqlx::query_as(
            "SELECT data FROM cache_metadata WHERE key = ? AND expires_at > ?",
        )
            .bind(key)
            .bind(now)
            .fetch_optional(pool)
            .await?;

        let result = row.map(|(data_str,)| {
            serde_json::from_str(&data_str).unwrap_or(Value::Null)
        });

        if result.is_some() {
            debug!(key = %key, "Cache hit");
        } else {
            debug!(key = %key, "Cache miss or expired");
        }

        Ok(result)
    }

    #[instrument(skip(pool))]
    pub async fn delete(pool: &SqlitePool, key: &str) -> CoreResult<()> {
        debug!(key = %key, "Deleting cache entry");
        sqlx::query("DELETE FROM cache_metadata WHERE key = ?")
            .bind(key)
            .execute(pool)
            .await?;
        Ok(())
    }

    #[instrument(skip(pool))]
    pub async fn cleanup(pool: &SqlitePool) -> CoreResult<()> {
        let now = Utc::now().timestamp();
        let result = sqlx::query("DELETE FROM cache_metadata WHERE expires_at <= ?")
            .bind(now)
            .execute(pool)
            .await?;

        let count = result.rows_affected();
        if count > 0 {
            debug!(deleted_count = count, "Expired cache entries cleaned up");
        }
        Ok(())
    }
}