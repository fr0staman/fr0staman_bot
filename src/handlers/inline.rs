use std::str::FromStr;

use async_recursion::async_recursion;
use futures::FutureExt;

use teloxide::payloads::AnswerInlineQuerySetters;
use teloxide::types::{ChatType, InlineQueryResultVoice, UserId};
use teloxide::utils::html::{bold, italic};
use teloxide::{
    requests::Requester,
    types::{
        InlineQuery, InlineQueryResult, InlineQueryResultArticle,
        InputMessageContent, InputMessageContentText,
    },
};

use crate::consts::{BOT_PARSE_MODE, DEFAULT_LANG_TAG, INLINE_QUERY_LIMIT};
use crate::db::DB;
use crate::enums::{Image, InlineCommands, InlineKeywords};
use crate::lang::{get_langs, get_tag, lng, tag_one_or, InnerLang, LocaleTag};
use crate::models::{
    InlineUser, InlineVoice, NewInlineUser, UpdateInlineUser, User,
};
use crate::types::MyBot;
use crate::utils::date::{get_date, get_datetime};
use crate::utils::flag::Flags;
use crate::utils::helpers::{get_photostock, truncate};
use crate::utils::{formulas, helpers};
use crate::{keyboards, MyError, MyResult};

pub async fn filter_inline_commands(
    bot: MyBot,
    q: InlineQuery,
) -> MyResult<()> {
    crate::metrics::INLINE_COUNTER.inc();
    let user = DB.other.get_user(q.from.id.0).await?;
    let ltag =
        tag_one_or(user.and_then(|u| u.lang).as_deref(), get_tag(&q.from));

    let temp_bot = bot.clone();

    let split_command = q.query.split_once(' ');

    let function = match split_command {
        Some((action, payload)) => match InlineCommands::from_str(action) {
            Ok(cmd) => match cmd {
                InlineCommands::Name => {
                    inline_rename_hrundel(bot, &q, ltag, payload).boxed()
                },
                InlineCommands::Hru => {
                    inline_hruks(bot, &q, ltag, payload).boxed()
                },
                InlineCommands::Flag => {
                    inline_flag(bot, &q, ltag, payload).boxed()
                },
            },
            Err(_) => inline_hrundel(bot, &q, ltag).boxed(),
        },
        None => match InlineKeywords::from_str(&q.query) {
            Ok(kwd) => match kwd {
                InlineKeywords::Name => {
                    inline_name_hrundel(bot, &q, ltag).boxed()
                },
                InlineKeywords::DayPig => inline_day_pig(bot, &q, ltag).boxed(),
                InlineKeywords::OC => inline_oc_stats(bot, &q, ltag).boxed(),
                InlineKeywords::Hru => inline_hruks(bot, &q, ltag, "").boxed(),
                InlineKeywords::Flag => inline_flag(bot, &q, ltag, "").boxed(),
                InlineKeywords::Lang => inline_lang(bot, &q, ltag).boxed(),
            },
            Err(_) => inline_hrundel(bot, &q, ltag).boxed(),
        },
    };

    let response = function.await;

    if let Err(err) = response {
        handle_error(temp_bot, q, ltag, err).await;
    } else {
        handle_good(q).await;
    }

    Ok(())
}

async fn inline_hrundel(
    bot: MyBot,
    q: &InlineQuery,
    ltag: LocaleTag,
) -> MyResult<()> {
    let show_query = _get_hryak(q, ltag).await?;

    let results = show_query
        .into_iter()
        .map(InlineQueryResult::Article)
        .collect::<Vec<_>>();

    bot.answer_inline_query(&q.id, results).cache_time(0).await?;
    Ok(())
}

