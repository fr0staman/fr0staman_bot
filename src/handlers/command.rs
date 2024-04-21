use futures::FutureExt;
use teloxide::prelude::*;
use teloxide::types::{ChatKind, InputFile};
use teloxide::utils::html::{italic, user_mention};

use crate::config::BOT_CONFIG;
use crate::consts::LOUDER_PREMIUM_VOICE_LIMIT;
use crate::consts::{CHAT_PIG_START_MASS, LOUDER_DEFAULT_RATIO};
use crate::consts::{LOUDER_DEFAULT_VOICE_LIMIT, SUBSCRIBE_GIFT};
use crate::db::DB;
use crate::enums::MyCommands;
use crate::keyboards;
use crate::lang::{get_tag_opt, lng, tag_one_two_or, InnerLang, LocaleTag};
use crate::models::UserStatus;
use crate::traits::{MaybeMessageSetter, MaybeVoiceSetter};
use crate::utils::date::{get_datetime, get_timediff};
use crate::utils::db_shortcuts;
use crate::utils::formulas::calculate_chat_pig_grow;
use crate::utils::helpers::{escape, get_file_from_stream, plural, truncate};
use crate::utils::ogg::increase_sound;
use crate::utils::text::generate_chat_top50_text;
use crate::{MyBot, MyResult};

pub async fn filter_commands(
    bot: MyBot,
    m: Message,
    cmd: MyCommands,
) -> MyResult<()> {
    crate::metrics::CMD_COUNTER.inc();

    let user_info = if let Some(from) = m.from() {
        DB.other.get_user(from.id.0).await?
    } else {
        None
    };

    let chat_info = DB.other.get_chat(m.chat.id.0).await?;

    let ltag = tag_one_two_or(
        user_info.and_then(|c| c.lang).as_deref(),
        chat_info.and_then(|c| c.lang).as_deref(),
        get_tag_opt(m.from()),
    );

    let function = match &cmd {
        MyCommands::Start => command_start(bot, &m, ltag).boxed(),
        MyCommands::Help => command_help(bot, &m, ltag).boxed(),
        MyCommands::Pidor => command_pidor(bot, &m, ltag).boxed(),
        MyCommands::Print(arg) | MyCommands::P(arg) => {
            command_print(bot, &m, ltag, arg).boxed()
        },
        MyCommands::Grow => command_grow(bot, &m, ltag).boxed(),
        MyCommands::Name(arg) => command_name(bot, &m, ltag, arg).boxed(),
        MyCommands::My => command_my(bot, &m, ltag).boxed(),
        MyCommands::Top => command_top(bot, &m, ltag).boxed(),
        MyCommands::Game => command_game(bot, &m, ltag).boxed(),
        MyCommands::Lang => command_lang(bot, &m, ltag).boxed(),
        MyCommands::Id => command_id(bot, &m, ltag).boxed(),
        MyCommands::Louder => command_louder(bot, &m, ltag).boxed(),
    };

    let response = function.await;

    let user_id = m.from().map_or(0, |u| u.id.0);

    if let Err(err) = response {
        crate::myerr!(
            "Error {:?}: command /{:?}: user [{}] in chat [{}]",
            err,
            cmd,
            user_id,
            m.chat.id,
        );
    } else {
        log::info!(
            "Handled command /{:?}: user [{}] in chat [{}]",
            cmd,
            user_id,
            m.chat.id,
        );
    }

    Ok(())
}

