use futures::FutureExt;
use teloxide::prelude::*;

use crate::{
    config::consts::HAND_PIG_ADDITION_ON_SUPPORTED,
    db::DB,
    db::models::UserStatus,
    enums::AdminCommands,
    lang::{InnerLang, LocaleTag, get_tag, lng, tag_one_or},
    traits::MaybeMessageSetter,
    types::{MyBot, MyResult},
    utils::{date::get_date, formulas::calculate_hryak_size, helpers},
};

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
