mod config;
mod consts;
mod db;
mod db_api;
mod enums;
mod handlers;
mod keyboards;
mod lang;
mod metrics;
mod models;
mod schema;
mod traits;
mod types;
mod utils;
pub use types::{MyBot, MyError, MyResult};

use axum::Router;
use std::sync::Arc;

use teloxide::{
    dispatching::UpdateFilterExt,
    prelude::*,
    types::MessageKind,
    update_listeners::{webhooks, UpdateListener},
    utils::command::BotCommands,
};

use crate::{
    config::{BOT_CONFIG, BOT_ME},
    consts::{BOT_PARSE_MODE, DEFAULT_LANG_TAG, IGNORED_COMMANDS},
    db::Database,
    enums::{AdminCommands, EpycCommands, MyCommands},
    handlers::{
        admin, callback, command, epyc, feedback, inline, message, system,
    },
    lang::{get_langs, lng},
    utils::helpers::get_chat_kind,
};

// ============================================================================
// [@fr0staman_bot Run!]
// ============================================================================
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    pretty_env_logger::init_timed();
    run().await
}

async fn run() {
    log::info!("Starting new version of @fr0staman_bot in Rust!");

    let bot = Bot::from_env().parse_mode(BOT_PARSE_MODE);
    setup_me(&bot).await;
    setup_lang();
    setup_db().await;
    setup_commands(&bot).await;

    let listener = setup_listener(&bot).await.expect("Couldn't setup webhook!");

    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .branch(
                    dptree::entry()
                        .filter_command::<MyCommands>()
                        .endpoint(command::filter_commands),
                )
                .branch(
                    dptree::entry()
                        .filter_command::<EpycCommands>()
                        .endpoint(epyc::filter_commands),
                )
                .branch(
                    dptree::filter(|m: Message| {
                        m.chat.is_private()
                            && m.from().is_some_and(|u| {
                                u.id.0 == BOT_CONFIG.creator_id
                            })
                    })
                    .filter_command::<AdminCommands>()
                    .endpoint(admin::filter_admin_commands),
                )
                .branch(
                    Message::filter_new_chat_members()
                        .endpoint(system::handle_new_member),
                )
                .branch(
                    Message::filter_left_chat_member()
                        .endpoint(system::handle_left_member),
                )
                .branch(
                    dptree::filter(|m: Message| {
                        m.migrate_to_chat_id().is_some()
                    })
                    .endpoint(system::handle_chat_migration),
                )
                .branch(
                    dptree::filter(|m: Message| {
                        matches!(
                            m.kind,
                            MessageKind::VideoChatStarted(_)
                                | MessageKind::VideoChatEnded(_)
                        )
                    })
                    .endpoint(system::handle_video_chat),
                )
                .branch(
                    dptree::filter(|m: Message| m.text().is_some())
                        .endpoint(message::handle_message),
                )
                .branch(
                    dptree::filter(|m: Message| {
                        m.voice().is_some() && m.chat.is_private()
                    })
                    .endpoint(system::handle_voice_private),
                )
                .branch(
                    dptree::filter(|m: Message| {
                        m.animation().is_some() && m.chat.is_private()
                    })
                    .endpoint(system::handle_animation_private),
                ),
        )
        .branch(
            Update::filter_inline_query()
                .endpoint(inline::filter_inline_commands),
        )
        .branch(
            Update::filter_callback_query()
                .endpoint(callback::filter_callback_commands),
        )
        .branch(
            Update::filter_my_chat_member()
                .filter(|u: Update| u.chat().is_some_and(|c| c.is_private()))
                .endpoint(system::handle_ban_or_unban_in_private),
        )
        .branch(
            Update::filter_chosen_inline_result()
                .endpoint(feedback::filter_inline_feedback_commands),
        );

    Dispatcher::builder(bot, handler)
        .default_handler(default_log_handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch_with_listener(listener, LoggingErrorHandler::new())
        .await;
}

async fn setup_listener(
    bot: &MyBot,
) -> MyResult<impl UpdateListener<Err = std::convert::Infallible>> {
    let port = BOT_CONFIG.webhook_port;
    let host = &BOT_CONFIG.webhook_url;

    let addr = ([0, 0, 0, 0], port).into();
    let url = host.join("/webhookBot").expect("Invalid WEBHOOK_URL");

    let options = webhooks::Options::new(addr, url);

    let webhooks::Options { address, .. } = options;

    let (mut update_listener, stop_flag, bot_router) =
        webhooks::axum_to_router(bot.clone(), options).await?;
    let stop_token = update_listener.stop_token();

    tokio::spawn(async move {
        axum::Server::bind(&address)
            .serve(
                Router::new()
                    .merge(bot_router)
                    .merge(metrics::init())
                    .fallback(fallback_404)
                    .into_make_service(),
            )
            .with_graceful_shutdown(stop_flag)
            .await
            .map_err(|err| {
                stop_token.stop();
                err
            })
            .expect("Axum server error");
    });

    Ok(update_listener)
}

fn setup_lang() {
    let loc = lang::Locale::new(DEFAULT_LANG_TAG);
    lang::LANG.set(loc).expect("Locale set error!");
}

async fn setup_me(bot: &MyBot) {
    let me = bot.get_me().await.unwrap();
    BOT_ME.set(me).unwrap();
}

async fn setup_db() {
    // I just try check database, thats not bad
    let _ = Database::new();
}

async fn setup_commands(bot: &MyBot) {
    let langs = get_langs();
    for (ltag, lang) in langs.iter().enumerate() {
        let mut game_commands = MyCommands::bot_commands();
        game_commands
            .retain(|i| !IGNORED_COMMANDS.contains(&i.command.as_str()));

        for command in game_commands.iter_mut() {
            command.description =
                lng(&format!("{}_desc", &command.command), ltag);
        }
        let res = if *lang == DEFAULT_LANG_TAG {
            bot.set_my_commands(game_commands).await
        } else {
            bot.set_my_commands(game_commands).language_code(*lang).await
        };

        res.expect("Error with command set: ");
    }
    log::info!("Commands installed successfully!");
}

async fn default_log_handler(upd: Arc<Update>) {
    crate::metrics::UNHANDLED_COUNTER.inc();

    let update_id = upd.id;
    if let Some(user) = upd.user() {
        let user_id = user.id;
        if let Some(chat) = upd.chat() {
            let chat_id = chat.id;
            let chat_kind = get_chat_kind(&chat.kind);
            log::info!("Unhandled update [{update_id}]: user: [{user_id}] chat: [{chat_kind}:{chat_id}]");
        } else {
            log::info!("Unhandled update [{update_id}]: user: [{user_id}] ");
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

async fn fallback_404() {
    log::warn!("Axum: 404");
}