async fn command_start(
    bot: MyBot,
    m: &Message,
    ltag: LocaleTag,
) -> MyResult<()> {
    let Some(from) = m.from() else { return Ok(()) };
    crate::metrics::CMD_START_COUNTER.inc();

    let text = lng("ChatGreetingFirst", ltag)
        .args(&[("channel", &BOT_CONFIG.channel_name)]);

    if let ChatKind::Private(_) = m.chat.kind {
        let is_channel_member =
            bot.get_chat_member(ChatId(BOT_CONFIG.channel_id), from.id).await?;
        let key = if is_channel_member.is_present() {
            "SubscribedChannel"
        } else {
            "AdSubscribeChannel"
        };
        let text_reg = lng(key, ltag).args(&[
            ("amount", &SUBSCRIBE_GIFT.to_string()),
            ("channel", &BOT_CONFIG.channel_name),
        ]);

        let probably_user = DB.other.get_user(from.id.0).await?;
        if let Some(user) = probably_user {
            let user_status = UserStatus {
                banned: false,
                started: true,
                supported: user.supported,
                subscribed: user.subscribed,
            };
            DB.other.change_user_status(from.id.0, user_status).await?;
        } else {
            db_shortcuts::maybe_get_or_insert_user(from, true).await?;
        };

        bot.send_message(m.chat.id, text_reg).maybe_thread_id(m).await?;

        bot.send_message(m.chat.id, text)
            .maybe_thread_id(m)
            .reply_markup(keyboards::keyboard_startgroup(ltag))
            .await?;
        return Ok(());
    } else if let ChatKind::Public(_) = m.chat.kind {
        // If user send bot to chat by startgroup, bot already receives update about adding him to chat
        // So as not to send two messages at the same time, I just skip this message
        // by checking, if its a start from startgroup deeplink

        let is_from_deeplink = m.text().is_some_and(|t| t.contains("inline"));

        if is_from_deeplink {
            return Ok(());
        }
    };

    bot.send_message(m.chat.id, text).maybe_thread_id(m).await?;

    Ok(())
}

async fn command_help(
    bot: MyBot,
    m: &Message,
    ltag: LocaleTag,
) -> MyResult<()> {
    crate::metrics::CMD_HELP_COUNTER.inc();
    let link = lng("HelpLink", ltag);
    let text = lng("HelpMessage", ltag).args(&[("link", link)]);
    bot.send_message(m.chat.id, text).maybe_thread_id(m).await?;
    Ok(())
}

async fn command_pidor(
    bot: MyBot,
    m: &Message,
    ltag: LocaleTag,
) -> MyResult<()> {
    let chat_id = m.chat.id;
    let m_id = m.id;

    if let Some(reply_id) = m.reply_to_message().map(|mr| mr.id) {
        let bot = bot.clone();
        let m = m.clone();
        tokio::spawn(async move {
            let text = lng("YouPidor", ltag);
            let _ = bot
                .send_message(chat_id, text)
                .reply_to_message_id(reply_id)
                .maybe_thread_id(&m)
                .await;
        });
    }

    tokio::spawn(async move {
        let _ = bot.delete_message(chat_id, m_id).await;
    });

    Ok(())
}

async fn command_print(
    bot: MyBot,
    m: &Message,
    ltag: LocaleTag,
    arg: &str,
) -> MyResult<()> {
    if arg.is_empty() {
        let text = lng("ErrorTextAsArgument", ltag);
        bot.send_message(m.chat.id, text).maybe_thread_id(m).await?;
    } else {
        let truncated = truncate(arg, 4096);
        let text = italic(truncated.0);
        let request = if let Some(reply) = m.reply_to_message() {
            bot.send_message(m.chat.id, text)
                .disable_web_page_preview(true)
                .reply_to_message_id(reply.id)
                .maybe_thread_id(m)
        } else {
            bot.send_message(m.chat.id, text)
                .disable_web_page_preview(true)
                .maybe_thread_id(m)
        };

        tokio::spawn(async move {
            let res = request.await;
            if res.is_err() {
                log::error!("Print error {:?}", res);
            }
        });
    }

    let chat_id = m.chat.id;
    let m_id = m.id;
    tokio::spawn(async move {
        let _ = bot.delete_message(chat_id, m_id).await;
    });

    Ok(())
}

