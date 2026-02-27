use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use serde::Deserialize;
use crate::error::{CoreError, CoreResult};

#[derive(Deserialize, Debug, Clone)]
pub struct RpcData {
    pub details: Option<String>,
    pub state: Option<String>,
    pub mode: Option<String>, // "watching" | "reading"
    pub start_timestamp: Option<i64>,
    pub end_timestamp: Option<i64>,
    pub paused: Option<bool>,
    pub version: Option<String>,
}

pub struct RpcManager {
    client: Option<DiscordIpcClient>,
    client_id: String,
}

impl RpcManager {
    pub fn new(client_id: String) -> Self {
        Self {
            client: None,
            client_id,
        }
    }

    fn connect(&mut self) -> CoreResult<()> {
        let mut client = DiscordIpcClient::new(&self.client_id)
            .map_err(|e| CoreError::Internal(format!("Error creando cliente Discord: {}", e)))?;

        client.connect()
            .map_err(|e| CoreError::Internal(format!("Error conectando a Discord: {}", e)))?;

        self.client = Some(client);
        tracing::info!("Discord RPC connected");
        Ok(())
    }

    pub fn set_activity(&mut self, data: RpcData) -> CoreResult<()> {
        if self.client.is_none() {
            if let Err(e) = self.connect() {
                tracing::warn!("Couldn't connect to discord: {}", e);
                return Ok(());
            }
        }

        let client = self.client.as_mut().unwrap();
        let mut activity = activity::Activity::new();

        if let Some(details) = &data.details {
            activity = activity.details(details);
        }
        if let Some(state) = &data.state {
            activity = activity.state(state);
        }

        let mut assets = activity::Assets::new();

        if data.paused == Some(true) {
            assets = assets.large_image("bigpicture");
            assets = assets.large_text("Paused");
        } else {
            assets = assets.large_image("bigpicture");
            assets = assets.large_text(data.version.as_deref().unwrap_or("v2.0.0"));

            if let (Some(start), Some(end)) = (data.start_timestamp, data.end_timestamp) {
                let mut timestamps = activity::Timestamps::new();
                timestamps = timestamps.start(start);
                timestamps = timestamps.end(end);
                activity = activity.timestamps(timestamps);
            }
        }
        activity = activity.assets(assets);

        if let Err(e) = client.set_activity(activity) {
            tracing::error!("Error sending activity to discord, disconnecting: {}", e);
            self.client = None;
            return Err(CoreError::Internal(format!("Discord IPC error: {}", e)));
        }

        Ok(())
    }
}