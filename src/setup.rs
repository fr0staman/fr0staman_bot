use axum::Router;

use teloxide::{
    prelude::*,
    update_listeners::{UpdateListener, webhooks},
    utils::command::BotCommands,
};
use tokio::net::TcpListener;

use crate::{
    config::{
        consts::{DEFAULT_LANG_TAG, IGNORED_COMMANDS},
        env::{BOT_CONFIG, BOT_ME, BOT_STATIC},
    },
    db::init::Database,
    enums::MyCommands,
    lang::{self, get_langs, lng},
    metrics,
    types::{MyBot, MyResult},
};

pub async fn setup_listener(
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

    let listener = TcpListener::bind(&address).await.expect("Listener error");
    tokio::spawn(async move {
        axum::serve(
            listener,
            Router::new()
                .merge(bot_router)
                .merge(metrics::init())
                .fallback(fallback_404)
                .into_make_service(),
        )
        .with_graceful_shutdown(stop_flag)
        .await
        .inspect_err(|_| stop_token.stop())
        .expect("Axum server error");
    });

    Ok(update_listener)
}

pub fn setup_lang() {
    let loc = lang::Locale::new(DEFAULT_LANG_TAG);
    lang::LANG.set(loc).expect("Locale set error!");
}

pub async fn setup_me(bot: &MyBot) {
    let me = bot.get_me().await.unwrap();
    BOT_ME.set(me).unwrap();
    BOT_STATIC.set(bot.clone()).unwrap()
}

pub async fn setup_db() {
    // I just try check database, thats not bad
    let _ = Database::new();
}

pub async fn setup_commands(bot: &MyBot) {
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
            bot.set_my_commands(game_commands).language_code(lang).await
        };

        res.expect("Error with command set: ");
    }
    log::info!("Commands installed successfully!");
}

async fn fallback_404() {
    log::warn!("Axum: 404");
}
