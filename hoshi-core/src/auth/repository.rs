use crate::error::CoreResult;
use sqlx::SqlitePool;

pub struct AuthRepository;

impl AuthRepository {
    pub async fn set_active_user(pool: &SqlitePool, user_id: Option<i32>) -> CoreResult<()> {
        sqlx::query(
            "INSERT OR REPLACE INTO auth_state (id, active_user_id) VALUES (1, ?)",
        )
            .bind(user_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn get_active_user(pool: &SqlitePool) -> CoreResult<Option<i32>> {
        let row: Option<(Option<i32>,)> = sqlx::query_as(
            "SELECT active_user_id FROM auth_state WHERE id = 1",
        )
            .fetch_optional(pool)
            .await?;

        Ok(row.and_then(|(id,)| id))
    }
}