async fn command_grow(
    bot: MyBot,
    m: &Message,
    ltag: LocaleTag,
) -> MyResult<()> {
    let Some(from) = m.from() else { return Ok(()) };

    if let ChatKind::Private(_) = m.chat.kind {
        _game_only_for_chats(bot, m, ltag).await?;
        return Ok(());
    }

    let cur_datetime = get_datetime();
    let cur_date = cur_datetime.date();
    let mention = user_mention(
        i64::try_from(from.id.0).unwrap_or_default(),
        &from.first_name,
    );

    let pig = DB.chat_pig.get_chat_pig(from.id.0, m.chat.id.0).await?;
    let mut skip_date_check = false;
    let pig = if let Some(pig) = pig {
        pig
    } else {
        let Some(chat_info) =
            db_shortcuts::maybe_get_or_insert_chat(&m.chat).await?
        else {
            return Ok(());
        };

        let Some(user) =
            db_shortcuts::maybe_get_or_insert_user(from, false).await?
        else {
            return Ok(());
        };

        let escaped_f_name = escape(&from.first_name);
        let f_name = truncate(&escaped_f_name, 64).0;
        DB.chat_pig
            .create_chat_pig(
                user.id,
                chat_info.id,
                f_name,
                cur_date,
                CHAT_PIG_START_MASS,
            )
            .await?;

        let Some(new_pig) =
            DB.chat_pig.get_chat_pig(from.id.0, m.chat.id.0).await?
        else {
            return Ok(());
        };

        skip_date_check = true;
        let text =
            lng("GameStartGreeting", ltag).args(&[("mention", &mention)]);
        bot.send_message(m.chat.id, text).maybe_thread_id(m).await?;
        new_pig
    };

    if (!skip_date_check) && (pig.date == cur_date) {
        let (hours, minutes, seconds) = get_timediff(cur_datetime);

        let h_rule = plural(hours);
        let m_rule = plural(minutes);
        let s_rule = plural(seconds);

        let h_lang = format!("hour_{}", h_rule);
        let m_lang = format!("minute_{}", m_rule);
        let s_lang = format!("second_{}", s_rule);

        let hours_text = format!("{} {}", hours, lng(&h_lang, ltag));
        let minut_text = format!("{} {}", minutes, lng(&m_lang, ltag));

        let next_feed = if hours != 0 {
            lng("GameNextFeedingToHoursMinutes", ltag)
                .args(&[("to_hours", &hours_text), ("to_minutes", &minut_text)])
        } else if minutes != 0 {
            lng("GameNextFeedingToMinutes", ltag)
                .args(&[("to_minutes", &minut_text)])
        } else {
            let secnd_text = format!("{} {}", seconds, lng(&s_lang, ltag));

            lng("GameNextFeedingToSeconds", ltag)
                .args(&[("to_seconds", &secnd_text)])
        };

        let text = lng("GameAlreadyFeeded", ltag)
            .args(&[("next_feed", &italic(&next_feed))]);
        bot.send_message(m.chat.id, text).maybe_thread_id(m).await?;
        return Ok(());
    }

    let (offset, status) = calculate_chat_pig_grow(pig.mass);
    let current = pig.mass + offset;

    DB.chat_pig
        .set_chat_pig_mass_n_date(from.id.0, m.chat.id.0, current, cur_date)
        .await?;

    let grow_status_key = format!("GamePigGrowMessage_{}", status.as_ref());

    let text = lng(&grow_status_key, ltag).args(&[
        ("name", pig.name),
        ("value", offset.abs().to_string()),
        ("current", current.to_string()),
        ("mention", mention),
    ]);

    bot.send_message(m.chat.id, text)
        .disable_web_page_preview(true)
        .maybe_thread_id(m)
        .await?;
    Ok(())
}

async fn command_name(
    bot: MyBot,
    m: &Message,
    ltag: LocaleTag,
    payload: &str,
) -> MyResult<()> {
    let Some(from) = m.from() else { return Ok(()) };

    if let ChatKind::Private(_) = m.chat.kind {
        _game_only_for_chats(bot, m, ltag).await?;
        return Ok(());
    }

    let pig = DB.chat_pig.get_chat_pig(from.id.0, m.chat.id.0).await?;
    let Some(pig) = pig else {
        _game_no_chat_pig(bot, m, ltag).await?;
        return Ok(());
    };

    let payload = escape(payload);
    if payload.is_empty() {
        let text = lng("GameNamePig", ltag).args(&[("name", &pig.name)]);
        bot.send_message(m.chat.id, text)
            .maybe_thread_id(m)
            .disable_web_page_preview(true)
            .await?;
        return Ok(());
    } else if payload.len() > 64 {
        let text = lng("GameNameTagLetterLimit", ltag);
        bot.send_message(m.chat.id, text).maybe_thread_id(m).await?;
        return Ok(());
    }

    let text = lng("GameNameNewPig", ltag).args(&[("new_name", &payload)]);

    DB.chat_pig.set_chat_pig_name(from.id.0, m.chat.id.0, payload).await?;

    bot.send_message(m.chat.id, text)
        .disable_web_page_preview(true)
        .maybe_thread_id(m)
        .await?;
    Ok(())
}

