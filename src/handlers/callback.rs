use futures::{future::BoxFuture, FutureExt};
use rand::Rng;
use std::str::FromStr;
use teloxide::{
    prelude::*,
    requests::Requester,
    types::{CallbackQuery, UserId},
    utils::html::user_mention,
};
use tokio::{
    sync::Mutex,
    time::{sleep, Duration},
};

use crate::{
    config::BOT_CONFIG,
    consts::{
        DAILY_GIFT_AMOUNT, DEFAULT_LANG_TAG, DUEL_LIST, DUEL_LOCKS,
        SUBSCRIBE_GIFT, TOP_LIMIT,
    },
    db::DB,
    enums::{CbActions, Top10Variant},
    keyboards,
    lang::{get_tag, lng, tag, tag_one_or, InnerLang, LocaleTag},
    models::{InlineUser, UpdateInlineUser, User, UserStatus},
    traits::MaybeMessageSetter,
    types::ParsedCallbackData,
    utils::{
        date::{get_date, get_datetime},
        flag::Flags,
        formulas,
        helpers::{self, get_hash},
        text::{generate_chat_top50_text, generate_top10_text},
    },
    MyBot, MyError, MyResult,
};

pub async fn filter_callback_commands(
    bot: MyBot,
    q: CallbackQuery,
) -> MyResult<()> {
    crate::metrics::CALLBACK_COUNTER.inc();
    let Some(user) =
        DB.other.maybe_get_or_insert_user(q.from.id.0, get_datetime).await?
    else {
        log::error!("User not exist after inserting!");
        return Ok(());
    };

    let ltag = tag_one_or(user.lang.as_deref(), get_tag(&q.from));
    let temp_bot = bot.clone();

    let decoded_data =
        q.data.as_deref().and_then(helpers::decode_callback_data);

    let function = match decoded_data {
        Some(payload) => _inner_filter(bot, &q, ltag, payload),
        None => callback_empty(bot, &q, ltag).boxed(),
    };

    let response = function.await;

    if let Err(err) = response {
        _handle_error(temp_bot, q, ltag, err).await?;
    } else {
        log::info!("Handled callback [{}]: user: [{}]", q.id, q.from.id);
    }

    Ok(())
}

fn _inner_filter<'a>(
    bot: MyBot,
    q: &'a CallbackQuery,
    ltag: LocaleTag,
    d: ParsedCallbackData<'a>,
) -> BoxFuture<'a, MyResult<()>> {
    let Ok(matched_enum) = CbActions::from_str(d.0) else {
        return callback_empty(bot, q, ltag).boxed();
    };

    match matched_enum {
        CbActions::GiveName => {
            callback_give_hand_pig_name(bot, q, ltag, d).boxed()
        },
        CbActions::FindHryak => callback_find_day_pig(bot, q, ltag, d).boxed(),
        CbActions::AddChat => callback_add_inline_chat(bot, q, ltag, d).boxed(),
        CbActions::Top10 => callback_top10(bot, q, ltag, d).boxed(),
        CbActions::StartDuel => callback_start_duel(bot, q, ltag, d).boxed(),
        CbActions::TopLeft | CbActions::TopRight => {
            callback_change_top(bot, q, ltag, d).boxed()
        },
        CbActions::AllowVoice => callback_allow_voice(bot, q, ltag, d).boxed(),
        CbActions::DisallowVoice => {
            callback_disallow_voice(bot, q, ltag, d).boxed()
        },
        CbActions::ChangeFlag => callback_change_flag(bot, q, ltag, d).boxed(),
        CbActions::SubCheck => {
            callback_check_subscribe(bot, q, ltag, d).boxed()
        },
        CbActions::SubGift => callback_gift(bot, q, ltag, d).boxed(),
        CbActions::ChangeLang => callback_change_lang(bot, q, ltag, d).boxed(),
    }
}

