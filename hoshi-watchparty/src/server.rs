use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use axum::{
    body::Body,
    extract::Query,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Router,
};
use hoshi_core::proxy::{ProxyBody, ProxyQuery, ProxyService};

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

    /// `spa_dir` es el directorio del frontend compilado.
    /// El servidor sirve los assets estáticos desde ahí y hace fallback
    /// a index.html para las rutas del SPA.
    pub async fn start(&self, port: u16, spa_dir: PathBuf) -> anyhow::Result<SocketAddr> {
        let mut cancel_guard = self.cancel.write().await;

        if cancel_guard.is_some() {
            return Ok(SocketAddr::from(([0, 0, 0, 0], port)));
        }

        let token = CancellationToken::new();
        let token_clone = token.clone();

        let manager: SharedManager = self.manager.clone();
        let tunnel = self.tunnel.clone();
        let token_store = self.token_store.clone();

        let spa = Router::new().fallback_service(
            ServeDir::new(&spa_dir)
                .fallback(ServeFile::new(spa_dir.join("index.html")))
        );

        let app: Router = Router::new()
            .merge(watchparty_guest_routes(manager.clone(), token_store))
            .merge(proxy_routes())
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
            println!("[WatchParty] Server stopped");
        });

        println!("[WatchParty] Listening on {bound_addr}");
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

fn proxy_routes() -> Router {
    Router::new()
        .route("/api/proxy", axum::routing::get(handle_proxy).options(handle_proxy_options))
}

async fn handle_proxy_options() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert("Access-Control-Allow-Origin", HeaderValue::from_static("*"));
    headers.insert("Access-Control-Allow-Methods", HeaderValue::from_static("GET, OPTIONS"));
    headers.insert("Access-Control-Allow-Headers", HeaderValue::from_static("Content-Type, Range"));
    (StatusCode::NO_CONTENT, headers)
}

async fn handle_proxy(
    Query(params): Query<ProxyQuery>,
    headers: HeaderMap,
) -> Response {
    let range_header = headers.get("range")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    let proxy_result = match ProxyService::handle_request(params, range_header).await {
        Ok(r) => r,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let mut reply_headers = HeaderMap::new();
    reply_headers.insert("Access-Control-Allow-Origin", HeaderValue::from_static("*"));
    reply_headers.insert("Access-Control-Allow-Methods", HeaderValue::from_static("GET, OPTIONS"));
    reply_headers.insert("Access-Control-Allow-Headers", HeaderValue::from_static("Content-Type, Range"));
    reply_headers.insert("Access-Control-Expose-Headers", HeaderValue::from_static(
        "Content-Length, Content-Range, Accept-Ranges"
    ));

    for (k, v) in proxy_result.headers {
        if let Some(key) = k {
            reply_headers.insert(key, v);
        }
    }

    match proxy_result.body {
        ProxyBody::Text { content, .. } => (proxy_result.status, reply_headers, content).into_response(),
        ProxyBody::Stream { stream, .. } => {
            let body = Body::from_stream(stream);
            (proxy_result.status, reply_headers, body).into_response()
        }
    }
}