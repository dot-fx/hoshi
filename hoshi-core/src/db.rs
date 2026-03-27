use crate::error::{CoreResult};
use crate::paths::AppPaths;
use rusqlite::{Connection, OpenFlags};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::{info, debug, error, instrument};

pub struct DatabaseManager {
    app_db: Arc<Mutex<Connection>>,
}

impl DatabaseManager {
    #[instrument(skip(paths))]
    pub fn new(paths: &AppPaths) -> CoreResult<Self> {
        let conn = open_database(&paths.database_path)?;

        Ok(Self {
            app_db: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn connection(&self) -> Arc<Mutex<Connection>> {
        Arc::clone(&self.app_db)
    }
}

fn open_database(path: &PathBuf) -> CoreResult<Connection> {
    let conn = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
    )?;

    info!(path = %path.display(), "Connected to SQLite database");
    Ok(conn)
}

#[instrument(skip(paths))]
pub fn init_all_databases(paths: &AppPaths) -> CoreResult<()> {
    let conn = Connection::open(&paths.database_path)?;

    info!(path = %paths.database_path.display(), "Running database schema migrations");

    execute_schema_file(&conn, include_str!("../schema/00_init.sql"), "00_init")?;
    execute_schema_file(&conn, include_str!("../schema/01_users.sql"), "01_users")?;
    execute_schema_file(&conn, include_str!("../schema/02_content_core.sql"), "02_content_core")?;
    execute_schema_file(&conn, include_str!("../schema/03_content_details.sql"), "03_content_details")?;
    execute_schema_file(&conn, include_str!("../schema/04_integrations.sql"), "04_integrations")?;
    execute_schema_file(&conn, include_str!("../schema/05_user_library.sql"), "05_user_library")?;
    execute_schema_file(&conn, include_str!("../schema/06_system.sql"), "06_system")?;

    info!("All database schemas applied successfully");
    Ok(())
}

fn execute_schema_file(conn: &Connection, sql: &str, name: &str) -> CoreResult<()> {
    debug!(schema = %name, "Executing SQL schema batch");

    conn.execute_batch(sql).map_err(|e| {
        error!(schema = %name, error = ?e, "Critical failure during schema execution");
        e
    })?;

    Ok(())
}