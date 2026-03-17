use std::net::SocketAddr;
use std::sync::Arc;
use anyhow::Result;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use hoshi_core::{build_app_state, build_app_state_with_headless, paths::AppPaths};

mod router;
mod assets;
mod middleware;
mod error;
mod headless;
mod api;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "hoshi_server=debug,hoshi_core=debug,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Hoshi Server...");

    let base_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("hoshi");

    let paths = AppPaths::from_base(base_dir);

    let state = match headless::AxumHeadless::new().await {
        Ok(browser) => {
            tracing::info!("Chromium headless initialized correctamente.");
            let headless_arc = Arc::new(browser);
            build_app_state_with_headless(paths, headless_arc).await?
        }
        Err(e) => {
            tracing::warn!("Chromium error {}. Fallback to noopheadless", e);
            build_app_state(paths).await?
        }
    };

    let app = router::build_router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 10090));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}