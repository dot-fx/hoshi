use std::sync::Arc;
use axum::{Router, middleware as axum_middleware};
use axum::http::{header, StatusCode, Uri};
use axum::response::{Html, IntoResponse, Response};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

use hoshi_core::state::AppState;
use hoshi_core::auth::service::AuthService;
use hoshi_core::users::service::UserService;
use hoshi_watchparty::{watchparty_routes, WatchPartyManager};
use hoshi_watchparty::routes::SessionResolver;

use crate::api::{auth, users, proxy, extensions, booru, list, content, collections, integrations, schedule, config, progress, backups};
use crate::middleware::{session_auth_middleware, tunnel_security_middleware, extract_session_cookie};
use crate::assets::Assets;

pub fn build_router(state: Arc<AppState>) -> Router {
    let wp_manager = Arc::new(WatchPartyManager::new());

    let resolver_state = state.clone();
    let session_resolver: SessionResolver = Arc::new(move |cookies: &str| {
        let session_id = extract_session_cookie(cookies)?;
        let session = AuthService::get_session(&resolver_state, &session_id).ok()??;
        let user = UserService::get_user_public(&resolver_state, session.user_id).ok()?;
        Some((user.id.to_string(), user.username, user.avatar))
    });

    Router::new()
        .nest("/api", auth::auth_routes())
        .nest("/api", users::user_routes())
        .nest("/api", proxy::proxy_routes())
        .nest("/api", extensions::extensions_routes())
        .nest("/api", booru::booru_routes())
        .nest("/api", list::list_routes())
        .nest("/api", content::content_routes())
        .nest("/api", collections::collection_routes())
        .nest("/api", integrations::integration_routes())
        .nest("/api", schedule::schedule_routes())
        .nest("/api", config::config_routes())
        .nest("/api", progress::progress_routes())
        .nest("/api", backups::backup_routes())
        .merge(watchparty_routes(wp_manager, session_resolver))
        .route("/_app/*file", axum::routing::get(static_handler))
        .route("/robots.txt", axum::routing::get(static_handler))
        .fallback(spa_fallback)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            session_auth_middleware,
        ))
        .layer(axum_middleware::from_fn(tunnel_security_middleware))
        .with_state(state)
}

pub async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => (StatusCode::NOT_FOUND, "404 Not Found").into_response(),
    }
}

pub async fn spa_fallback() -> Response {
    match Assets::get("index.html") {
        Some(content) => Html(content.data).into_response(),
        None => {
            tracing::error!("index.html not found in embedded assets");
            (StatusCode::INTERNAL_SERVER_ERROR, "Application bundle not found").into_response()
        }
    }
}