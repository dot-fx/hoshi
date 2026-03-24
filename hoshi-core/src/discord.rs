#[cfg(feature = "discord-rpc")]
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
#[cfg(feature = "discord-rpc")]
use std::sync::Mutex;

#[cfg(feature = "discord-rpc")]
pub struct DiscordRpcService {
    client: Mutex<Option<DiscordIpcClient>>,
    client_id: String,
}

#[cfg(feature = "discord-rpc")]
impl DiscordRpcService {
    pub fn new(client_id: &str) -> Self {
        Self {
            client: Mutex::new(None),
            client_id: client_id.to_string(),
        }
    }

    fn format_image(url: &str) -> String {
        // Probamos enviando la URL directa, que es soportado en versiones modernas de Discord
        if url.starts_with("http") {
            url.to_string()
        } else {
            url.to_string()
        }
    }

    // hoshi-core/src/discord.rs

    pub fn set_activity(
        &self,
        title: &str,
        details: &str,
        image_url: Option<&str>,
        start_time: Option<i64>,
        end_time: Option<i64>,
        is_video: bool, // <--- Asegúrate de que este parámetro esté aquí
    ) {
        let mut lock = self.client.lock().unwrap();

        if lock.is_none() {
            let mut client = DiscordIpcClient::new(&self.client_id);
            if client.connect().is_ok() {
                *lock = Some(client);
            }
        }

        if let Some(client) = lock.as_mut() {
            let img_string = image_url.map(Self::format_image);
            let mut assets = activity::Assets::new();
            if let Some(ref url) = img_string {
                assets = assets.large_image(url);
            }

            // Logs de Debug para la terminal
            println!("RPC Sync -> Start: {:?}, End: {:?}, Video: {}", start_time, end_time, is_video);

            let activity_type = if is_video {
                activity::ActivityType::Watching
            } else {
                activity::ActivityType::Playing
            };

            let mut payload = activity::Activity::new()
                .activity_type(activity_type)
                .details(title)
                .state(details)
                .assets(assets);

            // IMPORTANTE: Re-asignar el payload con los timestamps
            if start_time.is_some() || end_time.is_some() {
                let mut timestamps = activity::Timestamps::new();
                if let Some(s) = start_time { timestamps = timestamps.start(s); }
                if let Some(e) = end_time { timestamps = timestamps.end(e); }
                payload = payload.timestamps(timestamps);
            }

            if let Err(e) = client.set_activity(payload) {
                println!("Error al enviar RPC: {:?}", e);
            }
        }
    }

    pub fn clear_activity(&self) {
        let mut lock = self.client.lock().unwrap();
        if let Some(client) = lock.as_mut() {
            let _ = client.clear_activity();
        }
    }
}