#[async_recursion]
async fn _get_hryak(
    q: &InlineQuery,
    ltag: LocaleTag,
) -> MyResult<Vec<InlineQueryResultArticle>> {
    let hrundel_info = DB.hand_pig.get_hrundel(q.from.id.0).await?;
    let cur_date = get_date();

    let Some(info) = hrundel_info else {
        let biggest_mass = _get_biggest_chat_pig_mass(q.from.id).await?;

        let size = formulas::calculate_hryak_size(q.from.id.0) + biggest_mass;
        let truncated_f_name = truncate(&q.from.first_name, 64);
        let Some(user) = DB
            .other
            .maybe_get_or_insert_user(q.from.id.0, get_datetime)
            .await?
        else {
            return Ok(vec![handle_error_info(ltag)]);
        };

        let hrundel = NewInlineUser {
            uid: user.id,
            f_name: truncated_f_name.0,
            weight: size,
            date: cur_date,
            flag: q.from.language_code.as_deref().unwrap_or(DEFAULT_LANG_TAG),
            name: truncated_f_name.0,
        };
        DB.hand_pig.add_hrundel(hrundel).await?;
        return _get_hryak(q, ltag).await;
    };

    if info.0.date != cur_date {
        // Pig exist, but not "today", just recreate that!
        let size = formulas::calculate_hryak_size(q.from.id.0);
        let biggest_mass = _get_biggest_chat_pig_mass(q.from.id).await?;
        let add = biggest_mass + helpers::mass_addition_on_status(&info.1);

        let truncated_f_name = truncate(&q.from.first_name, 64).0;

        let update_data = UpdateInlineUser {
            id: info.0.id,
            f_name: truncated_f_name,
            weight: size + add,
            date: cur_date,
            gifted: false,
        };

        DB.hand_pig.update_hrundel(update_data).await?;
        return _get_hryak(q, ltag).await;
    }

    let (chat_type, remove_markup, to) =
        get_accesibility_by_chattype(q.chat_type);
    let text = _get_for_top10_info(ltag, chat_type).await?;

    let result = vec![
        get_start_duel(ltag, q.from.id, &info.0),
        get_top10_info(ltag, q.from.id, text, to),
        get_hryak_info(ltag, q.from.id, &info, remove_markup),
        get_more_info(ltag),
    ];

    Ok(result)
}

async fn inline_name_hrundel(
    bot: MyBot,
    q: &InlineQuery,
    ltag: LocaleTag,
) -> MyResult<()> {
    let Some(hrundel) = DB.hand_pig.get_hrundel(q.from.id.0).await? else {
        let results = InlineQueryResult::Article(handle_no_results(ltag));
        bot.answer_inline_query(&q.id, vec![results]).cache_time(0).await?;
        return Ok(());
    };

    let article = name_hryak_info(ltag, hrundel.0.name);
    let results = vec![InlineQueryResult::Article(article)];

    bot.answer_inline_query(&q.id, results).cache_time(0).await?;
    Ok(())
}

async fn inline_rename_hrundel(
    bot: MyBot,
    q: &InlineQuery,
    ltag: LocaleTag,
    new_name: &str,
) -> MyResult<()> {
    let Some(hrundel) = DB.hand_pig.get_hrundel(q.from.id.0).await? else {
        let results = InlineQueryResult::Article(handle_no_results(ltag));
        bot.answer_inline_query(&q.id, vec![results]).cache_time(0).await?;
        return Ok(());
    };

    let article = rename_hryak_info(ltag, q.from.id, hrundel.0.name, new_name);
    let results = vec![InlineQueryResult::Article(article)];

    bot.answer_inline_query(&q.id, results).cache_time(0).await?;
    Ok(())
}

async fn inline_day_pig(
    bot: MyBot,
    q: &InlineQuery,
    ltag: LocaleTag,
) -> MyResult<()> {
    let article = day_pig_info(ltag, q.from.id, q.chat_type);
    let results = vec![InlineQueryResult::Article(article)];

    bot.answer_inline_query(&q.id, results).cache_time(0).await?;
    Ok(())
}

async fn inline_oc_stats(
    bot: MyBot,
    q: &InlineQuery,
    ltag: LocaleTag,
) -> MyResult<()> {
    let user_id = q.from.id.0;
    let hryak_size = formulas::calculate_hryak_size(user_id);

    let cpu_clock = formulas::calculate_cpu_clock(hryak_size, user_id);
    let ram_clock = formulas::calculate_ram_clock(hryak_size, user_id);
    let gpu_hashr = formulas::calculate_gpu_hashrate(hryak_size, user_id);

    let results = vec![
        InlineQueryResult::Article(cpu_oc_info(ltag, cpu_clock)),
        InlineQueryResult::Article(ram_oc_info(ltag, ram_clock)),
        InlineQueryResult::Article(gpu_oc_info(ltag, gpu_hashr)),
    ];

    bot.answer_inline_query(&q.id, results).cache_time(0).await?;
    Ok(())
}

