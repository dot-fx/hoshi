#[cfg(feature = "discord-rpc")]
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
#[cfg(feature = "discord-rpc")]
use std::sync::Mutex;
#[cfg(feature = "discord-rpc")]
use tracing::{info, warn, error, debug, instrument};

#[cfg(feature = "discord-rpc")]
use crate::config::repository::ConfigRepo;
#[cfg(feature = "discord-rpc")]
use crate::state::AppState;

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
        url.to_string()
    }

    #[instrument(skip(self, state, details, image_url), fields(title = %title, is_video = %is_video))]
    pub fn set_activity(
        &self,
        state: &AppState,
        user_id: i32,
        title: &str,
        details: &str,
        image_url: Option<&str>,
        start_time: Option<i64>,
        end_time: Option<i64>,
        is_video: bool,
        is_nsfw: bool,
    ) {
        let user_config = {
            let conn = state.db.connection();
            let conn_lock = match conn.lock() {
                Ok(lock) => lock,
                Err(e) => {
                    error!(error = ?e, "Failed to lock DB connection for Discord RPC config");
                    return;
                }
            };
            ConfigRepo::get_config(&conn_lock, user_id).unwrap_or_default()
        };

        let config = &user_config.discord;

        if !config.enabled {
            debug!("Discord RPC is disabled in user config, clearing activity");
            self.clear_activity();
            return;
        }

        let mut lock = self.client.lock().unwrap();

        if lock.is_none() {
            debug!("Initializing new Discord IPC client");
            let mut client = DiscordIpcClient::new(&self.client_id);
            if let Err(e) = client.connect() {
                warn!(error = ?e, "Failed to connect to Discord client");
            } else {
                *lock = Some(client);
            }
        }

        if let Some(client) = lock.as_mut() {
            let hide_content = !config.show_title || (config.hide_nsfw && is_nsfw);

            let (final_details, final_state, final_image, final_start, final_end) = if hide_content {
                debug!("Hiding content details due to user preferences or NSFW flag");
                ("Hoshi", "In App", None, None, None)
            } else {
                (title, details, image_url, start_time, end_time)
            };

            let img_string = final_image.map(Self::format_image);
            let mut assets = activity::Assets::new();
            if let Some(ref url) = img_string {
                assets = assets.large_image(url);
            }

            let activity_type = if is_video && !hide_content {
                activity::ActivityType::Watching
            } else {
                activity::ActivityType::Playing
            };

            let mut payload = activity::Activity::new()
                .activity_type(activity_type)
                .details(final_details)
                .state(final_state)
                .assets(assets);

            if final_start.is_some() || final_end.is_some() {
                let mut timestamps = activity::Timestamps::new();
                if let Some(s) = final_start { timestamps = timestamps.start(s); }
                if let Some(e) = final_end { timestamps = timestamps.end(e); }
                payload = payload.timestamps(timestamps);
            }

            if let Err(e) = client.set_activity(payload) {
                error!(error = ?e, "Failed to send activity payload to Discord RPC");

                *lock = None;
            } else {
                debug!("Discord RPC activity updated successfully");
            }
        }
    }

    #[instrument(skip(self))]
    pub fn clear_activity(&self) {
        let mut lock = self.client.lock().unwrap();
        if let Some(client) = lock.as_mut() {
            if let Err(e) = client.clear_activity() {
                warn!(error = ?e, "Failed to clear Discord activity");
            } else {
                debug!("Discord activity cleared");
            }
        }
    }
}