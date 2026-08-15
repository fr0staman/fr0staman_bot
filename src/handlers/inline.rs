use std::borrow::Cow;
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

use crate::config::consts::{DEFAULT_LANG_TAG, INLINE_QUERY_LIMIT};
use crate::db::DB;
use crate::db::models::{
    InlineGif, InlineVoice, NewInlineUser, UpdateInlineUser,
};
use crate::db::shortcuts;
use crate::enums::{InlineCommands, InlineKeywords, Top10Variant};
use crate::lang::{InnerLang, LocaleTag, get_langs, get_tag, lng, tag_one_or};
use crate::types::MyBot;
use crate::types::{MyError, MyResult};
use crate::utils::date::get_date;
use crate::utils::flag::Flags;
use crate::utils::helpers::{escape, truncate};
use crate::utils::{formulas, helpers, iq_results};

#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub struct Page {
    pub start: usize,
    pub end: usize,
    /// `None` on the last page.
    pub next_offset: Option<String>,
}

/// An unparseable offset means the first page. `start` is clamped so an
/// out-of-range offset yields an empty page instead of panicking.
pub fn page_slice(total: usize, offset: &str, per_page: usize) -> Page {
    let page_number = offset.parse::<usize>().unwrap_or(0);

    let start = per_page.saturating_mul(page_number).min(total);
    let end = start.saturating_add(per_page).min(total);
    let next_offset = (end != total).then(|| (page_number + 1).to_string());

    Page { start, end, next_offset }
}

pub async fn filter_inline_commands(
    bot: MyBot,
    q: InlineQuery,
) -> MyResult<()> {
    crate::metrics::INLINE_COUNTER.inc();
    let user = shortcuts::maybe_get_or_insert_user(&q.from, false).await?;
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

    bot.answer_inline_query(q.id.clone(), results).cache_time(0).await?;
    Ok(())
}

