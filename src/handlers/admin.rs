use std::time::Duration;

use ahash::AHashSet;
use futures::FutureExt;
use teloxide::{RequestError, prelude::*};

use crate::{
    config::{consts::HAND_PIG_ADDITION_ON_SUPPORTED, env::BOT_CONFIG},
    db::{DB, models::UserStatus},
    enums::AdminCommands,
    lang::{InnerLang, LocaleTag, get_tag, lng, tag_one_or},
    traits::MaybeMessageSetter,
    types::{MyBot, MyResult},
    utils::{date::get_date, formulas::calculate_hryak_size, helpers},
};

#[derive(Eq, Hash, PartialEq)]
enum RepostTarget {
    DM,
    Chats,
}

pub async fn filter_admin_commands(
    bot: MyBot,
    m: Message,
    cmd: AdminCommands,
) -> MyResult<()> {
    let Some(from) = &m.from else { return Ok(()) };
    let Some(user) = DB.other.get_user(from.id.0).await? else { return Ok(()) };

    let ltag = tag_one_or(user.lang.as_deref(), get_tag(from));

    let function = match &cmd {
        AdminCommands::Promote(arg) => {
            admin_command_promote(bot, &m, ltag, arg).boxed()
        },
        AdminCommands::Repost(arg) => {
            admin_command_repost(bot, &m, ltag, arg).boxed()
        },
    };

    let response = function.await;

    let user_id = m.from.map_or(0, |u| u.id.0);

    if let Err(err) = response {
        crate::myerr!(
            "Error {:?}: admin command /{:?}: user [{}] in chat [{}]",
            err,
            cmd,
            user_id,
            m.chat.id,
        );
    } else {
        log::info!(
            "Handled admin command /{:?}: user [{}] in chat [{}]",
            cmd,
            user_id,
            m.chat.id,
        );
    }

    Ok(())
}

async fn admin_command_promote(
    bot: MyBot,
    m: &Message,
    ltag: LocaleTag,
    arg: &str,
) -> MyResult<()> {
    if arg.is_empty() {
        let text = lng("AdminCommandPromoteEmpty", ltag);
        bot.send_message(m.chat.id, text).maybe_thread_id(m).await?;
        return Ok(());
    }

    let Ok(parsed_id) = arg.parse::<u32>() else {
        let text = lng("AdminCommandPromoteInvalid", ltag);
        bot.send_message(m.chat.id, text).maybe_thread_id(m).await?;
        return Ok(());
    };

    let Some(user) = DB.other.get_user_by_id(parsed_id).await? else {
        let text = lng("AdminCommandPromoteNotFound", ltag);
        bot.send_message(m.chat.id, text).maybe_thread_id(m).await?;
        return Ok(());
    };

    if user.supported {
        let text = lng("AdminCommandPromoteAlready", ltag);
        bot.send_message(m.chat.id, text).await?;
        return Ok(());
    }

    let user_status = UserStatus {
        banned: user.banned,
        started: user.started,
        supported: true,
        subscribed: user.subscribed,
    };
    DB.other.change_user_status(user.user_id, user_status).await?;
    let Some(user) = DB.other.get_user_by_id(parsed_id).await? else {
        let text = lng("AdminCommandPromoteNotFound", ltag);
        bot.send_message(m.chat.id, text).maybe_thread_id(m).await?;
        return Ok(());
    };

    let cur_date = get_date();
    let weight = calculate_hryak_size(user.user_id);
    let mass = weight + helpers::mass_addition_on_status(&user);
    DB.hand_pig
        .update_hrundel_date_and_size(user.user_id, mass, cur_date)
        .await?;

    // TODO: get lang from user
    let text = lng("AdminCommandPromoteUserMessage", ltag)
        .args(&[("amount", HAND_PIG_ADDITION_ON_SUPPORTED)]);
    let res =
        bot.send_message(UserId(user.user_id), text).maybe_thread_id(m).await;

    // In 99.99% situations, user just banned me in private, but I didnt received it in the past.
    if res.is_err() {
        let user_status = UserStatus {
            banned: true,
            started: user.started,
            supported: true,
            subscribed: user.subscribed,
        };
        DB.other.change_user_status(user.user_id, user_status).await?;
    };

    let text = if res.is_err() {
        lng("AdminCommandPromoteError", ltag)
    } else {
        lng("AdminCommandPromoteSuccess", ltag)
    };

    bot.send_message(m.chat.id, text).maybe_thread_id(m).await?;

    Ok(())
}

