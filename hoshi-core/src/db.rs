use crate::error::CoreResult;
use crate::paths::AppPaths;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::str::FromStr;
use tracing::{debug, error, info, instrument};

pub struct DatabaseManager {
    pool: SqlitePool,
}

impl DatabaseManager {
    #[instrument(skip(paths))]
    pub async fn new(paths: &AppPaths) -> CoreResult<Self> {
        let url = format!("sqlite:{}", paths.database_path.display());

        let options = SqliteConnectOptions::from_str(&url)?
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .foreign_keys(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(8)
            .connect_with(options)
            .await?;

        info!(path = %paths.database_path.display(), "Connected to SQLite database (sqlx pool)");
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

#[instrument(skip(paths))]
pub async fn init_all_databases(paths: &AppPaths) -> CoreResult<()> {
    let url = format!("sqlite:{}", paths.database_path.display());

    let options = SqliteConnectOptions::from_str(&url)?.create_if_missing(true);
    let pool = SqlitePoolOptions::new().max_connections(1).connect_with(options).await?;

    info!(path = %paths.database_path.display(), "Running database schema migrations");

    let schemas: &[(&str, &str)] = &[
        ("00_init",            include_str!("../schema/00_init.sql")),
        ("01_users",           include_str!("../schema/01_users.sql")),
        ("02_content_core",    include_str!("../schema/02_content_core.sql")),
        ("03_content_details", include_str!("../schema/03_content_details.sql")),
        ("04_integrations",    include_str!("../schema/04_integrations.sql")),
        ("05_user_library",    include_str!("../schema/05_user_library.sql")),
        ("06_system",          include_str!("../schema/06_system.sql")),
    ];

    for (name, sql) in schemas {
        execute_schema(&pool, sql, name).await?;
    }

    info!("All database schemas applied successfully");
    pool.close().await;
    Ok(())
}

async fn execute_schema(pool: &SqlitePool, sql: &str, name: &str) -> CoreResult<()> {
    debug!(schema = %name, "Executing SQL schema batch");

    for statement in sql.split(';') {
        let trimmed = statement.trim();
        if trimmed.is_empty() { continue; }
        sqlx::query(trimmed)
            .execute(pool)
            .await
            .map_err(|e| {
                error!(schema = %name, error = ?e, statement = %trimmed, "Schema execution failed");
                e
            })?;
    }

    Ok(())
}