mod config;
mod db;
mod enums;
mod handlers;
mod keyboards;
mod lang;
mod metrics;
mod setup;
mod traits;
mod types;
mod utils;

pub use utils::mylog;

use teloxide::{dispatching::UpdateFilterExt, prelude::*, types::MessageKind};

use crate::{
    config::{consts::BOT_PARSE_MODE, env::BOT_CONFIG},
    enums::{AdminCommands, EpycCommands, MyCommands},
    handlers::{
        admin, callback, command, epyc, feedback, inline, message, system,
    },
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

    setup::setup_me(&bot).await;
    setup::setup_lang();
    setup::setup_db().await;
    setup::setup_commands(&bot).await;

    let listener =
        setup::setup_listener(&bot).await.expect("Couldn't setup webhook!");

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
                            && m.from.as_ref().is_some_and(|u| {
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
        .dispatch_with_listener(listener, mylog::MyErrorHandler::new())
        .await;
}

async fn default_log_handler(upd: std::sync::Arc<Update>) {
    crate::metrics::UNHANDLED_COUNTER.inc();

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
