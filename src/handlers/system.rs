use crate::{
    config::BOT_CONFIG,
    db::DB,
    keyboards::keyboard_voice_check,
    lang::{get_tag, get_tag_opt, lng, tag_one_or, InnerLang},
    models::{Groups, UpdateGroups, UserStatus},
    traits::MaybeMessageSetter,
    utils::date::get_datetime,
    MyBot, MyResult,
};
use teloxide::{
    prelude::*,
    types::{InputFile, UpdateKind},
    utils::html::user_mention,
};
use teloxide::{types::Message, utils::html::escape};
use teloxide::{types::MessageKind, utils::html::bold};
use tokio::time::{sleep, Duration};

pub async fn handle_new_member(bot: MyBot, m: Message) -> MyResult<()> {
    let Some(new_chat_members) = m.new_chat_members() else {
        log::error!("No chat member in new chat members, wtf?");
        return Ok(());
    };

    let Some(settings) = _get_or_insert_chat(m.chat.id).await? else {
        return Ok(());
    };

    if settings.settings == 1 {
        log::info!("New chat member in chat [{}], but silent", m.chat.id);
        return Ok(());
    }

    for user in new_chat_members {
        let ltag = tag_one_or(settings.lang.as_deref(), get_tag(user));

        if user.id == BOT_CONFIG.me.id {
            let text = lng("ChatGreetingFirst", ltag);
            bot.send_message(m.chat.id, text).maybe_thread_id(&m).await?;
            log::info!("Bot added to chat [{}]", m.chat.id);
        } else {
            let mention = user_mention(
                user.id.0.try_into().unwrap(),
                &escape(&user.first_name),
            );
            let chat_title = bold(&escape(m.chat.title().unwrap_or("chat")));
            let args = &[("mention", mention), ("chat_title", chat_title)];
            let text = lng("ChatGreeting", ltag).args(args);
            bot.send_message(m.chat.id, text)
                .reply_to_message_id(m.id)
                .maybe_thread_id(&m)
                .await?;
            log::info!("New chat member in chat [{}]", m.chat.id);
            sleep(Duration::from_millis(500)).await;
        }
    }

    Ok(())
}

pub async fn handle_left_member(bot: MyBot, m: Message) -> MyResult<()> {
    let Some(member) = m.left_chat_member() else {
        log::error!("No left chat member in left chat members, wtf?");
        return Ok(());
    };

    let Some(settings) = _get_or_insert_chat(m.chat.id).await? else {
        return Ok(());
    };

    if member.id == BOT_CONFIG.me.id {
        log::info!("Kicked me :( in chat [{}]", m.chat.id);
        let chat_info = UpdateGroups {
            settings: settings.settings,
            top10_setting: settings.top10_setting,
            lang: settings.lang,
            active: false,
        };
        DB.other.update_chat(m.chat.id.0, chat_info).await?;
        return Ok(());
    }

    if settings.settings == 1 {
        log::info!("Left chat member in chat [{}], but silent", m.chat.id);
        return Ok(());
    }

    let ltag = tag_one_or(settings.lang.as_deref(), get_tag(member));

    let mention = user_mention(
        member.id.0.try_into().unwrap(),
        &escape(&member.first_name),
    );
    let chat_title = bold(&escape(m.chat.title().unwrap_or("chat")));
    let args = &[("mention", mention), ("chat_title", chat_title)];
    let text = lng("UserLeaveChat", ltag).args(args);
    bot.send_message(m.chat.id, text)
        .reply_to_message_id(m.id)
        .maybe_thread_id(&m)
        .await?;

    log::info!("Left chat member in chat [{}]", m.chat.id);

    Ok(())
}

pub async fn handle_ban_or_unban_in_private(
    _bot: MyBot,
    u: Update,
) -> MyResult<()> {
    let UpdateKind::MyChatMember(member) = u.kind else { return Ok(()) };

    let is_banned = member.new_chat_member.is_banned();
    let Some(user) = DB
        .other
        .maybe_get_or_insert_user(
            member.new_chat_member.user.id.0,
            get_datetime,
        )
        .await?
    else {
        log::error!("User not inserted!");
        return Ok(());
    };

    let user_status = UserStatus {
        banned: is_banned,
        started: user.started,
        supported: user.supported,
        subscribed: user.subscribed,
    };

    DB.other.change_user_status(member.from.id.0, user_status).await?;

    if is_banned {
        log::info!("User ban bot [{}]", member.from.id);
    } else {
        log::info!("User unban bot [{}]", member.from.id);
    }
    Ok(())
}

pub async fn handle_video_chat(bot: MyBot, m: Message) -> MyResult<()> {
    let Some(settings) = _get_or_insert_chat(m.chat.id).await? else {
        return Ok(());
    };

    if settings.settings == 1 {
        log::info!("Video chat reaction in chat [{}], but silent", m.chat.id);
        return Ok(());
    }

    let key = match m.kind {
        MessageKind::VideoChatStarted(_) => "VoiceChatStartReaction",
        MessageKind::VideoChatEnded(_) => "VoiceChatEndReaction",
        _ => "EPYC",
    };

    let ltag = tag_one_or(settings.lang.as_deref(), get_tag_opt(m.from()));

    let text = lng(key, ltag);
    bot.send_message(m.chat.id, text)
        .reply_to_message_id(m.id)
        .maybe_thread_id(&m)
        .await?;
    log::info!("Video chat reaction in chat [{}]", m.chat.id);

    Ok(())
}

pub async fn handle_voice_private(bot: MyBot, m: Message) -> MyResult<()> {
    let Some(from) = m.from() else { return Ok(()) };
    let Some(user) =
        DB.other.maybe_get_or_insert_user(from.id.0, get_datetime).await?
    else {
        return Ok(());
    };

    let ltag = tag_one_or(user.lang.as_deref(), get_tag_opt(m.from()));
    let text = lng("InlineHrukAddMessage", ltag);

    bot.send_message(m.chat.id, text).maybe_thread_id(&m).await?;
    let voice = m.voice().unwrap();

    bot.send_voice(
        ChatId(BOT_CONFIG.content_check_channel_id),
        InputFile::file_id(&voice.file.id),
    )
    .caption(from.id.to_string())
    .reply_markup(keyboard_voice_check(from.id))
    .await?;

    Ok(())
}

pub async fn handle_chat_migration(_bot: MyBot, m: Message) -> MyResult<()> {
    let to = m.migrate_to_chat_id().unwrap();

    DB.other.update_chat_id(m.chat.id.0, to.0).await?;

    Ok(())
}

async fn _get_or_insert_chat(chat_id: ChatId) -> MyResult<Option<Groups>> {
    let settings = DB.other.get_chat(chat_id.0).await?;
    match settings {
        Some(s) => Ok(Some(s)),
        None => {
            let cur_datetime = get_datetime();
            DB.other.add_chat(chat_id.0, cur_datetime).await?;
            DB.other.get_chat(chat_id.0).await
        },
    }
}
