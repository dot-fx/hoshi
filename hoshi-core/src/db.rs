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
    execute_schema_file(&conn, include_str!("../schema/02_core_metadata.sql"), "02_core_metadata")?;
    execute_schema_file(&conn, include_str!("../schema/03_content_units.sql"), "03_content_units")?;
    execute_schema_file(&conn, include_str!("../schema/04_trackers.sql"), "04_trackers")?;
    execute_schema_file(&conn, include_str!("../schema/05_extensions.sql"), "05_extensions")?;
    execute_schema_file(&conn, include_str!("../schema/06_relations.sql"), "06_relations")?;
    execute_schema_file(&conn, include_str!("../schema/07_tags.sql"), "07_tags")?;
    execute_schema_file(&conn, include_str!("../schema/08_list_entries.sql"), "08_list_entries")?;
    execute_schema_file(&conn, include_str!("../schema/09_boorus.sql"), "09_boorus")?;
    execute_schema_file(&conn, include_str!("../schema/10_cache.sql"), "10_cache")?;
    execute_schema_file(&conn, include_str!("../schema/11_schedule.sql"), "11_schedule")?;

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