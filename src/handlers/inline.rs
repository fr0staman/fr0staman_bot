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

use crate::db::DB;
use crate::enums::{Commands, Image, Keywords};
use crate::keyboards;
use crate::lang::{get_tag, lng, tag, InnerLang, LocaleTag};
use crate::models::{InlineUser, InlineVoice, NewInlineUser};
use crate::types::MyBot;
use crate::utils::date::get_date;
use crate::utils::helpers::{get_photostock, truncate};
use crate::utils::{formulas, helpers};
use crate::{Error, MyResult, BOT_PARSE_MODE, DEFAULT_LANG_TAG};

use super::callback::generate_top10_text;

pub async fn filter_inline_commands(
    bot: MyBot,
    q: InlineQuery,
) -> MyResult<()> {
    let ltag = tag(get_tag(&q.from));

    let temp_q = q.clone();
    let temp_bot = bot.clone();
    let text = q.query.clone();

    let split_command = text.split_once(' ');

    let function = match split_command {
        Some((action, payload)) => match Commands::from_str(action) {
            Ok(cmd) => match cmd {
                Commands::Name => {
                    inline_rename_hrundel(bot, q, ltag, payload).boxed()
                },
                Commands::Hru => inline_hruks(bot, q, ltag, payload).boxed(),
            },
            Err(_) => inline_hrundel(bot, q, ltag).boxed(),
        },
        None => match Keywords::from_str(&q.query) {
            Ok(kwd) => match kwd {
                Keywords::Name => inline_name_hrundel(bot, q, ltag).boxed(),
                Keywords::DayPig => inline_day_pig(bot, q, ltag).boxed(),
                Keywords::OC => inline_oc_stats(bot, q, ltag).boxed(),
                Keywords::Hru => inline_hruks(bot, q, ltag, "").boxed(),
            },
            Err(_) => inline_hrundel(bot, q, ltag).boxed(),
        },
    };

    let response = function.await;

    if let Err(err) = response {
        handle_error(temp_bot, temp_q, ltag, err).await;
    } else {
        handle_good(temp_q).await;
    }

    Ok(())
}

async fn inline_hrundel(
    bot: MyBot,
    q: InlineQuery,
    ltag: LocaleTag,
) -> MyResult<()> {
    let show_query = _get_hryak(&q, ltag).await?;

    let results = show_query
        .into_iter()
        .map(InlineQueryResult::Article)
        .collect::<Vec<_>>();

    bot.answer_inline_query(q.id, results).cache_time(0).await?;
    Ok(())
}

#[async_recursion]
async fn _get_hryak(
    q: &InlineQuery,
    ltag: LocaleTag,
) -> MyResult<Vec<InlineQueryResultArticle>> {
    let hrundel_info = DB.hand_pig.get_hrundel(q.from.id.0).await?;
    let cur_date = get_date();

    if hrundel_info.is_none() {
        let biggest_mass = _get_biggest_chat_pig_mass(q.from.id).await?;

        let size = formulas::calculate_hryak_size(q.from.id.0) + biggest_mass;
        let truncated_f_name = truncate(&q.from.first_name, 64);
        let hrundel = NewInlineUser {
            user_id: q.from.id.0,
            f_name: truncated_f_name.0,
            weight: size,
            date: cur_date,
            lang: q.from.language_code.as_deref().unwrap_or(DEFAULT_LANG_TAG),
            name: truncated_f_name.0,
        };
        DB.hand_pig.add_hrundel(hrundel).await?;
        return _get_hryak(q, ltag).await;
    }

    let info = hrundel_info.unwrap();

    if info.date != cur_date {
        // Pig exist, but not "today", just recreate that!
        let biggest_mass = _get_biggest_chat_pig_mass(q.from.id).await?;
        let add = biggest_mass + helpers::mass_addition_on_status(info.status);
        DB.hand_pig.update_hrundel(&q.from, info, add).await?;
        return _get_hryak(q, ltag).await;
    }

    let (chat_type, remove_markup, to) =
        get_accesibility_by_chattype(q.chat_type);
    let text = _get_for_top10_info(ltag, chat_type).await?;

    let result = vec![
        get_start_duel(ltag, q.from.id, &info),
        get_top10_info(ltag, q.from.id, text, to),
        get_hryak_info(ltag, q.from.id, &info, remove_markup),
    ];

    Ok(result)
}

