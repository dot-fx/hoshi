use crate::error::AppResult;
use axum::{
    body::Body,
    extract::Query,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use hoshi_core::proxy::{ProxyBody, ProxyQuery, ProxyService};
use std::sync::Arc;

pub fn proxy_routes() -> Router<Arc<hoshi_core::state::AppState>> {
    Router::new()
        .route("/proxy", get(handle_proxy).options(handle_proxy_options))
}

async fn handle_proxy_options() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert("Access-Control-Allow-Origin", HeaderValue::from_static("*"));
    headers.insert("Access-Control-Allow-Methods", HeaderValue::from_static("GET, OPTIONS"));
    headers.insert("Access-Control-Allow-Headers", HeaderValue::from_static("Content-Type, Range"));
    (StatusCode::NO_CONTENT, headers)
}

pub async fn handle_proxy(
    Query(params): Query<ProxyQuery>,
    headers: HeaderMap,
) -> AppResult<Response> {
    let range_header = headers.get("range")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    let proxy_result = ProxyService::handle_request(params, range_header).await?;

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
        ProxyBody::Text { content, .. } => Ok((proxy_result.status, reply_headers, content).into_response()),
        ProxyBody::Stream { stream, .. } => {
            let body = Body::from_stream(stream);
            Ok((proxy_result.status, reply_headers, body).into_response())
        }
    }
}