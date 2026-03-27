use hoshi_core::proxy::{ProxyQuery, ProxyService};
use hoshi_core::error::CoreError;

#[tauri::command]
pub async fn proxy_fetch_text(
    url: String,
    referer: Option<String>,
    origin: Option<String>,
    user_agent: Option<String>,
    range: Option<String>,
) -> Result<String, CoreError> {
    let params = ProxyQuery { url, referer, origin, user_agent };

    let result = ProxyService::handle_request(params, range).await?;

    match result.body {
        hoshi_core::proxy::ProxyBody::Text { content, .. } => Ok(content),
        hoshi_core::proxy::ProxyBody::Stream { .. } => {
            Err(CoreError::BadRequest("error.proxy.binary_not_supported".into()))
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
) -> Result<Vec<u8>, CoreError> {
    use futures::TryStreamExt;

    let params = ProxyQuery { url, referer, origin, user_agent };
    let result = ProxyService::handle_request(params, range).await?;

    match result.body {
        hoshi_core::proxy::ProxyBody::Text { content, .. } => {
            Ok(content.into_bytes())
        }
        hoshi_core::proxy::ProxyBody::Stream { stream, .. } => {
            let bytes: Vec<u8> = stream
                .try_fold(Vec::new(), |mut acc, chunk| async move {
                    acc.extend_from_slice(&chunk);
                    Ok(acc)
                })
                .await
                .map_err(CoreError::Io)?;
            Ok(bytes)
        }
    }
}