async fn _handle_error(
    bot: MyBot,
    q: CallbackQuery,
    ltag: LocaleTag,
    err: MyError,
) -> MyResult<()> {
    let Some(im_id) = q.inline_message_id.clone() else { return Ok(()) };

    log::error!("Error in callback: {:?}", err);

    let key = q.from.id.0;
    let thread_identifier = get_hash(&im_id);

    tokio::spawn(async move {
        {
            let locks = DUEL_LOCKS.try_get(&key).try_unwrap();
            if let Some(value) = locks {
                log::warn!(
                    "Cleaned errored duel [{}] for user [{}]",
                    thread_identifier,
                    key
                );

                value.lock().await.retain(|&x| x != thread_identifier);
                drop(value);
                DUEL_LIST.retain(|&x| x != thread_identifier);
            };
        };

        // Try to change message about error
        if let MyError::RequestError(err) = err {
            let temp_bot = bot.clone();

            tokio::spawn(async move {
                let text = if let teloxide::RequestError::RetryAfter(time) = err
                {
                    let _ = callback_error(&temp_bot, &q, ltag).await;

                    sleep(time).await;
                    lng("ErrorInlineTooMuchMessage", ltag)
                } else {
                    lng("ErrorInlineInvalidQueryMessage", ltag)
                };

                let _ = temp_bot.edit_message_text_inline(im_id, text).await;
            });
        } else {
            let _ = callback_empty(bot, &q, ltag).await;
        };
    });

    Ok(())
}
async fn callback_give_hand_pig_name(
    bot: MyBot,
    q: &CallbackQuery,
    ltag: LocaleTag,
    data: ParsedCallbackData<'_>,
) -> MyResult<()> {
    let new_name = data.2;

    DB.hand_pig.update_hrundel_name(q.from.id.0, new_name).await?;

    let text = lng("HandPigNameChangedResponse", ltag);
    bot.answer_callback_query(&q.id).text(text).await?;

    if let Some(id) = &q.inline_message_id {
        let text =
            lng("HandPigNameNowIs", ltag).args(&[("new_name", new_name)]);
        bot.edit_message_text_inline(id, text)
            .disable_web_page_preview(true)
            .await?;
    };
    Ok(())
}

async fn callback_find_day_pig(
    bot: MyBot,
    q: &CallbackQuery,
    ltag: LocaleTag,
    _data: ParsedCallbackData<'_>,
) -> MyResult<()> {
    let Some(im_id) = &q.inline_message_id else {
        return Ok(());
    };

    let result =
        DB.hand_pig.get_hryak_day_in_chat(&q.chat_instance, get_date()).await?;

    if let Some(hryak_today) = result {
        let text = lng("DayPigAlreadyFound", ltag)
            .args(&[("name", hryak_today.2.f_name)]);
        bot.answer_callback_query(&q.id).await?;
        bot.edit_message_text_inline(im_id, text).await?;
        return Ok(());
    }

    let hryak_chat = DB
        .hand_pig
        .get_rand_inline_user_group_by_chat(&q.chat_instance)
        .await?;

    if let Some(chat) = hryak_chat {
        let cur_date = get_date();
        DB.hand_pig.add_hryak_day_to_chat(chat.id, cur_date).await?;
        let result = DB
            .hand_pig
            .get_hryak_day_in_chat(&q.chat_instance, cur_date)
            .await?;
        if let Some(current_chat) = result {
            bot.answer_callback_query(&q.id).await?;

            let mention = user_mention(
                i64::try_from(current_chat.3.user_id).unwrap(),
                &current_chat.2.f_name,
            );

            let text1 = lng("DayPigLabel1", ltag);
            bot.edit_message_text_inline(im_id, text1).await?;

            sleep(Duration::from_secs(2)).await;

            let text2 = lng("DayPigLabel2", ltag);
            bot.edit_message_text_inline(im_id, text2).await?;

            sleep(Duration::from_secs(2)).await;
            let text3 = lng("DayPigLabel3", ltag);
            bot.edit_message_text_inline(im_id, text3).await?;

            sleep(Duration::from_secs(2)).await;

            let res = lng("DayPigFound", ltag).args(&[("mention", mention)]);
            bot.edit_message_text_inline(im_id, res).await?;
        }
    } else {
        bot.answer_callback_query(&q.id)
            .text(lng("HandPigNoInBarn", ltag))
            .await?;
        return Ok(());
    }
    Ok(())
}

