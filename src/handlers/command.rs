use ahash::AHashSet;
use futures::FutureExt;
use strum::{EnumCount, VariantArray};
use teloxide::prelude::*;
use teloxide::types::{
    ChatKind, InputFile, LinkPreviewOptions, ReplyParameters, Seconds,
};
use teloxide::utils::html::{italic, user_mention};

use crate::config::consts::{LOUDER_PREMIUM_VOICE_LIMIT, TOP_LIMIT, TOP_LIMIT_WITH_CHARTS};
use crate::config::consts::{CHAT_PIG_START_MASS, LOUDER_DEFAULT_RATIO};
use crate::config::consts::{LOUDER_DEFAULT_VOICE_LIMIT, SUBSCRIBE_GIFT};
use crate::config::env::BOT_CONFIG;
use crate::db::DB;
use crate::db::models::{GrowLogAdd, UserStatus};
use crate::db::shortcuts;
use crate::enums::MyCommands;
use crate::keyboards;
use crate::lang::{InnerLang, LocaleTag, get_tag_opt, lng, tag_one_two_or};
use crate::services::achievements::{self, Ach};
use crate::services::charts::generate_charts;
use crate::traits::{
    MaybeMessageSetter, MaybePhotoSetter, MaybeVoiceSetter,
    SimpleDisableWebPagePreview,
};
use crate::types::{MyBot, MyError, MyResult};
use crate::utils::date::{
    get_datetime, get_datetime_from_message_date, get_timediff,
};
use crate::utils::formulas::calculate_chat_pig_grow;
use crate::utils::helpers::{escape, get_file_from_stream, plural, truncate};
use crate::utils::ogg::increase_sound;
use crate::utils::text::generate_chat_top_text;

