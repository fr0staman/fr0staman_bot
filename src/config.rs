use std::{env, str::FromStr, sync::OnceLock};

use once_cell::sync::Lazy;
use teloxide::types::Me;
use url::Url;

pub struct Config {
    // For some situations with props with "me"
    pub me: &'static Me,
    pub teloxide_token: String,
    pub webhook_url: Url,
    pub webhook_port: u16,
    pub database_url: Url,
    // For inline query pictures
    pub photostock_url: Url,
    // For checking subscription on channel
    pub channel_id: i64,
    // For "invite" link to channel
    pub channel_name: String,
    // Channel with contents check, especially grunts
    pub content_check_channel_id: i64,
    pub content_check_channel_name: String,
    // Bot creator for manage
    pub creator_id: u64,
    pub prometheus_token: String,
    pub gif_content_channel_id: i64,
    pub chat_link: String,
}

fn _from_env<T: FromStr>(name: &str) -> T
where
    <T as FromStr>::Err: std::fmt::Debug,
{
    env::var(name)
        .unwrap_or_else(|_| panic!("{} is not defined!", name))
        .parse::<T>()
        .unwrap_or_else(|_| panic!("{} is not valid!", name))
}

pub static BOT_CONFIG: Lazy<Config> = Lazy::new(|| Config {
    me: BOT_ME.get().expect("BOT_ME is not set!"),
    teloxide_token: _from_env("TELOXIDE_TOKEN"),
    webhook_url: _from_env("WEBHOOK_URL"),
    webhook_port: _from_env("WEBHOOK_PORT"),
    database_url: _from_env("DATABASE_URL"),
    photostock_url: _from_env("PHOTOSTOCK_URL"),
    channel_id: _from_env("CHANNEL_ID"),
    channel_name: _from_env("CHANNEL_NAME"),
    content_check_channel_id: _from_env("CONTENT_CHECK_CHANNEL_ID"),
    content_check_channel_name: _from_env("CONTENT_CHECK_CHANNEL_NAME"),
    creator_id: _from_env("CREATOR_ID"),
    prometheus_token: _from_env("PROMETHEUS_TOKEN"),
    gif_content_channel_id: _from_env("GIF_CONTENT_CHANNEL_ID"),
    chat_link: _from_env("CHAT_LINK"),
});

pub static BOT_ME: OnceLock<Me> = OnceLock::new();
