use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tower_http::cors::CorsLayer;
use axum::Router;

use crate::manager::{SharedManager, WatchPartyManager};
use crate::routes::{watchparty_guest_routes, new_token_store, TokenStore};
use crate::tunnel::TunnelManager;

pub struct WatchPartyServerState {
    cancel: RwLock<Option<CancellationToken>>,
    pub manager: Arc<WatchPartyManager>,
    pub tunnel: Arc<TunnelManager>,
    pub token_store: TokenStore,
}

impl WatchPartyServerState {
    pub fn new() -> Self {
        Self {
            cancel: RwLock::new(None),
            manager: Arc::new(WatchPartyManager::new()),
            tunnel: Arc::new(TunnelManager::new()),
            token_store: new_token_store(),
        }
    }

    pub async fn is_running(&self) -> bool {
        self.cancel.read().await.is_some()
    }
    pub async fn start(&self, port: u16, spa: Router) -> anyhow::Result<SocketAddr> {
        let mut cancel_guard = self.cancel.write().await;

        if cancel_guard.is_some() {
            return Ok(SocketAddr::from(([0, 0, 0, 0], port)));
        }

        let token = CancellationToken::new();
        let token_clone = token.clone();

        let manager: SharedManager = self.manager.clone();
        let tunnel = self.tunnel.clone();
        let token_store = self.token_store.clone();

        let app: Router = Router::new()
            .merge(watchparty_guest_routes(manager.clone(), token_store))
            .merge(spa)
            .layer(axum::Extension(tunnel))
            .layer(CorsLayer::permissive());

        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        let listener = tokio::net::TcpListener::bind(addr).await?;
        let bound_addr = listener.local_addr()?;

        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async move { token_clone.cancelled().await })
                .await
                .ok();
        });

        *cancel_guard = Some(token);
        Ok(bound_addr)
    }

    pub async fn stop(&self) {
        if let Some(token) = self.cancel.write().await.take() {
            token.cancel();
        }
        self.tunnel.force_close().await;
    }
}