async fn callback_add_inline_chat(
    bot: MyBot,
    q: &CallbackQuery,
    ltag: LocaleTag,
    data: ParsedCallbackData<'_>,
) -> MyResult<()> {
    if data.1 != q.from.id {
        callback_access_denied(bot, q, ltag).await?;
        return Ok(());
    }

    let Some(im_id) = &q.inline_message_id else { return Ok(()) };

    _check_or_insert_user_or_chat(q.from.id.0, &q.chat_instance).await?;

    let text = lng("ChatAddedToRating", ltag);
    bot.answer_callback_query(&q.id).text(text).await?;
    bot.edit_message_reply_markup_inline(im_id).await?;

    Ok(())
}

async fn callback_top10(
    bot: MyBot,
    q: &CallbackQuery,
    ltag: LocaleTag,
    data: ParsedCallbackData<'_>,
) -> MyResult<()> {
    bot.answer_callback_query(&q.id).await?;

    let Ok(top10var) = Top10Variant::from_str(data.2) else {
        return callback_empty(bot, q, ltag).await;
    };

    let function = match top10var {
        Top10Variant::Global => {
            top10_global(bot, q, ltag, top10var, Top10Variant::Chat).boxed()
        },
        Top10Variant::Chat => {
            top10_chat(bot, q, ltag, top10var, Top10Variant::Win).boxed()
        },
        Top10Variant::Win => {
            top10_win(bot, q, ltag, top10var, Top10Variant::Global).boxed()
        },
        // p_ = chat is private
        // I must define this in message creation, because i cant define chat type from callback
        Top10Variant::PGlobal => {
            top10_global(bot, q, ltag, top10var, Top10Variant::PWin).boxed()
        },
        Top10Variant::PWin => {
            top10_win(bot, q, ltag, top10var, Top10Variant::PGlobal).boxed()
        },
    };

    function.await?;

    Ok(())
}

async fn top10_chat(
    bot: MyBot,
    q: &CallbackQuery,
    ltag: LocaleTag,
    chat_type: Top10Variant,
    to: Top10Variant,
) -> MyResult<()> {
    let Some(im_id) = &q.inline_message_id else { return Ok(()) };

    _check_or_insert_user_or_chat(q.from.id.0, &q.chat_instance).await?;

    let top10_chat_info =
        DB.hand_pig.get_top10_chat(&q.chat_instance, get_date()).await?;

    let text = top10_chat_info.map_or_else(
        || lng("GameNoChatPigs", ltag),
        |v| generate_top10_text(ltag, v, chat_type.as_ref()),
    );

    let markup = keyboards::keyboard_in_top10(ltag, q.from.id, to.as_ref());
    bot.edit_message_text_inline(im_id, text)
        .reply_markup(markup)
        .disable_web_page_preview(true)
        .await?;
    Ok(())
}

async fn top10_global(
    bot: MyBot,
    q: &CallbackQuery,
    ltag: LocaleTag,
    chat_type: Top10Variant,
    to: Top10Variant,
) -> MyResult<()> {
    let Some(im_id) = &q.inline_message_id else { return Ok(()) };

    let cur_date = get_date();
    let top10_chat_info = DB.hand_pig.get_top10_global(cur_date).await?;

    let text = top10_chat_info.map_or_else(
        || lng("GameNoChatPigs", ltag),
        |v| generate_top10_text(ltag, v, chat_type.as_ref()),
    );

    let markup = keyboards::keyboard_in_top10(ltag, q.from.id, to.as_ref());

    bot.edit_message_text_inline(im_id, text)
        .reply_markup(markup)
        .disable_web_page_preview(true)
        .await?;
    Ok(())
}

