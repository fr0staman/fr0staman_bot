use std::str::FromStr;

use futures::FutureExt;

use teloxide::payloads::AnswerInlineQuerySetters;
use teloxide::{
    requests::Requester,
    types::{
        ChatType, InlineQuery, InlineQueryResult, InlineQueryResultArticle,
        UserId,
    },
};

use crate::consts::{DEFAULT_LANG_TAG, INLINE_QUERY_LIMIT};
use crate::db::DB;
use crate::enums::{InlineCommands, InlineKeywords, Top10Variant};
use crate::lang::{get_langs, get_tag, lng, tag_one_or, InnerLang, LocaleTag};
use crate::models::{InlineGif, InlineVoice, NewInlineUser, UpdateInlineUser};
use crate::types::MyBot;
use crate::utils::date::{get_date, get_datetime};
use crate::utils::flag::Flags;
use crate::utils::helpers::{escape, truncate};
use crate::utils::{formulas, helpers, iq_results};
use crate::{MyError, MyResult};

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
                InlineCommands::Gif => {
                    inline_gif(bot, &q, ltag, payload).boxed()
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
                InlineKeywords::Gif => inline_gif(bot, &q, ltag, "").boxed(),
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

async fn _get_hryak(
    q: &InlineQuery,
    ltag: LocaleTag,
) -> MyResult<Vec<InlineQueryResultArticle>> {
    let hrundel_info = DB.hand_pig.get_hrundel(q.from.id.0).await?;
    let cur_date = get_date();

    let Some(info) = hrundel_info else {
        let Some(user) = DB
            .other
            .maybe_get_or_insert_user(q.from.id.0, get_datetime)
            .await?
        else {
            return Ok(vec![iq_results::handle_error_info(ltag)]);
        };

        let biggest_mass = _get_biggest_chat_pig_mass(q.from.id).await?;

        let weight = formulas::calculate_hryak_size(q.from.id.0) + biggest_mass;
        let escaped_f_name = escape(&q.from.first_name);
        let f_name = truncate(&escaped_f_name, 64).0;

        let hrundel = NewInlineUser {
            uid: user.id,
            weight,
            f_name,
            name: f_name,
            date: cur_date,
            flag: q.from.language_code.as_deref().unwrap_or(DEFAULT_LANG_TAG),
        };
        DB.hand_pig.add_hrundel(hrundel).await?;
        return Box::pin(_get_hryak(q, ltag)).await;
    };

    if info.0.date != cur_date {
        // Pig exist, but not "today", just recreate that!
        let weight = formulas::calculate_hryak_size(q.from.id.0);
        let biggest_mass = _get_biggest_chat_pig_mass(q.from.id).await?;
        let add = biggest_mass + helpers::mass_addition_on_status(&info.1);

        let escaped_f_name = escape(&q.from.first_name);
        let f_name = truncate(&escaped_f_name, 64).0;

        let update_data = UpdateInlineUser {
            id: info.0.id,
            f_name,
            weight: weight + add,
            date: cur_date,
            gifted: false,
        };

        DB.hand_pig.update_hrundel(update_data).await?;
        return Box::pin(_get_hryak(q, ltag)).await;
    }

    let (chat_type, remove_markup, to) =
        get_accesibility_by_chattype(q.chat_type);
    let text = _get_for_top10_info(ltag, chat_type).await?;

    let result = vec![
        iq_results::get_start_duel(ltag, q.from.id, &info.0),
        iq_results::get_top10_info(ltag, q.from.id, text, to),
        iq_results::get_hryak_info(ltag, q.from.id, &info, remove_markup),
        iq_results::get_more_info(ltag),
    ];

    Ok(result)
}

async fn inline_name_hrundel(
    bot: MyBot,
    q: &InlineQuery,
    ltag: LocaleTag,
) -> MyResult<()> {
    let Some(hrundel) = DB.hand_pig.get_hrundel(q.from.id.0).await? else {
        let results =
            InlineQueryResult::Article(iq_results::handle_no_results(ltag));
        bot.answer_inline_query(&q.id, vec![results]).cache_time(0).await?;
        return Ok(());
    };

    let article = iq_results::name_hryak_info(ltag, hrundel.0.name);
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
        let results =
            InlineQueryResult::Article(iq_results::handle_no_results(ltag));
        bot.answer_inline_query(&q.id, vec![results]).cache_time(0).await?;
        return Ok(());
    };

    let new_name = escape(new_name);

    let new_name =
        if new_name.is_empty() { lng("UnnamedPig", ltag) } else { new_name };

    let article = iq_results::rename_hryak_info(
        ltag,
        q.from.id,
        hrundel.0.name,
        &new_name,
    );
    let results = vec![InlineQueryResult::Article(article)];

    bot.answer_inline_query(&q.id, results).cache_time(0).await?;
    Ok(())
}

