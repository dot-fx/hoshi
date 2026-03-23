use crate::error::CoreResult;
use rusqlite::{params, Connection, OptionalExtension};

pub struct AuthRepo;

impl AuthRepo {
    pub fn set_active_user(conn: &Connection, user_id: Option<i32>) -> CoreResult<()> {
        conn.execute(
            "INSERT OR REPLACE INTO auth_state (id, active_user_id) VALUES (1, ?)",
            params![user_id],
        )?;
        Ok(())
    }

    pub fn get_active_user(conn: &Connection) -> CoreResult<Option<i32>> {
        let result = conn.query_row(
            "SELECT active_user_id FROM auth_state WHERE id = 1",
            [],
            |row| row.get::<_, Option<i32>>(0),
        ).optional()?;

        Ok(result.flatten())
    }
}