pub async fn filter_commands(
    bot: MyBot,
    m: Message,
    cmd: MyCommands,
) -> MyResult<()> {
    crate::metrics::CMD_COUNTER.inc();

    let user_info = if let Some(from) = &m.from {
        shortcuts::maybe_get_or_insert_user(from, false).await?
    } else {
        None
    };

    let chat_info = if let ChatKind::Public(_) = &m.chat.kind {
        shortcuts::maybe_get_or_insert_chat(&m.chat).await?
    } else {
        None
    };

    let ltag = tag_one_two_or(
        user_info.and_then(|c| c.lang).as_deref(),
        chat_info.and_then(|c| c.lang).as_deref(),
        get_tag_opt(m.from.as_ref()),
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
        MyCommands::Achievements => command_achievements(bot, &m, ltag).boxed(),
    };

    let response = function.await;

    let user_id = m.from.map_or(0, |u| u.id.0);

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
    let Some(from) = &m.from else { return Ok(()) };
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

        if let Some(user) = DB.other.get_user(from.id.0 as i64).await? {
            let user_status = UserStatus {
                banned: false,
                started: true,
                supported: user.supported,
                subscribed: user.subscribed,
            };
            DB.other.change_user_status(from.id.0 as i64, user_status).await?;
        } else {
            shortcuts::maybe_get_or_insert_user(from, true).await?;
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
                .reply_parameters(ReplyParameters::new(reply_id))
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
                .link_preview_options(LinkPreviewOptions::disable(true))
                .reply_parameters(ReplyParameters::new(reply.id))
                .maybe_thread_id(m)
        } else {
            bot.send_message(m.chat.id, text)
                .link_preview_options(LinkPreviewOptions::disable(true))
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
    let Some(from) = &m.from else { return Ok(()) };

    if let ChatKind::Private(_) = m.chat.kind {
        _game_only_for_chats(bot, m, ltag).await?;
        return Ok(());
    }

    let cur_datetime = get_datetime_from_message_date(m.date);
    let cur_date = cur_datetime.date();
    let mention = user_mention(from.id, &from.first_name);

    let pig = DB.chat_pig.get_chat_pig(from.id.0 as i64, m.chat.id.0).await?;
    let mut skip_date_check = false;
    let pig = if let Some(pig) = pig {
        pig
    } else {
        let Some(chat_info) =
            shortcuts::maybe_get_or_insert_chat(&m.chat).await?
        else {
            return Ok(());
        };

        let Some(user) =
            shortcuts::maybe_get_or_insert_user(from, false).await?
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
            DB.chat_pig.get_chat_pig(from.id.0 as i64, m.chat.id.0).await?
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
        .set_chat_pig_mass_n_date(
            from.id.0 as i64,
            m.chat.id.0,
            current,
            cur_date,
        )
        .await?;

    let grow_log_info = GrowLogAdd {
        game_id: pig.id,
        created_at: get_datetime(),
        current_weight: current,
        weight_change: offset,
    };

    DB.chat_pig.add_grow_log_by_game(grow_log_info).await?;

    let message = m.clone();
    let outer_bot = bot.clone();
    tokio::spawn(async move {
        let new_achievements =
            achievements::check_achievements(pig.id, cur_datetime).await;

        if let Ok(achievements) = new_achievements {
            let _ = _handle_new_achievements(
                outer_bot,
                &message,
                ltag,
                pig.id,
                pig.uid,
                achievements,
            )
            .await;
        }
    });

    let grow_status_key = format!("GamePigGrowMessage_{}", status.into_str());

    let text = lng(&grow_status_key, ltag).args(&[
        ("name", pig.name),
        ("value", offset.abs().to_string()),
        ("current", current.to_string()),
        ("mention", mention),
    ]);

    bot.send_message(m.chat.id, text)
        .link_preview_options(LinkPreviewOptions::disable(true))
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
    let Some(from) = &m.from else { return Ok(()) };

    if let ChatKind::Private(_) = m.chat.kind {
        _game_only_for_chats(bot, m, ltag).await?;
        return Ok(());
    }

    let pig = DB.chat_pig.get_chat_pig(from.id.0 as i64, m.chat.id.0).await?;
    let Some(pig) = pig else {
        _game_no_chat_pig(bot, m, ltag).await?;
        return Ok(());
    };

    let payload = escape(payload);
    if payload.is_empty() {
        let text = lng("GameNamePig", ltag).args(&[("name", &pig.name)]);
        bot.send_message(m.chat.id, text)
            .link_preview_options(LinkPreviewOptions::disable(true))
            .maybe_thread_id(m)
            .await?;
        return Ok(());
    } else if payload.len() > 64 {
        let text = lng("GameNameTagLetterLimit", ltag);
        bot.send_message(m.chat.id, text).maybe_thread_id(m).await?;
        return Ok(());
    }

    let text = lng("GameNameNewPig", ltag).args(&[("new_name", &payload)]);

    DB.chat_pig
        .set_chat_pig_name(from.id.0 as i64, m.chat.id.0, payload)
        .await?;

    bot.send_message(m.chat.id, text)
        .link_preview_options(LinkPreviewOptions::disable(true))
        .maybe_thread_id(m)
        .await?;
    Ok(())
}

async fn command_my(bot: MyBot, m: &Message, ltag: LocaleTag) -> MyResult<()> {
    let Some(from) = &m.from else { return Ok(()) };

    if let ChatKind::Private(_) = m.chat.kind {
        _game_only_for_chats(bot, m, ltag).await?;
        return Ok(());
    }

    let pig = DB.chat_pig.get_chat_pig(from.id.0 as i64, m.chat.id.0).await?;
    let Some(pig) = pig else {
        _game_no_chat_pig(bot, m, ltag).await?;
        return Ok(());
    };

    let achievements = DB.other.get_achievements_by_game_id(pig.id).await?;

    // Chance to get achievements without weight change
    if achievements.is_empty() {
        let message = m.clone();
        let outer_bot = bot.clone();
        tokio::spawn(async move {
            let cur_datetime = get_datetime_from_message_date(message.date);
            let new_achievements =
                achievements::check_achievements(pig.id, cur_datetime).await;

            if let Ok(achievements) = new_achievements {
                let _ = _handle_new_achievements(
                    outer_bot,
                    &message,
                    ltag,
                    pig.id,
                    pig.uid,
                    achievements,
                )
                .await;
            }
        });
    }

    let text = lng("GamePigStats", ltag)
        .args(&[("name", &pig.name), ("current", &pig.mass.to_string())]);
    bot.send_message(m.chat.id, text)
        .link_preview_options(LinkPreviewOptions::disable(true))
        .maybe_thread_id(m)
        .await?;
    Ok(())
}

async fn command_top(bot: MyBot, m: &Message, ltag: LocaleTag) -> MyResult<()> {
    let Some(from) = &m.from else { return Ok(()) };

    if let ChatKind::Private(_) = m.chat.kind {
        _game_only_for_chats(bot, m, ltag).await?;
        return Ok(());
    }

    let Some(chat_settings) =
        shortcuts::maybe_get_or_insert_chat(&m.chat).await?
    else {
        return Ok(());
    };

    let user = DB.other.get_user(from.id.0 as i64).await?;

    let limit = chat_settings.top10_setting;

    let with_chart = user.is_some_and(|u| u.supported);

    let top_pigs = DB
        .chat_pig
        .get_top_chat_pigs(m.chat.id.0, limit, 0, with_chart)
        .await?;

    if top_pigs.is_empty() {
        let text = lng("GameNoChatPigs", ltag);
        bot.send_message(m.chat.id, text).maybe_thread_id(m).await?;
        return Ok(());
    }

    let pig_count = DB.chat_pig.count_chat_pig(m.chat.id.0, limit).await?;

    let text = generate_chat_top_text(ltag, top_pigs, 0, with_chart);

    let is_end = pig_count < (if with_chart { TOP_LIMIT_WITH_CHARTS } else { TOP_LIMIT });
    let markup = keyboards::keyboard_top(ltag, 1, from.id, is_end);

    if with_chart {
        let data = DB.chat_pig.get_top10_by_7days_growth(m.chat.id.0).await?;

        let Some(chart) = generate_charts(
            data,
            m.chat.title().unwrap_or_default().to_string(),
            ltag,
        )
        .await
        else {
            return Err(MyError::Unknown(
                "Charts generation error".to_string(),
            ));
        };

        let file = InputFile::memory(chart);
        bot.send_photo(m.chat.id, file)
            .caption(text)
            .reply_markup(markup)
            .maybe_thread_id(m)
            .await?;
    } else {
        bot.send_message(m.chat.id, text)
            .reply_markup(markup)
            .link_preview_options(LinkPreviewOptions::disable(true))
            .maybe_thread_id(m)
            .await?;
    }
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
    let Some(from) = &m.from else { return Ok(()) };
    let user_info = DB.other.get_user(from.id.0 as i64).await?;

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
    let Some(from) = &m.from else { return Ok(()) };

    let Some(user) = DB.other.get_user(from.id.0 as i64).await? else {
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

    let Some(from) = &m.from else { return Ok(()) };
    let Ok(Some(user)) = shortcuts::maybe_get_or_insert_user(from, false).await
    else {
        return Ok(());
    };

    if user.supported {
        if voice.duration > Seconds::from_seconds(LOUDER_PREMIUM_VOICE_LIMIT) {
            let text = lng("CmdLouderLimitPremium", ltag);
            bot.send_message(m.chat.id, text).maybe_thread_id(m).await?;
            return Ok(());
        }
    } else if voice.duration > Seconds::from_seconds(LOUDER_DEFAULT_VOICE_LIMIT)
    {
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

    let Ok(as_file) = bot.get_file(voice.file.id.clone()).await else {
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

async fn command_achievements(
    bot: MyBot,
    m: &Message,
    ltag: LocaleTag,
) -> MyResult<()> {
    let Some(from) = &m.from else { return Ok(()) };

    let pig = DB.chat_pig.get_chat_pig(from.id.0 as i64, m.chat.id.0).await?;
    let Some(pig) = pig else {
        _game_no_chat_pig(bot, m, ltag).await?;
        return Ok(());
    };

    let achievements_in_all_chats =
        DB.other.get_achievements_by_uid(pig.uid).await?;

    let achievements_in_this_chat: AHashSet<_> = achievements_in_all_chats
        .iter()
        .filter_map(|v| if v.game_id == pig.id { Some(v.code) } else { None })
        .collect();

    let achievements_in_all_chats: AHashSet<_> =
        achievements_in_all_chats.iter().map(|v| v.code).collect();

    let all_count = Ach::COUNT.to_string();

    let chat_count = achievements_in_this_chat.len().to_string();
    let global_count = achievements_in_all_chats.len().to_string();

    let mention = user_mention(from.id, &from.first_name);

    let mut done_list_text = String::with_capacity(512);
    let mut not_done_list_text = String::with_capacity(512);

    for achievement in Ach::VARIANTS {
        let code = achievement.clone() as i16;
        let is_in_this_chat = achievements_in_this_chat.contains(&code);

        let is_in_global = achievements_in_all_chats.contains(&code);
        let done = is_in_this_chat || is_in_global;

        let achievement_name = lng(&format!("Achievement_{}", code), ltag);

        let one_achievement_text = lng("AchievementListOne", ltag).args(&[
            ("achievement", achievement_name.as_str()),
            ("chat_emoji", if is_in_this_chat { "✅" } else { "➖" }),
            ("global_emoji", if is_in_global { "✅" } else { "➖" }),
        ]);

        if done {
            done_list_text.push_str(&format!("   {}\n", one_achievement_text));
        } else {
            not_done_list_text.push_str(&format!("{}\n", one_achievement_text));
        }
    }

    let text = lng("AchievementList", ltag).args(&[
        ("mention", &mention),
        ("done_achievements", &done_list_text),
        ("not_done_achievements", &not_done_list_text),
        ("chat_count", &chat_count),
        ("chat_all_count", &all_count),
        ("global_count", &global_count),
        ("global_all_count", &all_count),
    ]);

    bot.send_message(m.chat.id, text).maybe_thread_id(m).await?;

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

pub async fn _handle_new_achievements(
    bot: MyBot,
    m: &Message,
    ltag: LocaleTag,
    game_id: i32,
    id_uid: i32,
    new_achievements: Vec<Ach>,
) -> MyResult<()> {
    let achievements_in_all_chats =
        DB.other.get_achievements_by_uid(id_uid).await?;

    let achievements_in_this_chat: Vec<_> = achievements_in_all_chats
        .iter()
        .filter(|v| v.game_id == game_id)
        .collect();

    let achievements_in_all_chats: AHashSet<_> =
        achievements_in_all_chats.iter().map(|v| v.code).collect();

    let all_count = Ach::COUNT.to_string();

    let chat_count = achievements_in_this_chat.len().to_string();
    let global_count = achievements_in_all_chats.len().to_string();

    for achievement in new_achievements {
        let achievement_name =
            lng(&format!("Achievement_{}", achievement as i16), ltag);

        let text = lng("NewAchievementUnlocked", ltag).args(&[
            ("achievement_name", &achievement_name),
            ("chat_count", &chat_count),
            ("chat_all_count", &all_count),
            ("global_count", &global_count),
            ("global_all_count", &all_count),
        ]);

        bot.send_message(m.chat.id, text).maybe_thread_id(m).await?;
    }

    Ok(())
}