async fn inline_day_pig(
    bot: MyBot,
    q: &InlineQuery,
    ltag: LocaleTag,
) -> MyResult<()> {
    let article = iq_results::day_pig_info(ltag, q.from.id, q.chat_type);
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
        InlineQueryResult::Article(iq_results::cpu_oc_info(ltag, cpu_clock)),
        InlineQueryResult::Article(iq_results::ram_oc_info(ltag, ram_clock)),
        InlineQueryResult::Article(iq_results::gpu_oc_info(ltag, gpu_hashr)),
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
                vec![InlineQueryResult::Article(
                    iq_results::handle_error_parse(ltag),
                )],
            )
            .await?;
            return Ok(());
        };

        let voice = DB.other.get_inline_voice_by_id(id).await?;
        voice.into_iter().collect()
    };

    if voices.is_empty() {
        let result =
            InlineQueryResult::Article(iq_results::handle_no_results(ltag));
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
    let results: Vec<_> = paged_voices
        .iter()
        .map(|item| {
            let caption = lng("InlineHrukCaptionNumber", ltag)
                .args(&[("number", &item.id.to_string())]);
            let voice_url = url.join(&item.url).unwrap_or_else(|_| url.clone());

            InlineQueryResult::Voice(iq_results::hru_voice_info(
                item.id, voice_url, caption,
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
        let results =
            InlineQueryResult::Article(iq_results::handle_no_results(ltag));
        bot.answer_inline_query(&q.id, vec![results]).cache_time(0).await?;
        return Ok(());
    };

    let old_flag = Flags::from_code(&user.0.flag).unwrap_or(Flags::Us);
    let mut results = Vec::with_capacity(64);

    if q.offset.is_empty() {
        let start_info = InlineQueryResult::Article(iq_results::flag_info(
            ltag,
            old_flag.to_emoji(),
        ));

        results.push(start_info);
    }

    let number_from_offset = q.offset.parse::<usize>().unwrap_or(0);

    let searched_flags: Vec<_> = if payload.is_empty() {
        Flags::FLAGS.to_vec()
    } else {
        Flags::FLAGS
            .into_iter()
            .filter(|i| {
                i.to_code().contains(payload) || i.to_emoji().contains(payload)
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
        let empty_info = iq_results::flag_empty_info(ltag);
        results.push(InlineQueryResult::Article(empty_info));
    } else {
        for new_flag in selected_flags {
            let info = iq_results::flag_change_info(
                ltag, q.from.id, old_flag, *new_flag,
            );
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
        let results =
            InlineQueryResult::Article(iq_results::handle_no_results(ltag));
        bot.answer_inline_query(&q.id, vec![results]).cache_time(0).await?;
        return Ok(());
    };

    let mut langs: Vec<&str> = get_langs();

    langs.reverse();

    let current_flag = user.lang.as_deref().and_then(Flags::from_code);
    let mut results = Vec::new();

    let start_article = current_flag.map_or_else(
        || iq_results::lang_empty_info(ltag),
        |f| iq_results::lang_info(ltag, q.from.id, f.to_emoji(), f.to_code()),
    );

    results.push(InlineQueryResult::Article(start_article));

    for new_flag in langs {
        let info = iq_results::lang_change_info(
            ltag,
            q.from.id,
            user.lang.as_deref(),
            new_flag,
        );
        results.push(InlineQueryResult::Article(info));
    }

    bot.answer_inline_query(&q.id, results).cache_time(0).await?;

    Ok(())
}

async fn inline_gif(
    bot: MyBot,
    q: &InlineQuery,
    ltag: LocaleTag,
    payload: &str,
) -> MyResult<()> {
    let gifs: Vec<InlineGif> = if payload.is_empty() {
        DB.other.get_inline_gifs().await?
    } else {
        let Ok(id) = payload.parse::<i16>() else {
            bot.answer_inline_query(
                &q.id,
                vec![InlineQueryResult::Article(
                    iq_results::handle_error_parse(ltag),
                )],
            )
            .await?;
            return Ok(());
        };

        let gif = DB.other.get_inline_gif_by_id(id).await?;
        gif.into_iter().collect()
    };

    if gifs.is_empty() {
        let result =
            InlineQueryResult::Article(iq_results::handle_no_results(ltag));
        bot.answer_inline_query(&q.id, vec![result]).await?;
        return Ok(());
    }

    let number_from_offset = q.offset.parse::<usize>().unwrap_or(0);

    let (start_index, end_index) = {
        const ON_PAGE: usize = INLINE_QUERY_LIMIT;
        let start_index = ON_PAGE * number_from_offset;
        let probably_end_index = start_index + ON_PAGE;

        (start_index, probably_end_index.min(gifs.len()))
    };

    let paged_gifs = &gifs[start_index..end_index];
    let results: Vec<_> = paged_gifs
        .iter()
        .map(|item| {
            InlineQueryResult::CachedGif(iq_results::gif_pig_info(
                item.id,
                item.file_id.clone(),
            ))
        })
        .collect();

    let query = bot.answer_inline_query(&q.id, results).cache_time(30);

    if end_index != gifs.len() {
        let next_offset = (number_from_offset + 1).to_string();
        query.next_offset(next_offset).await?;
    } else {
        query.await?;
    };
    Ok(())
}

async fn handle_error(
    bot: MyBot,
    q: InlineQuery,
    ltag: LocaleTag,
    err: MyError,
) {
    let error_message =
        vec![InlineQueryResult::Article(iq_results::handle_error_info(ltag))];
    let _ = bot.answer_inline_query(q.id, error_message).cache_time(0).await;
    crate::myerr!("Error in inline handler: {:?} by user [{}]", err, q.from.id);
    if let MyError::Database(diesel::result::Error::DatabaseError(
        _,
        err_info,
    )) = err
    {
        let message = err_info.message();
        crate::myerr!("Error with database: {}", message);
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
    chat_type: Top10Variant,
) -> MyResult<String> {
    let cur_date = get_date();
    let top10_chat_info = DB.hand_pig.get_top10_global(cur_date).await?;

    let text = top10_chat_info.map_or_else(
        || lng("HandPigNoInBarn", ltag),
        |v| crate::utils::text::generate_top10_text(ltag, v, chat_type),
    );

    Ok(text)
}

fn get_accesibility_by_chattype(
    chat_type: Option<ChatType>,
) -> (Top10Variant, bool, Top10Variant) {
    //"chat_type, remove_markup, to"

    match chat_type {
        Some(ChatType::Private | ChatType::Channel) | None => {
            (Top10Variant::PGlobal, true, Top10Variant::PWin)
        },
        Some(_) => (Top10Variant::Global, false, Top10Variant::Chat),
    }
}