async fn top10_win(
    bot: MyBot,
    q: &CallbackQuery,
    ltag: LocaleTag,
    chat_type: Top10Variant,
    to: Top10Variant,
) -> MyResult<()> {
    let Some(im_id) = &q.inline_message_id else { return Ok(()) };

    let top10_chat_info = DB.hand_pig.get_top10_win().await?;

    let text = top10_chat_info.map_or_else(
        || lng("GameNoChatPigs", ltag),
        |v| generate_top10_text(ltag, v, chat_type.as_ref()),
    );

    let markup = keyboards::keyboard_in_top10(ltag, q.from.id, to.as_ref());
    bot.edit_message_text_inline(im_id, text)
        .reply_markup(markup)
        .disable_web_page_preview(true)
        .await?;
    Ok(())
}

async fn callback_start_duel(
    bot: MyBot,
    q: &CallbackQuery,
    ltag: LocaleTag,
    data: ParsedCallbackData<'_>,
) -> MyResult<()> {
    let Some(im_id) = &q.inline_message_id else { return Ok(()) };

    if q.from.id == data.1 {
        let message = lng("InlineDuelCantFightSelf", ltag);
        bot.answer_callback_query(&q.id).text(message).await?;
        return Ok(());
    }

    // Preliminary check, if pig really exist
    let hrundel = DB.hand_pig.get_hrundel(q.from.id.0).await?;
    if hrundel.is_none() {
        let text = lng("HandPigNoInBarn", ltag);
        bot.answer_callback_query(&q.id).text(text).await?;
        return Ok(());
    };

    let thread_identifier = get_hash(im_id);

    let key = q.from.id.0;

    log::info!(
        "Try starting duel [{}] from user [{}] to [{}]",
        thread_identifier,
        key,
        data.1,
    );

    {
        if DUEL_LIST.contains(&thread_identifier) {
            log::warn!("Pending duel here!");
            return Ok(());
        } else {
            DUEL_LIST.insert(thread_identifier);
        };
    }

    {
        let maybe_threads = DUEL_LOCKS.try_get(&key);
        if maybe_threads.is_locked() {
            return Ok(());
        }

        if let Some(user_threads) = maybe_threads.try_unwrap() {
            let mut locked_threads = user_threads.lock().await;
            if locked_threads.contains(&thread_identifier) {
                log::error!("Found user thread duplicate!");
                return Ok(());
            }
            locked_threads.push(thread_identifier);
        } else {
            DUEL_LOCKS.insert(key, Mutex::new(vec![thread_identifier]));
        }
    }

    let some_item = DUEL_LOCKS.try_get(&key).try_unwrap();
    let Some(new_ref_item) = some_item else {
        log::error!("User threads cleaned after insert or locked!");
        return Ok(());
    };

    let mut new_item = new_ref_item.lock().await;

    let hrundels = _start_duel_get_2_hrundels((q.from.id, data.1)).await?;

    let Some([first, second]) = hrundels else {
        new_item.retain(|&x| x != thread_identifier);
        drop(new_item);
        DUEL_LIST.retain(|&x| x != thread_identifier);
        let text = lng("HandPigNoInBarn", ltag);
        bot.answer_callback_query(&q.id).text(text).await?;
        return Ok(());
    };

    log::info!(
        "Started duel [{}] from user [{}] to [{}]",
        thread_identifier,
        key,
        data.1,
    );

    let text = lng("InlineDuelGoingMessage", ltag).args(&[
        ("first_name", &first.0.name),
        ("secnd_name", &second.0.name),
        ("first_weight", &first.0.weight.to_string()),
        ("secnd_weight", &second.0.weight.to_string()),
    ]);

    bot.edit_message_text_inline(im_id, text)
        .disable_web_page_preview(true)
        .await?;
    bot.answer_callback_query(&q.id).await?;

    sleep(Duration::from_secs(3)).await;

    let ((winner, looser), damage, status) = _duel_get_winner(&first, &second);

    let lng_key = &format!("InlineDuelMessage_{}", &status);
    let text = lng(lng_key, ltag).args(&[
        ("winner_name", &winner.0.name),
        ("looser_name", &looser.0.name),
        ("diff", &damage.to_string()),
        ("winner_weight", &(winner.0.weight + damage).to_string()),
        ("looser_weight", &(looser.0.weight - damage).to_string()),
    ]);

    let winner_id = winner.1.user_id;
    let looser_id = looser.1.user_id;

    let looser_is_win = status == 0;

    DB.hand_pig.update_hrundel_duel(winner_id, damage, true).await?;
    DB.hand_pig.update_hrundel_duel(looser_id, damage, looser_is_win).await?;

    bot.edit_message_text_inline(im_id, text)
        .disable_web_page_preview(true)
        .await?;

    new_item.retain(|&x| x != thread_identifier);
    drop(new_item);
    DUEL_LIST.retain(|&x| x != thread_identifier);

    log::info!(
        "Ended duel [{}] from user [{}] to [{}]",
        thread_identifier,
        key,
        data.1,
    );

    Ok(())
}