async fn inline_hruks(
    bot: MyBot,
    q: &InlineQuery,
    ltag: LocaleTag,
    payload: &str,
) -> MyResult<()> {
    let voices: Vec<InlineVoice> = if payload.is_empty() {
        DB.other.get_inline_voices().await?
    } else {
        let Ok(id) = payload.parse::<i16>() else {
            bot.answer_inline_query(
                &q.id,
                vec![InlineQueryResult::Article(handle_error_parse(ltag))],
            )
            .await?;
            return Ok(());
        };

        let voice = DB.other.get_inline_voice_by_id(id).await?;
        voice.into_iter().collect()
    };

    if voices.is_empty() {
        let result = InlineQueryResult::Article(handle_no_results(ltag));
        bot.answer_inline_query(&q.id, vec![result]).await?;
        return Ok(());
    }

    let number_from_offset = q.offset.parse::<usize>().unwrap_or(0);

    let (start_index, end_index) = {
        const ON_PAGE: usize = INLINE_QUERY_LIMIT;
        let start_index = ON_PAGE * number_from_offset;
        let probably_end_index = start_index + ON_PAGE;

        (start_index, probably_end_index.min(voices.len()))
    };

    let url = "https://t.me".parse::<url::Url>().unwrap();

    let paged_voices = &voices[start_index..end_index];
    let results: Vec<InlineQueryResult> = paged_voices
        .iter()
        .map(|item| {
            let caption = lng("InlineHrukCaptionNumber", ltag)
                .args(&[("number", &item.id.to_string())]);
            let voice_url = url.join(&item.url).unwrap_or_else(|_| url.clone());
            InlineQueryResult::Voice(InlineQueryResultVoice::new(
                item.id.to_string(),
                voice_url,
                caption,
            ))
        })
        .collect();

    let query = bot.answer_inline_query(&q.id, results).cache_time(30);

    if end_index != voices.len() {
        let next_offset = (number_from_offset + 1).to_string();
        query.next_offset(next_offset).await?;
    } else {
        query.await?;
    };
    Ok(())
}

async fn inline_flag(
    bot: MyBot,
    q: &InlineQuery,
    ltag: LocaleTag,
    payload: &str,
) -> MyResult<()> {
    let Some(user) = DB.hand_pig.get_hrundel(q.from.id.0).await? else {
        let results = InlineQueryResult::Article(handle_no_results(ltag));
        bot.answer_inline_query(&q.id, vec![results]).cache_time(0).await?;
        return Ok(());
    };

    let old_flag = Flags::from_code(&user.0.flag).unwrap_or(Flags::Us);
    let mut results = Vec::with_capacity(64);

    if q.offset.is_empty() {
        let start_info =
            InlineQueryResult::Article(flag_info(ltag, old_flag.to_emoji()));

        results.push(start_info);
    }

    let number_from_offset = q.offset.parse::<usize>().unwrap_or(0);

    let searched_flags: Vec<_> = if payload.is_empty() {
        Flags::FLAGS.to_vec()
    } else {
        Flags::FLAGS
            .into_iter()
            .filter(|i| {
                if i.to_code().contains(payload)
                    || i.to_emoji().contains(payload)
                {
                    return true;
                }
                false
            })
            .collect()
    };

    let (start_index, end_index) = {
        const ON_PAGE: usize = INLINE_QUERY_LIMIT - 1;
        let start_index = ON_PAGE * number_from_offset;
        let probably_end_index = start_index + ON_PAGE;

        (start_index, probably_end_index.min(searched_flags.len()))
    };

    let selected_flags = &searched_flags[start_index..end_index];
    if selected_flags.is_empty() {
        let empty_info = flag_empty_info(ltag);
        results.push(InlineQueryResult::Article(empty_info));
    } else {
        for (idx, new_flag) in selected_flags.iter().enumerate() {
            let number = idx + start_index;

            let info =
                flag_change_info(ltag, q.from.id, old_flag, *new_flag, number);
            results.push(InlineQueryResult::Article(info));
        }
    }

    let new_offset = number_from_offset + 1;
    let query = bot.answer_inline_query(&q.id, results).cache_time(0);

    if end_index != searched_flags.len() {
        query.next_offset(new_offset.to_string()).await?;
    } else {
        query.await?;
    }
    Ok(())
}

