use futures::FutureExt;
use teloxide::prelude::*;
use teloxide::types::ChatKind;
use teloxide::utils::html::{italic, user_mention};

use crate::config::BOT_CONFIG;
use crate::consts::CHAT_PIG_START_MASS;
use crate::db::DB;
use crate::enums::MyCommands;
use crate::keyboards::{keyboard_startgroup, keyboard_top50};
use crate::lang::{get_tag_opt, lng, tag_one_two_or, InnerLang, LocaleTag};
use crate::models::UserStatus;
use crate::traits::MaybeMessageSetter;
use crate::utils::date::{get_datetime, get_timediff};
use crate::utils::formulas::calculate_chat_pig_grow;
use crate::utils::helpers::{escape, plural, truncate};
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
    };

    let response = function.await;

    let user_id = m.from().map_or(0, |u| u.id.0);

    if let Err(err) = response {
        log::error!(
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

    let text = lng("ChatGreetingFirst", ltag);

    if let ChatKind::Private(_) = m.chat.kind {
        let is_channel_member =
            bot.get_chat_member(ChatId(BOT_CONFIG.channel_id), from.id).await?;
        let key = if is_channel_member.is_present() {
            "SubscribedChannel"
        } else {
            "AdSubscribeChannel"
        };
        let text_reg = lng(key, ltag);

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
            let cur_datetime = get_datetime();
            DB.other.register_user(from.id.0, true, cur_datetime).await?;
        };

        bot.send_message(m.chat.id, text_reg).maybe_thread_id(m).await?;

        let url = BOT_CONFIG.me.tme_url();
        bot.send_message(m.chat.id, text)
            .maybe_thread_id(m)
            .reply_markup(keyboard_startgroup(ltag, url))
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

    if let Some(reply) = m.reply_to_message().cloned() {
        let bot = bot.clone();
        tokio::spawn(async move {
            let text = lng("YouPidor", ltag);
            let _ = bot
                .send_message(chat_id, text)
                .reply_to_message_id(reply.id)
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
        DB.other.add_chat(m.chat.id.0, get_datetime()).await?;

        let Some(chat_info) = DB.other.get_chat(m.chat.id.0).await? else {
            return Ok(());
        };

        let Some(user) =
            DB.other.maybe_get_or_insert_user(from.id.0, get_datetime).await?
        else {
            return Ok(());
        };

        let truncated_f_name = truncate(&from.first_name, 64).0;
        DB.chat_pig
            .create_chat_pig(
                user.id,
                chat_info.id,
                truncated_f_name,
                cur_date,
                CHAT_PIG_START_MASS,
            )
            .await?;
        let maybe_new_pig =
            DB.chat_pig.get_chat_pig(from.id.0, m.chat.id.0).await?;
        if let Some(new_pig) = maybe_new_pig {
            skip_date_check = true;
            let text =
                lng("GameStartGreeting", ltag).args(&[("mention", &mention)]);
            bot.send_message(m.chat.id, text).maybe_thread_id(m).await?;
            new_pig
        } else {
            return Ok(());
        }
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

    let chat_settings = DB.other.get_chat(m.chat.id.0).await?;

    let chat_settings = if let Some(chat_settings) = chat_settings {
        chat_settings
    } else {
        let cur_datetime = get_datetime();
        DB.other.add_chat(m.chat.id.0, cur_datetime).await?;
        let chat_settings = DB.other.get_chat(m.chat.id.0).await?;
        let Some(chat_settings) = chat_settings else {
            return Ok(());
        };
        chat_settings
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
    let markup = keyboard_top50(ltag, 1, from.id, is_end);
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
    let url = BOT_CONFIG.me.tme_url();
    let text = lng("GameAboutMessage", ltag);
    let markup = keyboard_startgroup(ltag, url);
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

async fn _game_only_for_chats(
    bot: MyBot,
    m: &Message,
    ltag: LocaleTag,
) -> MyResult<()> {
    let url = BOT_CONFIG.me.tme_url();
    let text = lng("GameOnlyForChats", ltag);
    let markup = keyboard_startgroup(ltag, url);
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