async fn admin_command_repost(
    bot: MyBot,
    m: &Message,
    ltag: LocaleTag,
    arg: &str,
) -> MyResult<()> {
    let m = m.clone();
    let arg = arg.to_string();
    tokio::spawn(async move {
        _inner_admin_command_repost(bot, &m, ltag, &arg).await
    });

    Ok(())
}

async fn _inner_admin_command_repost(
    bot: MyBot,
    m: &Message,
    _ltag: LocaleTag,
    arg: &str,
) -> MyResult<()> {
    const USER_SENDING_THROTTLE_TIME_MS: u64 = 333;
    const CHAT_SENDING_THROTTLE_TIME_MS: u64 = 333;

    let Some(reply) = m.reply_to_message() else {
        bot.send_message(m.chat.id, "Where is no reply.")
            .maybe_thread_id(m)
            .await?;
        return Ok(());
    };

    let mut flags = AHashSet::new();
    for word in arg.split_whitespace() {
        match word {
            "+dm" => flags.insert(RepostTarget::DM),
            "+chats" => flags.insert(RepostTarget::Chats),
            _ => {
                let error_text = format!("Unknown flag: {}", word);
                log::error!("{}", error_text);
                bot.send_message(m.chat.id, error_text)
                    .maybe_thread_id(m)
                    .await?;
                return Ok(());
            },
        };
    }

    let mut chat_sended_count = 0;
    let mut chat_sended_error_count = 0;
    let mut user_sended_count = 0;
    let mut user_sended_error_count = 0;

    if flags.contains(&RepostTarget::Chats) {
        let chats = DB.other.get_chats().await?;

        for chat in chats {
            if !chat.active {
                continue;
            }
            let res = bot
                .forward_message(ChatId(chat.chat_id), reply.chat.id, reply.id)
                .await;

            if let Err(err) = res {
                log::error!(
                    "Failed to repost to chat [{}] {}",
                    chat.chat_id,
                    err
                );
                if let RequestError::RetryAfter(sec) = err {
                    log::info!(
                        "Retrying to repost to chat [{}] in {} seconds...",
                        chat.id,
                        sec.seconds()
                    );
                    tokio::time::sleep(sec.duration()).await;
                    let res_again = bot
                        .forward_message(
                            ChatId(chat.chat_id),
                            reply.chat.id,
                            reply.id,
                        )
                        .await;
                    if res_again.is_err() {
                        chat_sended_error_count += 1;
                    }
                }
            } else {
                chat_sended_count += 1;
                log::info!("Reposted to chat [{}]", chat.chat_id);
            }
            tokio::time::sleep(Duration::from_millis(
                CHAT_SENDING_THROTTLE_TIME_MS,
            ))
            .await;
        }
    }

    let text = format!(
        "Sending chats: {}\nErrors: {}",
        chat_sended_count, chat_sended_error_count
    );
    let _ = bot.send_message(UserId(BOT_CONFIG.creator_id), text).await;

    if flags.contains(&RepostTarget::DM) {
        let users = DB.other.get_users().await?;

        for user in users {
            if !user.started || user.banned {
                continue;
            }
            let res = bot
                .forward_message(UserId(user.user_id), reply.chat.id, reply.id)
                .await;

            if let Err(err) = res {
                log::error!(
                    "Failed to repost to user [{}] {}",
                    user.user_id,
                    err
                );
                if let RequestError::RetryAfter(sec) = err {
                    log::info!(
                        "Retrying to repost to user [{}] in {} seconds...",
                        user.user_id,
                        sec.seconds()
                    );
                    tokio::time::sleep(sec.duration()).await;
                    let res_again = bot
                        .forward_message(
                            UserId(user.user_id),
                            reply.chat.id,
                            reply.id,
                        )
                        .await;

                    if res_again.is_err() {
                        user_sended_error_count += 1;
                    }
                }
            } else {
                user_sended_count += 1;
                log::info!("Reposted to user [{}]", user.user_id);
            }
            tokio::time::sleep(Duration::from_millis(
                USER_SENDING_THROTTLE_TIME_MS,
            ))
            .await;
        }
    }

    let text = format!(
        "Sended users: {}\nErrors: {}",
        user_sended_count, user_sended_error_count
    );
    let _ = bot.send_message(UserId(BOT_CONFIG.creator_id), text).await;

    Ok(())
}
