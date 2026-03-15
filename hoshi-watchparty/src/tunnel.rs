use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

#[derive(Debug, thiserror::Error)]
pub enum TunnelError {
    #[error("cloudflared is not installed or not found in PATH")]
    NotInstalled,
    #[error("Timeout waiting for tunnel URL (is cloudflared working?)")]
    Timeout,
    #[error("Failed to obtain tunnel URL from cloudflared output")]
    NoUrlFound,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Internal error: {0}")]
    Internal(String),
}

pub type TunnelResult<T> = Result<T, TunnelError>;

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

    pub async fn open_tunnel(&self, local_port: u16) -> TunnelResult<String> {
        let mut process_guard = self.process.lock().await;
        let mut url_guard = self.public_url.lock().await;
        let mut rooms_guard = self.exposed_rooms.lock().await;

        *rooms_guard += 1;

        if process_guard.is_some() {
            if let Some(url) = url_guard.clone() {
                return Ok(url);
            }
        }

        println!("[Tunnel] Waiting for local server on port {local_port}...");

        for _ in 0..20 {
            if tokio::net::TcpStream::connect(format!("127.0.0.1:{local_port}")).await.is_ok() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }

        println!("[Tunnel] Starting cloudflared on port {local_port}...");

        let mut child = Command::new("cloudflared")
            .arg("tunnel")
            .arg("--url")
            .arg(format!("http://127.0.0.1:{local_port}"))
            .arg("--no-autoupdate")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    TunnelError::NotInstalled
                } else {
                    TunnelError::Io(e)
                }
            })?;

        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| TunnelError::Internal("Failed to capture stderr".into()))?;

        let mut reader = BufReader::new(stderr).lines();

        let found_url = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            async {
                let re = regex::Regex::new(
                    r"https://[a-z0-9-]+\.trycloudflare\.com"
                ).unwrap();

                while let Ok(Some(line)) = reader.next_line().await {
                    println!("[Tunnel] cloudflared: {line}");
                    if let Some(mat) = re.find(&line) {
                        return Some(mat.as_str().to_string());
                    }
                }
                None
            },
        )
            .await
            .map_err(|_| TunnelError::Timeout)?;

        match found_url {
            Some(url) => {
                println!("[Tunnel] Tunnel opened at: {url}, waiting for propagation...");
                tokio::time::sleep(std::time::Duration::from_secs(5)).await; // child is loose here
                *process_guard = Some(child); // only stored after sleep
                *url_guard = Some(url.clone());
                Ok(url)
            }
            None => {
                let _ = child.kill().await;
                Err(TunnelError::NoUrlFound)
            }
        }
    }

    pub async fn close_tunnel_if_unused(&self) {
        let mut rooms_guard = self.exposed_rooms.lock().await;
        if *rooms_guard > 0 {
            *rooms_guard -= 1;
        }

        println!("[Tunnel] Exposed rooms count: {}", *rooms_guard);

        if *rooms_guard == 0 {
            let mut process_guard = self.process.lock().await;
            if let Some(mut child) = process_guard.take() {
                println!("[Tunnel] Closing tunnel (no rooms exposed)...");
                let _ = child.kill().await;
                *self.public_url.lock().await = None;
            }
        }
    }

    pub async fn force_close(&self) {
        let mut process_guard = self.process.lock().await;
        if let Some(mut child) = process_guard.take() {
            println!("[Tunnel] Forcing tunnel close...");
            let _ = child.kill().await;
            *self.public_url.lock().await = None;
            *self.exposed_rooms.lock().await = 0;
        }
    }
}