async fn callback_change_top(
    bot: MyBot,
    q: &CallbackQuery,
    ltag: LocaleTag,
    data: ParsedCallbackData<'_>,
) -> MyResult<()> {
    let Some(m) = &q.message else { return Ok(()) };
    let Some(from) = m.from() else { return Ok(()) };

    let chat_info = DB.other.get_chat(m.chat.id.0).await?;

    let chat_info = if let Some(chat_info) = chat_info {
        chat_info
    } else {
        let cur_datetime = get_datetime();
        DB.other.add_chat(m.chat.id.0, cur_datetime).await?;
        let chat_info = DB.other.get_chat(m.chat.id.0).await?;
        let Some(chat_info) = chat_info else {
            return Ok(());
        };
        chat_info
    };

    let limit = chat_info.top10_setting;

    let offset = data.2.parse::<i64>().unwrap();

    let top50_pigs =
        DB.chat_pig.get_top50_chat_pigs(m.chat.id.0, limit, offset - 1).await?;

    if top50_pigs.is_empty() {
        let text = lng("HandPigNoInBarn", ltag);
        bot.send_message(m.chat.id, text).maybe_thread_id(m).await?;
        return Ok(());
    }

    let pig_count = DB.chat_pig.count_chat_pig(m.chat.id.0, limit).await?;

    let text = generate_chat_top50_text(ltag, top50_pigs, offset - 1);

    let is_end = pig_count < (TOP_LIMIT * offset);
    let markup = keyboards::keyboard_top50(ltag, offset, from.id, is_end);
    bot.edit_message_text(m.chat.id, m.id, text)
        .disable_web_page_preview(true)
        .reply_markup(markup)
        .await?;

    Ok(())
}

type Hrundel = (InlineUser, User);

fn _duel_get_winner<'a>(
    first: &'a Hrundel,
    second: &'a Hrundel,
) -> ((&'a Hrundel, &'a Hrundel), i32, i32) {
    let mut first_chance = first.0.weight;
    let mut second_chance = second.0.weight;

    if first_chance / second_chance > 5 {
        second_chance = first_chance / 5;
    } else if second_chance / first_chance > 5 {
        first_chance = second_chance / 5;
    }

    let first_random = rand::thread_rng().gen_range(0..first_chance);
    let second_random = rand::thread_rng().gen_range(0..second_chance);

    let mut status = 0;
    let (mut winner, mut looser) = (first, second);

    #[allow(clippy::comparison_chain)]
    if first_random > second_random {
        status = 1;
        if first_random >= (first.0.weight * 95) / 100 {
            status = 3;
            if first_random >= (first.0.weight * 95) / 100 {
                status = 5;
            }
        }
    } else if second_random > first_random {
        status = 2;
        if second_random >= (second.0.weight * 95) / 100 {
            status = 4;
            if second_random >= (second.0.weight * 95) / 100 {
                status = 6;
            }
        }
        winner = second;
        looser = first;
    }

    let damage = match status {
        1 | 2 => looser.0.weight / 8,
        3 | 4 => looser.0.weight / 3,
        5 | 6 => (looser.0.weight as f32 / 1.5) as i32,
        _ => std::cmp::max(first.0.weight, second.0.weight) / 8,
    };

    ((winner, looser), damage, status)
}