async fn command_my(bot: MyBot, m: &Message, ltag: LocaleTag) -> MyResult<()> {
    let Some(from) = m.from() else { return Ok(()) };

    if let ChatKind::Private(_) = m.chat.kind {
        _game_only_for_chats(bot, m, ltag).await?;
        return Ok(());
    }

    let pig = DB.chat_pig.get_chat_pig(from.id.0, m.chat.id.0).await?;
    let Some(pig) = pig else {
        _game_no_chat_pig(bot, m, ltag).await?;
        return Ok(());
    };

    let text = lng("GamePigStats", ltag)
        .args(&[("name", &pig.name), ("current", &pig.mass.to_string())]);
    bot.send_message(m.chat.id, text)
        .disable_web_page_preview(true)
        .maybe_thread_id(m)
        .await?;
    Ok(())
}

async fn command_top(bot: MyBot, m: &Message, ltag: LocaleTag) -> MyResult<()> {
    let Some(from) = m.from() else { return Ok(()) };

    if let ChatKind::Private(_) = m.chat.kind {
        _game_only_for_chats(bot, m, ltag).await?;
        return Ok(());
    }

    let Some(chat_settings) =
        db_shortcuts::maybe_get_or_insert_chat(&m.chat).await?
    else {
        return Ok(());
    };

    let limit = chat_settings.top10_setting;

    let top50_pigs =
        DB.chat_pig.get_top50_chat_pigs(m.chat.id.0, limit, 0).await?;

    if top50_pigs.is_empty() {
        let text = lng("GameNoChatPigs", ltag);
        bot.send_message(m.chat.id, text).maybe_thread_id(m).await?;
        return Ok(());
    }

    let pig_count = DB.chat_pig.count_chat_pig(m.chat.id.0, limit).await?;

    let text = generate_chat_top50_text(ltag, top50_pigs, 0);

    let is_end = pig_count < 50;
    let markup = keyboards::keyboard_top50(ltag, 1, from.id, is_end);
    bot.send_message(m.chat.id, text)
        .reply_markup(markup)
        .disable_web_page_preview(true)
        .maybe_thread_id(m)
        .await?;

    Ok(())
}

async fn command_game(
    bot: MyBot,
    m: &Message,
    ltag: LocaleTag,
) -> MyResult<()> {
    let text = lng("GameAboutMessage", ltag);
    let markup = keyboards::keyboard_startgroup(ltag);
    bot.send_message(m.chat.id, text)
        .reply_markup(markup)
        .maybe_thread_id(m)
        .await?;

    Ok(())
}

async fn command_lang(
    bot: MyBot,
    m: &Message,
    ltag: LocaleTag,
) -> MyResult<()> {
    let Some(from) = m.from() else { return Ok(()) };
    let user_info = DB.other.get_user(from.id.0).await?;

    let is_not_public = m.chat.is_private() || m.chat.is_channel();
    let chat_info = if is_not_public {
        None
    } else {
        DB.other.get_chat(m.chat.id.0).await?
    };

    let maybe_user_lang = user_info.and_then(|u| u.lang);
    let maybe_chat_lang = chat_info.and_then(|c| c.lang);

    let user_lang = maybe_user_lang.as_deref().unwrap_or("-");
    let chat_lang = maybe_chat_lang.as_deref().unwrap_or("-");
    let client_lang = from.language_code.as_deref().unwrap_or("-");

    let append = if is_not_public {
        "".to_string()
    } else {
        lng("UserCommandLangPublicMessage", ltag)
            .args(&[("chat_lang", chat_lang)])
            + "\n"
    };

    let text = append
        + &lng("UserCommandLangMessage", ltag)
            .args(&[("user_lang", user_lang), ("client_lang", client_lang)]);
    bot.send_message(m.chat.id, text).maybe_thread_id(m).await?;
    Ok(())
}

