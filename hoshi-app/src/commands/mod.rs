pub mod auth;
pub mod users;
pub mod content;
pub mod extensions;
pub mod intergations;
pub mod schedule;
pub mod list;
pub mod proxy;
pub mod config;
pub mod progress;
pub mod backups;
#[cfg(feature = "watchparty")]
pub mod watchparty;

#[cfg(feature = "discord-rpc")]
pub mod discord;