async fn _start_duel_get_2_hrundels(
    ids: (UserId, UserId),
) -> MyResult<Option<[(InlineUser, User); 2]>> {
    let Some(first) = DB.hand_pig.get_hrundel(ids.0 .0).await? else {
        return Ok(None);
    };

    let Some(second) = DB.hand_pig.get_hrundel(ids.1 .0).await? else {
        return Ok(None);
    };

    let today = get_date();

    let mut hrundels = [first, second];

    for hrundel in hrundels.iter_mut() {
        if hrundel.0.date != today {
            let user_id = hrundel.1.user_id;
            let size = formulas::calculate_hryak_size(user_id);
            let biggest = _get_biggest_chat_pig_mass(user_id).await?;
            let add =
                size + biggest + helpers::mass_addition_on_status(&hrundel.1);

            DB.hand_pig
                .update_hrundel_date_and_size(user_id, add, today)
                .await?;

            let Some(exist) = DB.hand_pig.get_hrundel(user_id).await? else {
                return Ok(None);
            };

            *hrundel = exist;
        }
    }

    Ok(Some(hrundels))
}

async fn callback_access_denied(
    bot: MyBot,
    q: &CallbackQuery,
    ltag: LocaleTag,
) -> MyResult<()> {
    let text = lng("UserAccessDeniedResponse", ltag);
    bot.answer_callback_query(&q.id).text(text).await?;
    Ok(())
}

async fn callback_error(
    bot: &MyBot,
    q: &CallbackQuery,
    ltag: LocaleTag,
) -> MyResult<()> {
    let text = lng("ErrorInlineTooMuchResponse", ltag);
    bot.answer_callback_query(&q.id).text(text).await?;
    let user_id = q.from.id;
    log::error!("Empty callback from user [{}]", user_id);

    Ok(())
}

async fn callback_empty(
    bot: MyBot,
    q: &CallbackQuery,
    ltag: LocaleTag,
) -> MyResult<()> {
    let text = lng("ErrorUndefCallbackResponse", ltag);
    bot.answer_callback_query(&q.id).text(text).await?;
    let user_id = q.from.id;
    log::error!("Empty callback from user [{}]", user_id);

    Ok(())
}

async fn callback_allow_voice(
    bot: MyBot,
    q: &CallbackQuery,
    mut ltag: LocaleTag,
    data: ParsedCallbackData<'_>,
) -> MyResult<()> {
    let Some(m) = &q.message else { return Ok(()) };
    let user_id = data.1;

    log::info!("Allowed voice from [{}]", user_id);
    if q.from.id.0 != BOT_CONFIG.creator_id {
        let text = lng("AccessDenied", ltag);
        bot.answer_callback_query(&q.id).text(text).await?;
        return Ok(());
    }
    let text = lng("VoiceAccepted", ltag);
    bot.answer_callback_query(&q.id).text(text).await?;
    let probably_url =
        BOT_CONFIG.content_check_channel_name.clone() + "/" + &m.id.to_string();

    let accepted = lng("Accepted", ltag);
    let edited_text = format!("{} {}", accepted, user_id);

    bot.edit_message_caption(m.chat.id, m.id)
        .caption(edited_text)
        .reply_markup(keyboards::keyboard_empty())
        .await?;

    let hrundel = DB.hand_pig.get_hrundel(user_id.0).await?;
    if let Some(hrundel) = hrundel {
        let final_mass = hrundel.0.weight + 250;
        let cur_date = get_date();

        DB.hand_pig
            .update_hrundel_date_and_size(user_id.0, final_mass, cur_date)
            .await?;
        ltag = tag_one_or(hrundel.1.lang.as_deref(), DEFAULT_LANG_TAG);
    }
    let Some(user) =
        DB.other.maybe_get_or_insert_user(user_id.0, get_datetime).await?
    else {
        return Ok(());
    };
    DB.other.add_voice(user.id, probably_url).await?;

    let voices = DB.other.get_voices_by_user(user.id).await?;
    let number = voices.last().map_or(0, |v| v.id);

    let text = lng("VoiceAcceptedCongrats", ltag).args(&[("number", number)]);
    bot.send_message(user_id, text).maybe_thread_id(m).await?;

    Ok(())
}