async fn command_id(bot: MyBot, m: &Message, ltag: LocaleTag) -> MyResult<()> {
    let Some(from) = m.from() else { return Ok(()) };

    let Some(user) = DB.other.get_user(from.id.0).await? else {
        return Ok(());
    };
    let text = lng("UserCommandIdMessage", ltag).args(&[("id", user.id)]);
    bot.send_message(m.chat.id, text).maybe_thread_id(m).await?;
    Ok(())
}

async fn command_louder(
    bot: MyBot,
    m: &Message,
    ltag: LocaleTag,
) -> MyResult<()> {
    let Some(reply) = m.reply_to_message() else {
        let text = lng("CmdLouderNoReply", ltag);
        bot.send_message(m.chat.id, text).maybe_thread_id(m).await?;
        return Ok(());
    };

    let Some(voice) = reply.voice() else {
        let text = lng("CmdLouderNoVoice", ltag);
        bot.send_message(m.chat.id, text).maybe_thread_id(m).await?;
        return Ok(());
    };

    let Some(from) = m.from() else { return Ok(()) };
    let Ok(Some(user)) =
        db_shortcuts::maybe_get_or_insert_user(from, false).await
    else {
        return Ok(());
    };

    if user.supported {
        if voice.duration > LOUDER_PREMIUM_VOICE_LIMIT {
            let text = lng("CmdLouderLimitPremium", ltag);
            bot.send_message(m.chat.id, text).maybe_thread_id(m).await?;
            return Ok(());
        }
    } else if voice.duration > LOUDER_DEFAULT_VOICE_LIMIT {
        let text = lng("CmdLouderLimitDefault", ltag);
        bot.send_message(m.chat.id, text).maybe_thread_id(m).await?;
        return Ok(());
    };

    let splitted = m.text().unwrap_or("").split_once(' ');

    let volume_factor = if let Some((_, volume)) = splitted {
        match volume.parse() {
            Ok(value) => value,
            _ => {
                let text = lng("CmdLouderParseError", ltag);
                bot.send_message(m.chat.id, text).maybe_thread_id(m).await?;
                return Ok(());
            },
        }
    } else {
        LOUDER_DEFAULT_RATIO
    };

    let Ok(as_file) = bot.get_file(&voice.file.id).await else {
        let text = lng("CmdLouderErrorFileDownload", ltag);
        bot.send_message(m.chat.id, text).maybe_thread_id(m).await?;
        return Ok(());
    };
    let Some(voice_content) = get_file_from_stream(&bot, &as_file).await else {
        let text = lng("CmdLouderErrorFileDownload", ltag);
        bot.send_message(m.chat.id, text).maybe_thread_id(m).await?;
        return Ok(());
    };

    let Some(new_voice) = increase_sound(voice_content, volume_factor).await
    else {
        let text = lng("CmdLouderFailedProcess", ltag);
        let _ = bot.send_message(m.chat.id, text).maybe_thread_id(m).await;
        return Ok(());
    };

    let res = bot
        .send_voice(m.chat.id, InputFile::memory(new_voice))
        .maybe_thread_id(m)
        .await;

    if res.is_err() {
        let text = lng("CmdLouderFailedSend", ltag);
        bot.send_message(m.chat.id, text).maybe_thread_id(m).await?;
    };

    Ok(())
}

async fn _game_only_for_chats(
    bot: MyBot,
    m: &Message,
    ltag: LocaleTag,
) -> MyResult<()> {
    let text = lng("GameOnlyForChats", ltag);
    let markup = keyboards::keyboard_startgroup(ltag);
    bot.send_message(m.chat.id, text)
        .reply_markup(markup)
        .maybe_thread_id(m)
        .await?;
    Ok(())
}

async fn _game_no_chat_pig(
    bot: MyBot,
    m: &Message,
    ltag: LocaleTag,
) -> MyResult<()> {
    let text = lng("GameNoChatPigs", ltag);
    bot.send_message(m.chat.id, text).maybe_thread_id(m).await?;
    Ok(())
}
