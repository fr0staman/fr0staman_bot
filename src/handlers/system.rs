use crate::{
    config::env::BOT_CONFIG,
    db::DB,
    db::models::{UpdateGroups, UserStatus},
    db::shortcuts,
    keyboards,
    lang::{InnerLang, get_tag, get_tag_opt, lng, tag_one_or},
    traits::MaybeMessageSetter,
    types::{MyBot, MyResult},
};
use teloxide::{
    prelude::*,
    types::{InputFile, ReplyParameters, UpdateKind},
    utils::html::user_mention,
};
use teloxide::{types::Message, utils::html::escape};
use teloxide::{types::MessageKind, utils::html::bold};
use tokio::time::{Duration, sleep};

pub async fn handle_new_member(bot: MyBot, m: Message) -> MyResult<()> {
    let Some(new_chat_members) = m.new_chat_members() else {
        crate::myerr!("No chat member in new chat members, wtf?");
        return Ok(());
    };

    let Some(settings) = shortcuts::maybe_get_or_insert_chat(&m.chat).await?
    else {
        return Ok(());
    };

    if settings.settings == 1 {
        log::info!("New chat member in chat [{}], but silent", m.chat.id);
        return Ok(());
    }

    for user in new_chat_members {
        let ltag = tag_one_or(settings.lang.as_deref(), get_tag(user));

        if user.id == BOT_CONFIG.me.id {
            let text = lng("ChatGreetingFirst", ltag)
                .args(&[("channel", &BOT_CONFIG.channel_name)]);
            bot.send_message(m.chat.id, text).maybe_thread_id(&m).await?;
            log::info!("Bot added to chat [{}]", m.chat.id);
        } else {
            let mention = user_mention(user.id, &escape(&user.first_name));
            let chat_title = bold(&escape(m.chat.title().unwrap_or("chat")));
            let text = lng("ChatGreeting", ltag).args(&[
                ("mention", &mention),
                ("chat_title", &chat_title),
                ("channel", &BOT_CONFIG.channel_name),
            ]);
            bot.send_message(m.chat.id, text)
                .reply_parameters(ReplyParameters::new(m.id))
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
        crate::myerr!("No left chat member in left chat members, wtf?");
        return Ok(());
    };

    let Some(settings) = shortcuts::maybe_get_or_insert_chat(&m.chat).await?
    else {
        return Ok(());
    };

    if member.id == BOT_CONFIG.me.id {
        log::info!("Kicked me :( in chat [{}]", m.chat.id);
        let chat_info = UpdateGroups { active: false, ..settings.to_update() };
        DB.other.update_chat(m.chat.id.0, chat_info).await?;
        return Ok(());
    }

    if settings.settings == 1 {
        log::info!("Left chat member in chat [{}], but silent", m.chat.id);
        return Ok(());
    }

    let ltag = tag_one_or(settings.lang.as_deref(), get_tag(member));

    let mention = user_mention(member.id, &escape(&member.first_name));
    let chat_title = bold(&escape(m.chat.title().unwrap_or("chat")));
    let args = &[("mention", mention), ("chat_title", chat_title)];
    let text = lng("UserLeaveChat", ltag).args(args);
    bot.send_message(m.chat.id, text)
        .reply_parameters(ReplyParameters::new(m.id))
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

    let Some(user) =
        shortcuts::maybe_get_or_insert_user(&member.new_chat_member.user, true)
            .await?
    else {
        crate::myerr!("User not inserted!");
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
    let Some(settings) = shortcuts::maybe_get_or_insert_chat(&m.chat).await?
    else {
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

    let ltag =
        tag_one_or(settings.lang.as_deref(), get_tag_opt(m.from.as_ref()));

    let text = lng(key, ltag);
    bot.send_message(m.chat.id, text)
        .reply_parameters(ReplyParameters::new(m.id))
        .maybe_thread_id(&m)
        .await?;
    log::info!("Video chat reaction in chat [{}]", m.chat.id);

    Ok(())
}

pub async fn handle_voice_private(bot: MyBot, m: Message) -> MyResult<()> {
    let Some(from) = &m.from else { return Ok(()) };
    let Some(user) = shortcuts::maybe_get_or_insert_user(from, true).await?
    else {
        return Ok(());
    };

    let ltag = tag_one_or(user.lang.as_deref(), get_tag_opt(m.from.as_ref()));
    let text = lng("InlineHrukAddMessage", ltag);

    bot.send_message(m.chat.id, text).maybe_thread_id(&m).await?;
    let voice = m.voice().unwrap();

    bot.send_voice(
        ChatId(BOT_CONFIG.content_check_channel_id),
        InputFile::file_id(&voice.file.id),
    )
    .caption(from.id.to_string())
    .reply_markup(keyboards::keyboard_voice_check(from.id))
    .await?;

    Ok(())
}

pub async fn handle_animation_private(bot: MyBot, m: Message) -> MyResult<()> {
    let Some(from) = &m.from else { return Ok(()) };
    let Some(animation) = m.animation() else { return Ok(()) };

    let Some(user) = shortcuts::maybe_get_or_insert_user(from, true).await?
    else {
        return Ok(());
    };

    let ltag = tag_one_or(user.lang.as_deref(), get_tag_opt(m.from.as_ref()));

    let maybe_gif_with_same_id =
        DB.other.get_gif_by_file_unique_id(&animation.file.unique_id).await?;

    if maybe_gif_with_same_id.is_some() {
        let text = lng("InlineGifAlreadyExist", ltag);
        bot.send_message(m.chat.id, text).maybe_thread_id(&m).await?;
        return Ok(());
    }

    let text = lng("InlineHrukAddMessage", ltag);

    bot.send_message(m.chat.id, text).maybe_thread_id(&m).await?;

    let file = InputFile::file_id(&animation.file.id);

    bot.send_animation(ChatId(BOT_CONFIG.content_check_channel_id), file)
        .caption(from.id.to_string())
        .reply_markup(keyboards::keyboard_gif_check(from.id))
        .await?;
    Ok(())
}

pub async fn handle_chat_migration(_bot: MyBot, m: Message) -> MyResult<()> {
    let to = m.migrate_to_chat_id().unwrap();

    DB.other.update_chat_id(m.chat.id.0, to.0).await?;

    Ok(())
}
