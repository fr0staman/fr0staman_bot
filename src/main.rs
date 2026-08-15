mod config;
mod db;
mod dispatch;
mod enums;
mod handlers;
mod keyboards;
mod lang;
mod metrics;
mod services;
mod setup;
mod traits;
mod types;
mod utils;

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;

use teloxide::prelude::*;

use std::sync::Arc;

use crate::{
    config::{consts::BOT_PARSE_MODE, consts::GameState, env::BOT_CONFIG},
    dispatch::build_handler,
    utils::{helpers::get_chat_kind, mylog},
};

// [@fr0staman_bot Run!]
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    pretty_env_logger::init_timed();
    run().await
}

async fn run() {
    log::info!("Starting new version of @fr0staman_bot in Rust!");

    let bot = Bot::from_env().parse_mode(BOT_PARSE_MODE);

    setup::setup_me(&bot).await;
    setup::setup_lang();
    setup::setup_db().await;
    setup::setup_commands(&bot).await;

    let listener =
        setup::setup_listener(&bot).await.expect("Couldn't setup webhook!");

    let handler = build_handler(BOT_CONFIG.creator_id);

    let game_state = Arc::new(GameState::new());

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![game_state])
        .default_handler(default_log_handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch_with_listener(listener, mylog::MyErrorHandler::new())
        .await;
}

async fn default_log_handler(upd: std::sync::Arc<Update>) {
    metrics::UNHANDLED_COUNTER.inc();

    let update_id = upd.id.0;
    if let Some(user) = upd.from() {
        let user_id = user.id;
        if let Some(chat) = upd.chat() {
            let chat_id = chat.id;
            let chat_kind = get_chat_kind(&chat.kind);
            log::info!(
                "Unhandled update [{update_id}]: user: [{user_id}] chat: [{chat_kind}:{chat_id}]"
            );
        } else {
            log::info!("Unhandled update [{update_id}]: user: [{user_id}]");
        };
    } else if let Some(chat) = upd.chat() {
        let chat_id = chat.id;
        let chat_kind = get_chat_kind(&chat.kind);
        log::info!(
            "Unhandled update [{update_id}]: chat: [{chat_kind}:{chat_id}]"
        );
    } else {
        log::info!("Unhandled update [{update_id}]: kind: {:?}", upd.kind);
    }
}