async fn inline_lang(
    bot: MyBot,
    q: &InlineQuery,
    ltag: LocaleTag,
) -> MyResult<()> {
    let Some(user) = DB.other.get_user(q.from.id.0).await? else {
        let results = InlineQueryResult::Article(handle_no_results(ltag));
        bot.answer_inline_query(&q.id, vec![results]).cache_time(0).await?;
        return Ok(());
    };

    let mut langs: Vec<&str> = get_langs();

    langs.reverse();

    let current_flag = user.lang.as_deref().and_then(Flags::from_code);
    let mut results = Vec::new();

    let start_article = current_flag.map_or_else(
        || lang_empty_info(ltag),
        |f| lang_info(ltag, q.from.id, f.to_emoji(), f.to_code()),
    );

    results.push(InlineQueryResult::Article(start_article));

    for (idx, new_flag) in langs.iter().enumerate() {
        let info = lang_change_info(
            ltag,
            q.from.id,
            user.lang.as_deref(),
            new_flag,
            idx,
        );
        results.push(InlineQueryResult::Article(info));
    }

    bot.answer_inline_query(&q.id, results).cache_time(0).await?;

    Ok(())
}

async fn handle_error(
    bot: MyBot,
    q: InlineQuery,
    ltag: LocaleTag,
    err: MyError,
) {
    let error_message =
        vec![InlineQueryResult::Article(handle_error_info(ltag))];
    let _ = bot.answer_inline_query(q.id, error_message).cache_time(0).await;
    log::error!("Error in inline handler: {:?} by user [{}]", err, q.from.id);
    if let MyError::Database(diesel::result::Error::DatabaseError(
        _,
        err_info,
    )) = err
    {
        let message = err_info.message();
        log::error!("Error with database: {}", message);
    }
}

async fn handle_good(q: InlineQuery) {
    log::info!("Handled inline query [{}]: user: [{}]", q.id, q.from.id);
}

async fn _get_biggest_chat_pig_mass(id_user: UserId) -> MyResult<i32> {
    let biggest = DB.chat_pig.get_biggest_chat_pig(id_user.0).await?;
    let biggest_mass = biggest.map_or(0, |b| b.mass);

    Ok(biggest_mass)
}

async fn _get_for_top10_info(
    ltag: LocaleTag,
    chat_type: &str,
) -> MyResult<String> {
    let cur_date = get_date();
    let top10_chat_info = DB.hand_pig.get_top10_global(cur_date).await?;

    let text = top10_chat_info.map_or_else(
        || lng("HandPigNoInBarn", ltag),
        |v| crate::utils::text::generate_top10_text(ltag, v, chat_type),
    );

    Ok(text)
}

fn get_start_duel(
    ltag: LocaleTag,
    id_user: UserId,
    info: &InlineUser,
) -> InlineQueryResultArticle {
    let winrate = _get_duel_winrate(ltag, info.win, info.rout);

    let title = lng("DuelInlineCaption", ltag);
    let text = lng("InlineDuelStartMessage", ltag).args(&[
        ("name", &info.name),
        ("winrate", &winrate),
        ("weight", &info.weight.to_string()),
    ]);

    let content = InputMessageContentText::new(text).parse_mode(BOT_PARSE_MODE);

    InlineQueryResultArticle::new(
        "11",
        title,
        InputMessageContent::Text(content),
    )
    .description(lng("DuelInlineDesc", ltag))
    .thumb_url(get_photostock(Image::Fight))
    .reply_markup(keyboards::keyboard_start_duel(ltag, id_user))
}

fn get_top10_info(
    ltag: LocaleTag,
    id_user: UserId,
    text: String,
    chat_type: &str,
) -> InlineQueryResultArticle {
    let title = lng("Top10Caption", ltag);
    let message_text =
        InputMessageContentText::new(text).parse_mode(BOT_PARSE_MODE);

    InlineQueryResultArticle::new(
        "10",
        title,
        InputMessageContent::Text(message_text),
    )
    .description(lng("InlineTop10Desc", ltag))
    .thumb_url(get_photostock(Image::Top))
    .reply_markup(keyboards::keyboard_in_top10(ltag, id_user, chat_type))
}

