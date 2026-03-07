use hoshi_core::proxy::service::{ProxyQuery, ProxyService};
use tauri::State;

#[tauri::command]
pub async fn proxy_fetch_text(
    url: String,
    referer: Option<String>,
    origin: Option<String>,
    user_agent: Option<String>,
    range: Option<String>,
) -> Result<String, String> {
    let params = ProxyQuery { url, referer, origin, user_agent };
    let result = ProxyService::handle_request(params, range)
        .await
        .map_err(|e| e.to_string())?;

    match result.body {
        hoshi_core::proxy::service::ProxyBody::Text { content, .. } => Ok(content),
        hoshi_core::proxy::service::ProxyBody::Stream { .. } => {
            Err("Binary streams not supported via text proxy — use proxy_fetch_bytes".into())
        }
    }
}

#[tauri::command]
pub async fn proxy_fetch_bytes(
    url: String,
    referer: Option<String>,
    origin: Option<String>,
    user_agent: Option<String>,
    range: Option<String>,
) -> Result<Vec<u8>, String> {
    use futures::TryStreamExt;

    let params = ProxyQuery { url, referer, origin, user_agent };
    let result = ProxyService::handle_request(params, range)
        .await
        .map_err(|e| e.to_string())?;

    match result.body {
        hoshi_core::proxy::service::ProxyBody::Text { content, .. } => {
            Ok(content.into_bytes())
        }
        hoshi_core::proxy::service::ProxyBody::Stream { stream, .. } => {
            let bytes: Vec<u8> = stream
                .try_fold(Vec::new(), |mut acc, chunk| async move {
                    acc.extend_from_slice(&chunk);
                    Ok(acc)
                })
                .await
                .map_err(|e| e.to_string())?;
            Ok(bytes)
        }
    }
}