async fn inline_name_hrundel(
    bot: MyBot,
    q: InlineQuery,
    ltag: LocaleTag,
) -> MyResult<()> {
    let hrundel_info = DB.hand_pig.get_hrundel(q.from.id.0).await?.unwrap();

    let results = vec![InlineQueryResult::Article(name_hryak_info(
        ltag,
        hrundel_info.name,
    ))];

    bot.answer_inline_query(&q.id, results).cache_time(0).await?;
    Ok(())
}

async fn inline_rename_hrundel(
    bot: MyBot,
    q: InlineQuery,
    ltag: LocaleTag,
    new_name: &str,
) -> MyResult<()> {
    let old_name = DB.hand_pig.get_hrundel(q.from.id.0).await?.unwrap().name;
    let results = vec![InlineQueryResult::Article(rename_hryak_info(
        ltag, q.from.id, old_name, new_name,
    ))];

    bot.answer_inline_query(&q.id, results).cache_time(0).await?;
    Ok(())
}

async fn inline_day_pig(
    bot: MyBot,
    q: InlineQuery,
    ltag: LocaleTag,
) -> MyResult<()> {
    let results =
        vec![InlineQueryResult::Article(day_pig_info(ltag, q.from.id))];

    bot.answer_inline_query(&q.id, results).cache_time(0).await?;
    Ok(())
}

async fn inline_oc_stats(
    bot: MyBot,
    q: InlineQuery,
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
    q: InlineQuery,
    ltag: LocaleTag,
    #[allow(unused)] payload: &str,
) -> MyResult<()> {
    let voices: Vec<InlineVoice> = if payload.is_empty() {
        DB.other.get_50_inline_voices().await?
    } else {
        let Ok(id) = payload.parse::<i16>() else {
            bot.answer_inline_query(
                q.id,
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
        bot.answer_inline_query(q.id, vec![result]).await?;
        return Ok(());
    }

    let url = "https://t.me".parse::<url::Url>().unwrap();

    let results: Vec<InlineQueryResult> = voices
        .iter()
        .map(|item| {
            let caption = lng("InlineHrukCaptionNumber", ltag)
                .args(&[("number", &item.id.to_string())]);

            InlineQueryResult::Voice(InlineQueryResultVoice::new(
                item.id.to_string(),
                url.join(&item.url).unwrap_or(url.clone()),
                caption,
            ))
        })
        .collect();

    bot.answer_inline_query(&q.id, results).cache_time(30).await?;
    Ok(())
}

async fn handle_error(bot: MyBot, q: InlineQuery, ltag: LocaleTag, err: Error) {
    let error_message =
        vec![InlineQueryResult::Article(handle_error_info(ltag))];
    let _ = bot.answer_inline_query(q.id, error_message).cache_time(0).await;
    log::error!("Error in inline handler: {:?} by user [{}]", err, q.from.id);
    if let Error::Database(diesel::result::Error::DatabaseError(_, err_info)) =
        err
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
        |v| generate_top10_text(ltag, v, chat_type),
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
    info: &InlineUser,
    remove_markup: bool,
) -> InlineQueryResultArticle {
    let append = if info.status == 2 {
        lng("InlineSupportedDeveloping", ltag)
    } else {
        String::new()
    };

    let converted_mass = info.weight.to_string();

    let caption = lng("InlineStatsCaption", ltag);
    let message = lng("HandPigWeightMessage", ltag).args(&[
        ("weight", converted_mass.as_str()),
        ("emoji", formulas::get_pig_emoji(info.weight)),
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

fn day_pig_info(ltag: LocaleTag, id_user: UserId) -> InlineQueryResultArticle {
    let caption = lng("InlineDayPigCaption", ltag);
    let message = lng("InlineDayPigMessage", ltag);
    let desc = lng("InlineDayPigDesc", ltag);

    InlineQueryResultArticle::new(
        "5",
        caption,
        InputMessageContent::Text(
            InputMessageContentText::new(message).parse_mode(BOT_PARSE_MODE),
        ),
    )
    .description(desc)
    .thumb_url(get_photostock(Image::DayPig))
    .reply_markup(keyboards::keyboard_day_pig(ltag, id_user))
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