async fn callback_disallow_voice(
    bot: MyBot,
    q: &CallbackQuery,
    mut ltag: LocaleTag,
    data: ParsedCallbackData<'_>,
) -> MyResult<()> {
    let Some(m) = &q.message else { return Ok(()) };

    let user_id = data.1;

    log::info!("Disallowed voice from [{}]", user_id);

    if q.from.id.0 != BOT_CONFIG.creator_id {
        let text = lng("AccessDenied", ltag);
        bot.answer_callback_query(&q.id).text(text).await?;
        return Ok(());
    }
    let text = lng("VoiceNotAccepted", ltag);
    bot.answer_callback_query(&q.id).text(text).await?;

    let not_accepted = lng("NotAccepted", ltag);
    let edited_text = format!("{} {}", not_accepted, user_id);
    bot.edit_message_caption(m.chat.id, m.id)
        .caption(edited_text)
        .reply_markup(keyboards::keyboard_empty())
        .await?;

    let hrundel = DB.hand_pig.get_hrundel(user_id.0).await?;
    if let Some(hrundel) = hrundel {
        ltag = tag_one_or(hrundel.1.lang.as_deref(), DEFAULT_LANG_TAG);
    }

    let text = lng("VoiceNotAcceptedMsg", ltag);
    bot.send_message(user_id, text).maybe_thread_id(m).await?;
    Ok(())
}

async fn callback_change_flag(
    bot: MyBot,
    q: &CallbackQuery,
    ltag: LocaleTag,
    data: ParsedCallbackData<'_>,
) -> MyResult<()> {
    if data.1 != q.from.id {
        callback_access_denied(bot, q, ltag).await?;
        return Ok(());
    }

    let Some(im_id) = &q.inline_message_id else { return Ok(()) };
    let probably_flag = Flags::from_code(data.2).unwrap_or(Flags::Us);

    DB.hand_pig
        .update_hrundel_flag(q.from.id.0, probably_flag.to_code())
        .await?;

    let text = lng("HandPigFlagChangeResponse", ltag)
        .args(&[("flag", probably_flag.to_emoji())]);
    bot.edit_message_text_inline(im_id, &text).await?;
    bot.answer_callback_query(&q.id).text(text).await?;
    Ok(())
}

async fn callback_change_lang(
    bot: MyBot,
    q: &CallbackQuery,
    ltag: LocaleTag,
    data: ParsedCallbackData<'_>,
) -> MyResult<()> {
    if data.1 != q.from.id {
        callback_access_denied(bot, q, ltag).await?;
        return Ok(());
    }

    let Some(im_id) = &q.inline_message_id else { return Ok(()) };
    let probably_code = data.2;

    let (text, res) = if probably_code == "-" {
        DB.other.change_user_lang(q.from.id.0, None).await?;

        let text = lng("InlineLangDeleteResponse", ltag);
        (text, None)
    } else {
        let probably_flag = Flags::from_code(data.2).unwrap_or(Flags::Us);

        let new_ltag = tag(probably_code);
        let text = lng("InlineLangChangeResponse", new_ltag).args(&[
            ("code", probably_code),
            ("flag", probably_flag.to_emoji()),
        ]);
        (text, Some(probably_code))
    };

    DB.other.change_user_lang(q.from.id.0, res).await?;

    bot.edit_message_text_inline(im_id, &text).await?;
    bot.answer_callback_query(&q.id).text(text).await?;
    Ok(())
}

async fn _get_biggest_chat_pig_mass(id_user: u64) -> MyResult<i32> {
    let biggest = DB.chat_pig.get_biggest_chat_pig(id_user).await?;
    let biggest_mass = biggest.map_or(0, |b| b.mass);

    Ok(biggest_mass)
}