fn get_hryak_info(
    ltag: LocaleTag,
    id_user: UserId,
    info: &(InlineUser, User),
    remove_markup: bool,
) -> InlineQueryResultArticle {
    let append = if info.1.supported {
        lng("InlineSupportedDeveloping", ltag)
    } else {
        String::new()
    };

    let converted_mass = info.0.weight.to_string();

    let caption = lng("InlineStatsCaption", ltag);
    let message = lng("HandPigWeightMessage", ltag).args(&[
        ("weight", converted_mass.as_str()),
        ("emoji", formulas::get_pig_emoji(info.0.weight)),
        ("append", append.as_str()),
    ]);

    let desc =
        lng("InlineStatsDesc", ltag).args(&[("weight", &converted_mass)]);
    let query_result = InlineQueryResultArticle::new(
        "0",
        caption,
        InputMessageContent::Text(
            InputMessageContentText::new(message).parse_mode(BOT_PARSE_MODE),
        ),
    )
    .description(desc)
    .thumb_url(get_photostock(Image::TakeWeight));

    if remove_markup {
        query_result
    } else {
        query_result
            .reply_markup(keyboards::keyboard_add_inline_top10(ltag, id_user))
    }
}

fn get_more_info(ltag: LocaleTag) -> InlineQueryResultArticle {
    let caption = lng("InlineMoreInfoCaption", ltag);
    let message = lng("InlineMoreInfoMessage", ltag);

    let desc = lng("InlineMoreInfoDesc", ltag);

    InlineQueryResultArticle::new(
        "600",
        caption,
        InputMessageContent::Text(
            InputMessageContentText::new(message).parse_mode(BOT_PARSE_MODE),
        ),
    )
    .description(desc)
    .thumb_url(get_photostock(Image::MoreInfo))
    .reply_markup(keyboards::keyboard_more_info(ltag))
}

fn name_hryak_info(ltag: LocaleTag, name: String) -> InlineQueryResultArticle {
    let caption = lng("HandPigNameGoCaption", ltag);
    let message = lng("HandPigNameGoMessage", ltag).args(&[("name", &name)]);

    InlineQueryResultArticle::new(
        "3",
        caption,
        InputMessageContent::Text(
            InputMessageContentText::new(message).parse_mode(BOT_PARSE_MODE),
        ),
    )
    .description(lng("HandPigNameGoDesc", ltag))
    .thumb_url(get_photostock(Image::NameTyping))
}

fn rename_hryak_info(
    ltag: LocaleTag,
    id_user: UserId,
    old_name: String,
    new_name: &str,
) -> InlineQueryResultArticle {
    let (cutted_name, _) = truncate(new_name, 20);
    let cutted_name = cutted_name.to_string();

    let message = lng("HandPigNameChangeMessage", ltag)
        .args(&[("past_name", &old_name), ("future_name", &cutted_name)]);

    let desc = lng("HandPigNameChangeDesc", ltag);
    InlineQueryResultArticle::new(
        "4",
        &cutted_name,
        InputMessageContent::Text(
            InputMessageContentText::new(message).parse_mode(BOT_PARSE_MODE),
        ),
    )
    .description(desc)
    .reply_markup(keyboards::keyboard_new_name(ltag, id_user, cutted_name))
    .thumb_url(get_photostock(Image::NameSuccess))
}

fn day_pig_info(
    ltag: LocaleTag,
    id_user: UserId,
    chat_type: Option<ChatType>,
) -> InlineQueryResultArticle {
    let caption = lng("InlineDayPigCaption", ltag);
    let message = lng("InlineDayPigMessage", ltag);
    let desc = lng("InlineDayPigDesc", ltag);

    let is_public_chat = chat_type
        .is_some_and(|v| !matches!(v, ChatType::Private | ChatType::Channel));

    let markup = if is_public_chat {
        keyboards::keyboard_day_pig(ltag, id_user)
    } else {
        keyboards::keyboard_day_pig_to_inline(ltag)
    };
    InlineQueryResultArticle::new(
        "5",
        caption,
        InputMessageContent::Text(
            InputMessageContentText::new(message).parse_mode(BOT_PARSE_MODE),
        ),
    )
    .description(desc)
    .thumb_url(get_photostock(Image::DayPig))
    .reply_markup(markup)
}

fn flag_info(ltag: LocaleTag, flag: &str) -> InlineQueryResultArticle {
    let caption = lng("HandPigFlagGoCaption", ltag).args(&[("flag", flag)]);
    let desc = lng("HandPigFlagGoDesc", ltag);
    let message = lng("HandPigFlagGoMessage", ltag).args(&[("flag", flag)]);

    InlineQueryResultArticle::new(
        "40004",
        caption,
        InputMessageContent::Text(
            InputMessageContentText::new(message).parse_mode(BOT_PARSE_MODE),
        ),
    )
    .description(desc)
}

