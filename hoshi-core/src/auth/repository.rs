use crate::error::CoreResult;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};

#[derive(Debug, Clone)]
pub struct Session {
    pub session_id: String,
    pub user_id: i32,
    pub expires_at: DateTime<Utc>,
}

pub struct AuthRepo;

impl AuthRepo {
    pub fn create_session(conn: &Connection, session: &Session) -> CoreResult<()> {
        conn.execute(
            "INSERT INTO Session (session_id, user_id, expires_at) VALUES (?, ?, ?)",
            params![
                session.session_id,
                session.user_id,
                session.expires_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get_session(conn: &Connection, session_id: &str) -> CoreResult<Option<Session>> {
        let result = conn.query_row(
            "SELECT session_id, user_id, expires_at FROM Session WHERE session_id = ?",
            [session_id],
            |row| {
                Ok(Session {
                    session_id: row.get(0)?,
                    user_id: row.get(1)?,
                    expires_at: row.get::<_, String>(2)?
                        .parse::<DateTime<Utc>>()
                        .map_err(|_| rusqlite::Error::InvalidQuery)?,
                })
            },
        ).optional()?;

        Ok(result)
    }

    pub fn delete_session(conn: &Connection, session_id: &str) -> CoreResult<()> {
        conn.execute("DELETE FROM Session WHERE session_id = ?", [session_id])?;
        Ok(())
    }

    pub fn cleanup_expired_sessions(conn: &Connection) -> CoreResult<()> {
        conn.execute(
            "DELETE FROM Session WHERE datetime(expires_at) < datetime('now')",
            [],
        )?;
        Ok(())
    }
}