async fn _get_hryak(
    q: &InlineQuery,
    ltag: LocaleTag,
) -> MyResult<Vec<InlineQueryResultArticle>> {
    let hrundel_info = DB.hand_pig.get_hrundel(q.from.id.0 as i64).await?;
    let cur_date = get_date();

    let Some(info) = hrundel_info else {
        let Some(user) =
            shortcuts::maybe_get_or_insert_user(&q.from, false).await?
        else {
            return Ok(vec![iq_results::handle_error_info(ltag)]);
        };

        let biggest_mass = _get_biggest_chat_pig_mass(q.from.id).await?;

        let weight =
            formulas::calculate_hryak_size(q.from.id.0 as i64) + biggest_mass;
        let escaped_f_name = escape(&q.from.first_name);
        let f_name = truncate(&escaped_f_name, 64).0;

        let hrundel = NewInlineUser {
            uid: user.id,
            weight,
            name: f_name,
            date: cur_date,
            flag: q.from.language_code.as_deref().unwrap_or(DEFAULT_LANG_TAG),
            gifted: false,
            rout: 0,
            win: 0,
        };
        DB.hand_pig.add_hrundel(hrundel).await?;
        return Box::pin(_get_hryak(q, ltag)).await;
    };

    if info.0.date != cur_date {
        // Pig exist, but not "today", just recreate that!
        let weight = formulas::calculate_hryak_size(q.from.id.0 as i64);
        let biggest_mass = _get_biggest_chat_pig_mass(q.from.id).await?;
        let add = biggest_mass + helpers::mass_addition_on_status(&info.1);

        let update_data = UpdateInlineUser {
            id: info.0.id,
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
    let Some(hrundel) = DB.hand_pig.get_hrundel(q.from.id.0 as i64).await?
    else {
        let results =
            InlineQueryResult::Article(iq_results::handle_no_results(ltag));
        bot.answer_inline_query(q.id.clone(), vec![results])
            .cache_time(0)
            .await?;
        return Ok(());
    };

    let article = iq_results::name_hryak_info(ltag, hrundel.0.name);
    let results = vec![InlineQueryResult::Article(article)];

    bot.answer_inline_query(q.id.clone(), results).cache_time(0).await?;
    Ok(())
}

async fn inline_rename_hrundel(
    bot: MyBot,
    q: &InlineQuery,
    ltag: LocaleTag,
    new_name: &str,
) -> MyResult<()> {
    let Some(hrundel) = DB.hand_pig.get_hrundel(q.from.id.0 as i64).await?
    else {
        let results =
            InlineQueryResult::Article(iq_results::handle_no_results(ltag));
        bot.answer_inline_query(q.id.clone(), vec![results])
            .cache_time(0)
            .await?;
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

    bot.answer_inline_query(q.id.clone(), results).cache_time(0).await?;
    Ok(())
}

async fn inline_day_pig(
    bot: MyBot,
    q: &InlineQuery,
    ltag: LocaleTag,
) -> MyResult<()> {
    let article = iq_results::day_pig_info(ltag, q.from.id, q.chat_type);
    let results = vec![InlineQueryResult::Article(article)];

    bot.answer_inline_query(q.id.clone(), results).cache_time(0).await?;
    Ok(())
}

async fn inline_oc_stats(
    bot: MyBot,
    q: &InlineQuery,
    ltag: LocaleTag,
) -> MyResult<()> {
    let user_id = q.from.id.0 as i64;
    let hryak_size = formulas::calculate_hryak_size(user_id);

    let cpu_clock = formulas::calculate_cpu_clock(hryak_size, user_id);
    let ram_clock = formulas::calculate_ram_clock(hryak_size, user_id);
    let gpu_hashr = formulas::calculate_gpu_hashrate(hryak_size, user_id);

    let results = vec![
        InlineQueryResult::Article(iq_results::cpu_oc_info(ltag, cpu_clock)),
        InlineQueryResult::Article(iq_results::ram_oc_info(ltag, ram_clock)),
        InlineQueryResult::Article(iq_results::gpu_oc_info(ltag, gpu_hashr)),
    ];

    bot.answer_inline_query(q.id.clone(), results).cache_time(0).await?;
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
                q.id.clone(),
                [InlineQueryResult::Article(iq_results::handle_error_parse(
                    ltag,
                ))],
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
        bot.answer_inline_query(q.id.clone(), vec![result]).await?;
        return Ok(());
    }

    let page = page_slice(voices.len(), &q.offset, INLINE_QUERY_LIMIT);

    let url = "https://t.me".parse::<url::Url>().unwrap();

    let paged_voices = &voices[page.start..page.end];
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

    let query = bot.answer_inline_query(q.id.clone(), results).cache_time(30);

    if let Some(next_offset) = page.next_offset {
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
    let Some(user) = DB.hand_pig.get_hrundel(q.from.id.0 as i64).await? else {
        let results =
            InlineQueryResult::Article(iq_results::handle_no_results(ltag));
        bot.answer_inline_query(q.id.clone(), vec![results])
            .cache_time(0)
            .await?;
        return Ok(());
    };

    let old_flag = Flags::from_code(&user.0.flag).unwrap_or(Flags::Us);
    let mut results = Vec::with_capacity(64);

    if q.offset.is_empty() {
        let start_info = iq_results::flag_info(ltag, old_flag.to_emoji());
        results.push(InlineQueryResult::Article(start_info));
    }

    let searched_flags: Cow<[_]> = if payload.is_empty() {
        Flags::FLAGS.into()
    } else {
        Flags::FLAGS
            .iter()
            .copied()
            .filter(|v| {
                v.to_code().contains(payload) || v.to_emoji().contains(payload)
            })
            .collect()
    };

    let count = searched_flags.len();

    // One slot is reserved for the "current flag" article above.
    const ON_PAGE: usize = INLINE_QUERY_LIMIT - 1;
    let page = page_slice(count, &q.offset, ON_PAGE);

    if count == 0 {
        let empty_info = iq_results::flag_empty_info(ltag);
        results.push(InlineQueryResult::Article(empty_info));
    } else {
        let selected_flags = &searched_flags[page.start..page.end];

        for new_flag in selected_flags {
            let info = iq_results::flag_change_info(
                ltag, q.from.id, old_flag, *new_flag,
            );
            results.push(InlineQueryResult::Article(info));
        }
    }

    let mut query =
        bot.answer_inline_query(q.id.clone(), results).cache_time(0);

    if let Some(next_offset) = page.next_offset {
        query = query.next_offset(next_offset);
    }

    query.await?;

    Ok(())
}

async fn inline_lang(
    bot: MyBot,
    q: &InlineQuery,
    ltag: LocaleTag,
) -> MyResult<()> {
    let Some(user) = DB.other.get_user(q.from.id.0 as i64).await? else {
        let results =
            InlineQueryResult::Article(iq_results::handle_no_results(ltag));
        bot.answer_inline_query(q.id.clone(), vec![results])
            .cache_time(0)
            .await?;
        return Ok(());
    };

    let mut langs = get_langs();

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
            &new_flag,
        );
        results.push(InlineQueryResult::Article(info));
    }

    bot.answer_inline_query(q.id.clone(), results).cache_time(0).await?;

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
                q.id.clone(),
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
        bot.answer_inline_query(q.id.clone(), vec![result]).await?;
        return Ok(());
    }

    let page = page_slice(gifs.len(), &q.offset, INLINE_QUERY_LIMIT);

    let paged_gifs = &gifs[page.start..page.end];
    let results: Vec<_> = paged_gifs
        .iter()
        .map(|item| {
            InlineQueryResult::CachedGif(iq_results::gif_pig_info(
                item.id,
                item.file_id.clone(),
            ))
        })
        .collect();

    let query = bot.answer_inline_query(q.id.clone(), results).cache_time(30);

    if let Some(next_offset) = page.next_offset {
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
    let biggest = DB.chat_pig.get_biggest_chat_pig(id_user.0 as i64).await?;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn page(total: usize, offset: &str) -> Page {
        page_slice(total, offset, 50)
    }

    #[test]
    fn an_empty_offset_is_the_first_page() {
        assert_eq!(
            page(120, ""),
            Page { start: 0, end: 50, next_offset: Some("1".to_owned()) }
        );
    }

    #[test]
    fn pages_walk_forward_and_the_last_one_has_no_next_offset() {
        assert_eq!(
            page(120, "1"),
            Page { start: 50, end: 100, next_offset: Some("2".to_owned()) }
        );
        assert_eq!(page(120, "2"), Page {
            start: 100,
            end: 120,
            next_offset: None
        });
    }

    #[test]
    fn an_exactly_full_page_is_the_last_one() {
        assert_eq!(page(50, ""), Page {
            start: 0,
            end: 50,
            next_offset: None
        });
    }

    #[test]
    fn a_short_list_fits_on_one_page() {
        assert_eq!(page(3, ""), Page { start: 0, end: 3, next_offset: None });
        assert_eq!(page(0, ""), Page { start: 0, end: 0, next_offset: None });
    }

    #[test]
    fn an_unparseable_offset_falls_back_to_the_first_page() {
        for offset in ["", "abc", "-1", "1.5", " 1"] {
            assert_eq!(page(120, offset).start, 0, "offset {offset:?}");
        }
    }

    #[test]
    fn an_out_of_range_offset_yields_an_empty_page_instead_of_panicking() {
        // The handlers slice with `&items[page.start..page.end]`, so an
        // unclamped `start` past the end would panic.
        let p = page(10, "99");
        assert_eq!(p, Page { start: 10, end: 10, next_offset: None });
        assert!(p.start <= p.end);
    }

    #[test]
    fn start_never_exceeds_end_for_any_offset() {
        for total in [0usize, 1, 49, 50, 51, 120] {
            for n in 0..8usize {
                let p = page(total, &n.to_string());
                assert!(p.start <= p.end, "total {total} offset {n}");
                assert!(p.end <= total, "total {total} offset {n}");
            }
        }
    }

    #[test]
    fn the_flag_picker_reserves_one_slot_per_page() {
        // `inline_flag` pages by INLINE_QUERY_LIMIT - 1 to leave room for the
        // "current flag" article.
        let per_page = INLINE_QUERY_LIMIT - 1;
        let p = page_slice(200, "", per_page);

        assert_eq!(p.end, per_page);
        assert_eq!(p.next_offset, Some("1".to_owned()));
    }

    #[test]
    fn every_item_is_visited_exactly_once_walking_the_pages() {
        let total = 137usize;
        let mut seen = 0usize;
        let mut offset = String::new();

        loop {
            let p = page(total, &offset);
            seen += p.end - p.start;
            match p.next_offset {
                Some(next) => offset = next,
                None => break,
            }
        }

        assert_eq!(seen, total);
    }
}