fn flag_empty_info(ltag: LocaleTag) -> InlineQueryResultArticle {
    let caption = lng("HandPigNoFlagChangeCaption", ltag);
    let desc = lng("HandPigNoFlagChangeDesc", ltag);
    let message = lng("HandPigNoFlagChangeMessage", ltag);

    InlineQueryResultArticle::new(
        "40005",
        caption,
        InputMessageContent::Text(
            InputMessageContentText::new(message).parse_mode(BOT_PARSE_MODE),
        ),
    )
    .description(desc)
}

fn flag_change_info(
    ltag: LocaleTag,
    id_user: UserId,
    old_flag: Flags,
    new_flag: Flags,
    idx: usize,
) -> InlineQueryResultArticle {
    let old_flag_emoji = old_flag.to_emoji();
    let new_flag_emoji = new_flag.to_emoji();
    let new_flag_code = new_flag.to_code();

    let caption =
        lng("HandPigFlagChangeCaption", ltag).args(&[("flag", new_flag_emoji)]);
    let desc =
        lng("HandPigFlagChangeDesc", ltag).args(&[("code", new_flag_code)]);

    let message = lng("HandPigFlagChangeMessage", ltag)
        .args(&[("old_flag", old_flag_emoji), ("new_flag", new_flag_emoji)]);

    let markup = keyboards::keyboard_change_flag(ltag, id_user, new_flag_code);

    InlineQueryResultArticle::new(
        idx.to_string(),
        caption,
        InputMessageContent::Text(
            InputMessageContentText::new(message).parse_mode(BOT_PARSE_MODE),
        ),
    )
    .description(desc)
    .reply_markup(markup)
}

fn lang_info(
    ltag: LocaleTag,
    id_user: UserId,
    flag: &str,
    code: &str,
) -> InlineQueryResultArticle {
    let caption = lng("InlineLangGoCaption", ltag)
        .args(&[("flag", flag), ("code", code)]);
    let desc = lng("InlineLangGoDesc", ltag);
    let message = lng("InlineLangGoMessage", ltag)
        .args(&[("flag", flag), ("code", code)]);

    let markup = keyboards::keyboard_change_lang(ltag, id_user, "-");
    InlineQueryResultArticle::new(
        "50004",
        caption,
        InputMessageContent::Text(
            InputMessageContentText::new(message).parse_mode(BOT_PARSE_MODE),
        ),
    )
    .description(desc)
    .reply_markup(markup)
}

fn lang_empty_info(ltag: LocaleTag) -> InlineQueryResultArticle {
    let caption = lng("InlineLangNoChangeCaption", ltag);
    let desc = lng("InlineLangNoChangeDesc", ltag);
    let message = lng("InlineLangNoChangeMessage", ltag);

    InlineQueryResultArticle::new(
        "50005",
        caption,
        InputMessageContent::Text(
            InputMessageContentText::new(message).parse_mode(BOT_PARSE_MODE),
        ),
    )
    .description(desc)
}

fn lang_change_info(
    ltag: LocaleTag,
    id_user: UserId,
    old_lang_code: Option<&str>,
    new_lang_code: &str,
    idx: usize,
) -> InlineQueryResultArticle {
    let old_lang_emoji = old_lang_code
        .map_or("-", |v| Flags::from_code(v).unwrap_or(Flags::Us).to_emoji());
    let new_lang_emoji =
        Flags::from_code(new_lang_code).unwrap_or(Flags::Us).to_emoji();

    let langed_key = format!("lang_{new_lang_code}");

    let langed_name = lng(&langed_key, ltag);
    let caption = lng("InlineLangChangeCaption", ltag)
        .args(&[("flag", new_lang_emoji), ("lang", &langed_name)]);
    let desc_key = if old_lang_emoji == new_lang_emoji {
        "InlineLangChangeDescAlready"
    } else {
        "InlineLangChangeDesc"
    };
    let desc = lng(desc_key, ltag).args(&[("code", new_lang_code)]);

    let message = lng("InlineLangChangeMessage", ltag).args(&[
        ("old_code", old_lang_code.unwrap_or("-")),
        ("old_flag", old_lang_emoji),
        ("new_code", new_lang_code),
        ("new_flag", new_lang_emoji),
    ]);

    let markup = keyboards::keyboard_change_lang(ltag, id_user, new_lang_code);

    InlineQueryResultArticle::new(
        idx.to_string(),
        caption,
        InputMessageContent::Text(
            InputMessageContentText::new(message).parse_mode(BOT_PARSE_MODE),
        ),
    )
    .description(desc)
    .reply_markup(markup)
}

