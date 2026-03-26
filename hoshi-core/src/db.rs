use crate::error::CoreResult;
use crate::paths::AppPaths;
use rusqlite::{Connection, OpenFlags};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct DatabaseManager {
    app_db: Arc<Mutex<Connection>>,
}

impl DatabaseManager {
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

    tracing::info!("Connected to database: {}", path.display());
    Ok(conn)
}

pub fn init_all_databases(paths: &AppPaths) -> CoreResult<()> {
    let conn = Connection::open(&paths.database_path)?;

    tracing::info!("Initializing unified database: {}", paths.database_path.display());

    execute_schema_file(&conn, include_str!("../schema/00_init.sql"), "00_init")?;
    execute_schema_file(&conn, include_str!("../schema/01_users.sql"), "01_users")?;
    execute_schema_file(&conn, include_str!("../schema/02_content_core.sql"), "02_content_core")?;
    execute_schema_file(&conn, include_str!("../schema/03_content_details.sql"), "03_content_details")?;
    execute_schema_file(&conn, include_str!("../schema/04_integrations.sql"), "04_integrations")?;
    execute_schema_file(&conn, include_str!("../schema/05_user_library.sql"), "05_user_library")?;
    execute_schema_file(&conn, include_str!("../schema/06_system.sql"), "06_system")?;

    tracing::info!("Database initialization completed successfully");

    Ok(())
}

fn execute_schema_file(conn: &Connection, sql: &str, name: &str) -> CoreResult<()> {
    tracing::debug!("Executing schema: {}", name);

    conn.execute_batch(sql).map_err(|e| {
        tracing::error!("Failed to execute schema {}: {}", name, e);
        e
    })?;

    tracing::debug!("Schema {} executed successfully", name);
    Ok(())
}