async fn callback_gift<'a>(
    bot: MyBot,
    q: &'a CallbackQuery,
    ltag: LocaleTag,
    _data: ParsedCallbackData<'a>,
) -> MyResult<()> {
    let chat_id = ChatId(BOT_CONFIG.channel_id);
    let channel_member = bot.get_chat_member(chat_id, q.from.id).await?;

    if !channel_member.is_present() {
        let text = lng("SubscribeChannelFirstResponse", ltag);
        bot.answer_callback_query(&q.id).text(text).await?;
        return Ok(());
    }

    let Some(hrundel) = DB.hand_pig.get_hrundel(q.from.id.0).await? else {
        let text = lng("HandPigNoInBarn", ltag);
        bot.answer_callback_query(&q.id).text(text).await?;
        return Ok(());
    };

    if hrundel.0.gifted {
        let text = lng("GiftAlreadyTakenTomorrow", ltag);
        bot.answer_callback_query(&q.id).text(text).await?;
        return Ok(());
    }

    let hrundel_on_update = UpdateInlineUser {
        id: hrundel.0.id,
        date: hrundel.0.date,
        f_name: &q.from.first_name,
        weight: hrundel.0.weight + DAILY_GIFT_AMOUNT,
        gifted: true,
    };

    DB.hand_pig.update_hrundel(hrundel_on_update).await?;

    let text = lng("GiftThanksReceive500", ltag);
    bot.answer_callback_query(&q.id).text(text).await?;

    Ok(())
}

async fn callback_check_subscribe<'a>(
    bot: MyBot,
    q: &'a CallbackQuery,
    ltag: LocaleTag,
    _data: ParsedCallbackData<'a>,
) -> MyResult<()> {
    let chat_id = ChatId(BOT_CONFIG.channel_id);
    let channel_member = bot.get_chat_member(chat_id, q.from.id).await?;

    if !channel_member.is_present() {
        let text = lng("SubscribeChannelFirstResponse", ltag);
        bot.answer_callback_query(&q.id).text(text).await?;
        return Ok(());
    }

    let Some(hrundel) = DB.hand_pig.get_hrundel(q.from.id.0).await? else {
        let text = lng("HandPigNoInBarn", ltag);
        bot.answer_callback_query(&q.id).text(text).await?;
        return Ok(());
    };

    if hrundel.1.subscribed || hrundel.1.supported {
        let text = lng("GiftAlreadyTaken", ltag);
        bot.answer_callback_query(&q.id).text(text).await?;
        return Ok(());
    }

    let hrundel_on_update = UpdateInlineUser {
        id: hrundel.0.id,
        date: hrundel.0.date,
        f_name: &q.from.first_name,
        weight: hrundel.0.weight + SUBSCRIBE_GIFT,
        gifted: false,
    };

    let user_status = UserStatus {
        subscribed: true,
        banned: hrundel.1.banned,
        started: hrundel.1.started,
        supported: hrundel.1.supported,
    };
    DB.hand_pig.update_hrundel(hrundel_on_update).await?;
    DB.other.change_user_status(q.from.id.0, user_status).await?;

    let text = lng("GiftThanksReceive100", ltag);
    bot.answer_callback_query(&q.id).text(text).await?;

    Ok(())
}

async fn _check_or_insert_user_or_chat(
    user_id: u64,
    chat_instance: &str,
) -> MyResult<()> {
    let user_in_group =
        DB.hand_pig.get_group_user(chat_instance, user_id).await?;

    // check if actually user or chat does exist, and insert missing
    if user_in_group.is_some() {
        return Ok(());
    };

    // and insert missing chat, or user to chat join
    // user dont be created without his consent
    let user = DB.hand_pig.get_hrundel(user_id).await?;
    let chat = DB.hand_pig.get_inline_group(chat_instance).await?;

    if let Some(user) = user {
        if let Some(chat) = chat {
            DB.hand_pig.add_group_to_user(user.0.id, chat.id).await?;
        } else {
            let cur_datetime = get_datetime();
            DB.hand_pig.add_inline_group(chat_instance, cur_datetime).await?;

            let Some(chat) =
                DB.hand_pig.get_inline_group(chat_instance).await?
            else {
                return Ok(());
            };
            DB.hand_pig.add_group_to_user(user.0.id, chat.id).await?;
        }
    }

    Ok(())
}