fn cpu_oc_info(ltag: LocaleTag, mass: f32) -> InlineQueryResultArticle {
    let caption = lng("InlineOcCPUCaption", ltag);

    let message = lng("InlineOcCPUMessage", ltag).args(&[
        ("cpu_clock", mass.to_string().as_str()),
        ("cpu_emoji", formulas::get_oc_cpu_emoji(mass)),
    ]);

    InlineQueryResultArticle::new(
        "6",
        caption,
        InputMessageContent::Text(
            InputMessageContentText::new(message).parse_mode(BOT_PARSE_MODE),
        ),
    )
    .thumb_url(get_photostock(Image::OCCPU))
}

fn ram_oc_info(ltag: LocaleTag, mass: u32) -> InlineQueryResultArticle {
    let caption = lng("InlineOcRAMCaption", ltag);
    let message = lng("InlineOcRAMMessage", ltag).args(&[
        ("ram_clock", mass.to_string().as_str()),
        ("ram_emoji", formulas::get_oc_ram_emoji(mass)),
    ]);

    InlineQueryResultArticle::new(
        "7",
        caption,
        InputMessageContent::Text(
            InputMessageContentText::new(message).parse_mode(BOT_PARSE_MODE),
        ),
    )
    .thumb_url(get_photostock(Image::OCRAM))
}

fn gpu_oc_info(ltag: LocaleTag, mass: f32) -> InlineQueryResultArticle {
    let caption = lng("InlineOcGPUCaption", ltag);
    let message = lng("InlineOcGPUMessage", ltag).args(&[
        ("gpu_hashrate", mass.to_string().as_str()),
        ("gpu_emoji", formulas::get_oc_gpu_emoji(mass)),
    ]);

    InlineQueryResultArticle::new(
        "8",
        caption,
        InputMessageContent::Text(
            InputMessageContentText::new(message).parse_mode(BOT_PARSE_MODE),
        ),
    )
    .thumb_url(get_photostock(Image::OCGPU))
}

fn get_accesibility_by_chattype<'a>(
    chat_type: Option<ChatType>,
) -> (&'a str, bool, &'a str) {
    //"chat_type, remove_markup, to"
    match chat_type {
        Some(ChatType::Private | ChatType::Channel) | None => {
            ("p_global", true, "p_win")
        },
        Some(_) => ("global", false, "chat"),
    }
}

fn handle_error_info(ltag: LocaleTag) -> InlineQueryResultArticle {
    let caption = lng("Error", ltag);
    let message = lng("InlineTechDesc", ltag);
    let desc = lng("InlineTechCaption", ltag);

    InlineQueryResultArticle::new(
        "500",
        caption,
        InputMessageContent::Text(
            InputMessageContentText::new(message).parse_mode(BOT_PARSE_MODE),
        ),
    )
    .description(desc)
    .thumb_url(get_photostock(Image::Error))
}

fn handle_error_parse(ltag: LocaleTag) -> InlineQueryResultArticle {
    let caption = lng("ErrorParseInlineNumberCaption", ltag);
    let desc = lng("ErrorParseInlineNumberDesc", ltag);

    let message = format!("{}\n\n{}", &caption, &desc);

    InlineQueryResultArticle::new(
        "5001",
        caption,
        InputMessageContent::Text(
            InputMessageContentText::new(message).parse_mode(BOT_PARSE_MODE),
        ),
    )
    .description(desc)
}

fn handle_no_results(ltag: LocaleTag) -> InlineQueryResultArticle {
    let caption = lng("ErrorNoResultsCaption", ltag);
    let desc = lng("ErrorNoResultsDesc", ltag);

    let message = format!("{}\n\n{}", &caption, &desc);

    InlineQueryResultArticle::new(
        "5002",
        caption,
        InputMessageContent::Text(
            InputMessageContentText::new(message).parse_mode(BOT_PARSE_MODE),
        ),
    )
    .description(desc)
}

fn _get_duel_winrate(ltag: LocaleTag, win: u16, rout: u16) -> String {
    if rout == 0 || win == 0 {
        return italic(&lng("InlineDuelNotEnoughBattles", ltag));
    }

    let percent_winrate = 100.0 / ((win as f32 + rout as f32) / win as f32);

    bold(&format!("{} %", percent_winrate as u32))
}
