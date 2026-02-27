use crate::error::{AppError, AppResult};
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use hoshi_core::error::CoreError;

pub struct TunnelManager {
    process: Arc<Mutex<Option<Child>>>,
    public_url: Arc<Mutex<Option<String>>>,
    exposed_rooms: Arc<Mutex<usize>>,
}

impl TunnelManager {
    pub fn new() -> Self {
        Self {
            process: Arc::new(Mutex::new(None)),
            public_url: Arc::new(Mutex::new(None)),
            exposed_rooms: Arc::new(Mutex::new(0)),
        }
    }

    pub async fn open_tunnel(&self) -> AppResult<String> {
        let mut process_guard = self.process.lock().await;
        let mut url_guard = self.public_url.lock().await;
        let mut rooms_guard = self.exposed_rooms.lock().await;

        *rooms_guard += 1;

        if process_guard.is_some() {
            if let Some(url) = url_guard.clone() {
                return Ok(url);
            }
        }

        tracing::info!("[Tunnel] Starting cloudflared...");

        let mut child = Command::new("cloudflared")
            .arg("tunnel")
            .arg("--url")
            .arg("http://localhost:PORT")
            .arg("--no-autoupdate")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(CoreError::from)?;

        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| CoreError::Internal("Failed to capture stderr".into()))?;

        let mut reader = BufReader::new(stderr).lines();

        let found_url = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            async {
                let re = regex::Regex::new(
                    r"https://[a-z0-9-]+\.trycloudflare\.com"
                ).unwrap();

                while let Ok(Some(line)) = reader.next_line().await {
                    if let Some(mat) = re.find(&line) {
                        return Some(mat.as_str().to_string());
                    }
                }
                None
            },
        )
            .await
            .map_err(|_| CoreError::Internal("Timeout waiting for Tunnel URL".into()))?;

        if let Some(url) = found_url {
            tracing::info!("[Tunnel] Tunnel opened at: {}", url);
            *process_guard = Some(child);
            *url_guard = Some(url.clone());
            Ok(url)
        } else {
            let _ = child.kill().await;
            Err(CoreError::Internal("Failed to obtain Tunnel URL".into()).into())
        }
    }

    pub async fn close_tunnel_if_unused(&self) {
        let mut rooms_guard = self.exposed_rooms.lock().await;
        if *rooms_guard > 0 {
            *rooms_guard -= 1;
        }

        tracing::info!("[Tunnel] Exposed rooms count: {}", *rooms_guard);

        if *rooms_guard == 0 {
            let mut process_guard = self.process.lock().await;
            if let Some(mut child) = process_guard.take() {
                tracing::info!("[Tunnel] Closing tunnel (no rooms exposed)...");
                let _ = child.kill().await;
                *self.public_url.lock().await = None;
            }
        }
    }

    pub async fn force_close(&self) {
        let mut process_guard = self.process.lock().await;
        if let Some(mut child) = process_guard.take() {
            tracing::warn!("[Tunnel] Forcing tunnel close...");
            let _ = child.kill().await;
            *self.public_url.lock().await = None;
            *self.exposed_rooms.lock().await = 0;
        }
    }
}