use dashmap::DashMap;
use futures::{future::BoxFuture, FutureExt};
use once_cell::sync::Lazy;
use rand::Rng;
use teloxide::{
    payloads::{
        AnswerCallbackQuerySetters, EditMessageCaptionSetters,
        EditMessageTextInlineSetters, EditMessageTextSetters,
    },
    requests::Requester,
    types::{CallbackQuery, UserId},
    utils::html::{bold, user_mention},
};

use crate::{
    config::BOT_CONFIG,
    db::DB,
    enums::{Actions, Top10Variant},
    keyboards,
    lang::{get_tag, lng, tag, LocaleTag},
    models::{Game, InlineUser},
    traits::MaybeMessageSetter,
    utils::{
        date::{get_date, get_datetime},
        flag::get_flag,
        formulas,
        helpers::{self, get_hash},
    },
    InnerLang, MyBot, MyResult, TOP_LIMIT,
};

use std::{str::FromStr, sync::Arc};

use tokio::{
    sync::Mutex,
    time::{sleep, Duration},
};

static DUEL_LOCKS: Lazy<Arc<DashMap<u64, Mutex<Vec<u64>>>>> =
    Lazy::new(|| Arc::new(DashMap::new()));
static DUEL_LIST: Lazy<Mutex<Vec<u64>>> = Lazy::new(|| Mutex::new(Vec::new()));

type ParsedCallbackData<'a> = (&'a str, UserId, &'a str);

pub async fn filter_callback_commands(
    bot: MyBot,
    q: CallbackQuery,
) -> MyResult<()> {
    let ltag = tag(get_tag(&q.from));

    let temp_bot = bot.clone();
    let temp_q = q.clone();

    let cloned_data = q.data.clone().unwrap_or_default();
    let decoded_data = helpers::decode_callback_data(&cloned_data);

    let function = match decoded_data {
        Some(payload) => _inner_filter(bot, q, ltag, payload),
        None => callback_empty(bot, q, ltag).boxed(),
    };

    let response = function.await;

    if let Err(err) = response {
        _handle_error(temp_bot, temp_q, ltag, err).await?;
    } else {
        log::info!(
            "Handled callback [{}]: user: [{}]",
            temp_q.id,
            temp_q.from.id
        );
    }

    Ok(())
}

fn _inner_filter(
    bot: MyBot,
    q: CallbackQuery,
    ltag: LocaleTag,
    data: ParsedCallbackData<'_>,
) -> BoxFuture<'_, MyResult<()>> {
    let Ok(matched_enum) = Actions::from_str(data.0) else {
        return callback_empty(bot, q, ltag).boxed();
    };

    match matched_enum {
        Actions::GiveName => {
            callback_give_hand_pig_name(bot, q, ltag, data).boxed()
        },
        Actions::FindHryak => callback_find_day_pig(bot, q, ltag, data).boxed(),
        Actions::AddChat => {
            callback_add_inline_chat(bot, q, ltag, data).boxed()
        },
        Actions::Top10 => callback_top10(bot, q, ltag, data).boxed(),
        Actions::StartDuel => callback_start_duel(bot, q, ltag, data).boxed(),
        Actions::TopLeft | Actions::TopRight => {
            callback_change_top(bot, q, ltag, data).boxed()
        },
        Actions::AllowVoice => callback_allow_voice(bot, q, ltag, data).boxed(),
        Actions::DisallowVoice => {
            callback_disallow_voice(bot, q, ltag, data).boxed()
        },
    }
}

async fn _handle_error(
    bot: MyBot,
    q: CallbackQuery,
    ltag: LocaleTag,
    err: crate::Error,
) -> MyResult<()> {
    let Some(inline_message_id) = q.inline_message_id.clone() else { return Ok(()) };

    log::error!("Error in callback: {:?}", err);

    tokio::spawn(async move {
        {
            let locks = DUEL_LOCKS.clone();
            if let Some(value) = locks.get(&q.from.id.0) {
                let mut user_threads = value.lock().await;
                user_threads.clear();
                log::warn!("Cleaned threads for user [{}]", &q.from.id)
            };
        };
        {
            let thread_identifier = get_hash(&inline_message_id);

            let mut going_duels = DUEL_LIST.lock().await;
            going_duels.retain(|&x| x != thread_identifier);
        };
    });
    callback_empty(bot, q, ltag).await?;
    Ok(())
}
async fn callback_give_hand_pig_name(
    bot: MyBot,
    q: CallbackQuery,
    ltag: LocaleTag,
    data: ParsedCallbackData<'_>,
) -> MyResult<()> {
    let new_name = data.2;

    DB.hand_pig.update_hrundel_name(q.from.id.0, new_name).await?;

    let text = lng("HandPigNameChangedResponse", ltag);
    bot.answer_callback_query(q.id).text(text).await?;

    if let Some(id) = q.inline_message_id {
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
    q: CallbackQuery,
    ltag: LocaleTag,
    _data: ParsedCallbackData<'_>,
) -> MyResult<()> {
    let Some(im_id) = q.inline_message_id else {
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
            bot.answer_callback_query(q.id).await?;

            let mention = user_mention(
                i64::try_from(current_chat.2.user_id).unwrap(),
                &current_chat.2.f_name,
            );

            let text1 = lng("DayPigLabel1", ltag);
            bot.edit_message_text_inline(&im_id, text1).await?;

            sleep(Duration::from_secs(2)).await;

            let text2 = lng("DayPigLabel2", ltag);
            bot.edit_message_text_inline(&im_id, text2).await?;

            sleep(Duration::from_secs(2)).await;
            let text3 = lng("DayPigLabel3", ltag);
            bot.edit_message_text_inline(&im_id, text3).await?;

            sleep(Duration::from_secs(2)).await;

            let res = lng("DayPigFound", ltag).args(&[("mention", mention)]);
            bot.edit_message_text_inline(&im_id, res).await?;
        }
    } else {
        bot.answer_callback_query(q.id)
            .text(lng("HandPigNoInBarn", ltag))
            .await?;
        return Ok(());
    }
    Ok(())
}

async fn callback_add_inline_chat(
    bot: MyBot,
    q: CallbackQuery,
    ltag: LocaleTag,
    data: ParsedCallbackData<'_>,
) -> MyResult<()> {
    if data.1 != q.from.id {
        callback_access_denied(bot, q, ltag).await?;
        return Ok(());
    }

    let Some(im_id) = q.inline_message_id else { return Ok(()) };

    _check_or_insert_user_or_chat(q.from.id.0, &q.chat_instance).await?;

    let text = lng("ChatAddedToRating", ltag);
    bot.answer_callback_query(q.id).text(text).await?;
    bot.edit_message_reply_markup_inline(im_id).await?;

    Ok(())
}

async fn callback_top10(
    bot: MyBot,
    q: CallbackQuery,
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
    q: CallbackQuery,
    ltag: LocaleTag,
    chat_type: Top10Variant,
    to: Top10Variant,
) -> MyResult<()> {
    let Some(im_id) = q.inline_message_id else { return Ok(()) };

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
    q: CallbackQuery,
    ltag: LocaleTag,
    chat_type: Top10Variant,
    to: Top10Variant,
) -> MyResult<()> {
    let Some(im_id) = q.inline_message_id else { return Ok(()) };

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
    q: CallbackQuery,
    ltag: LocaleTag,
    chat_type: Top10Variant,
    to: Top10Variant,
) -> MyResult<()> {
    let Some(im_id) = q.inline_message_id else { return Ok(()) };

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
    q: CallbackQuery,
    ltag: LocaleTag,
    data: ParsedCallbackData<'_>,
) -> MyResult<()> {
    let Some(inline_message_id) = q.inline_message_id else { return Ok(()) };

    if q.from.id == data.1 {
        let message = lng("InlineDuelCantFightSelf", ltag);
        bot.answer_callback_query(q.id).text(message).await?;
        return Ok(());
    }

    let thread_identifier = get_hash(&inline_message_id);

    let key = q.from.id.0;
    let locks = DUEL_LOCKS.clone();

    {
        let mut going_duels = DUEL_LIST.lock().await;

        if going_duels.contains(&thread_identifier) {
            log::warn!("Pending duel here!");
            return Ok(());
        } else {
            going_duels.push(thread_identifier);
        };
    }

    {
        if let Some(user_threads) = locks.get(&key) {
            let mut locked_threads = user_threads.lock().await;
            if locked_threads.contains(&thread_identifier) {
                log::error!("Found user thread duplicate!");
                return Ok(());
            }
            locked_threads.push(thread_identifier);
        } else {
            locks.insert(key, Mutex::new(vec![thread_identifier]));
        }
    }

    let Some(new_item) = locks.get(&key) else {
        log::error!("User threads cleaned after insert!");
        return Ok(())
    };

    let mut new_item = new_item.lock().await;

    let hrundels = _start_duel_get_2_hrundels((q.from.id, data.1)).await?;

    let Some([first, second]) = hrundels else {
        let text = lng("HandPigNoInBarn", ltag);
        bot.answer_callback_query(q.id).text(text).await?;
        return Ok(());
    };

    let text = lng("InlineDuelGoingMessage", ltag).args(&[
        ("first_name", bold(&first.name)),
        ("secnd_name", bold(&second.name)),
        ("first_weight", bold(&(first.weight).to_string())),
        ("secnd_weight", bold(&(second.weight).to_string())),
    ]);

    bot.edit_message_text_inline(&inline_message_id, text)
        .disable_web_page_preview(true)
        .await?;
    bot.answer_callback_query(q.id).await?;

    sleep(Duration::from_secs(3)).await;

    let ((winner, looser), damage, status) = _duel_get_winner(&first, &second);

    let key = &format!("InlineDuelMessage_{}", &status);
    let text = lng(key, ltag).args(&[
        ("winner_name", bold(&winner.name)),
        ("looser_name", bold(&looser.name)),
        ("diff", bold(&damage.to_string())),
        ("winner_weight", bold(&(winner.weight + damage).to_string())),
        ("looser_weight", bold(&(looser.weight - damage).to_string())),
    ]);

    let winner_id = winner.user_id;
    let looser_id = looser.user_id;

    let looser_is_win = status == 0;

    DB.hand_pig.update_hrundel_duel(winner_id, damage, true).await?;
    DB.hand_pig.update_hrundel_duel(looser_id, damage, looser_is_win).await?;

    bot.edit_message_text_inline(inline_message_id, text)
        .disable_web_page_preview(true)
        .await?;
    {
        new_item.retain(|&x| x != thread_identifier);

        let mut going_duels = DUEL_LIST.lock().await;
        going_duels.retain(|&x| x != thread_identifier);
    }

    Ok(())
}

async fn callback_change_top(
    bot: MyBot,
    q: CallbackQuery,
    ltag: LocaleTag,
    data: ParsedCallbackData<'_>,
) -> MyResult<()> {
    let Some(m) = q.message else { return Ok(()) };
    let Some(from) = m.from() else { return Ok(())};

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
        bot.send_message(m.chat.id, text).maybe_thread_id(&m).await?;
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

fn _duel_get_winner<'a>(
    first: &'a InlineUser,
    second: &'a InlineUser,
) -> ((&'a InlineUser, &'a InlineUser), i32, i32) {
    let mut first_chance = first.weight;
    let mut second_chance = second.weight;

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
        if first_random >= (first.weight * 95) / 100 {
            status = 3;
            if first_random >= (first.weight * 95) / 100 {
                status = 5;
            }
        }
    } else if second_random > first_random {
        status = 2;
        if second_random >= (second.weight * 95) / 100 {
            status = 4;
            if second_random >= (second.weight * 95) / 100 {
                status = 6;
            }
        }
        winner = second;
        looser = first;
    }

    let damage = match status {
        1 | 2 => looser.weight / 8,
        3 | 4 => looser.weight / 3,
        5 | 6 => (looser.weight as f32 / 1.5) as i32,
        _ => std::cmp::max(first.weight, second.weight) / 8,
    };

    ((winner, looser), damage, status)
}

async fn _start_duel_get_2_hrundels(
    ids: (UserId, UserId),
) -> MyResult<Option<[InlineUser; 2]>> {
    let Some(first) = DB.hand_pig.get_hrundel(ids.0.0).await? else {
            return Ok(None);
        };

    let Some(second) = DB.hand_pig.get_hrundel(ids.1.0).await? else {
        return Ok(None);
    };

    let today = get_date();

    let mut hrundels = [first, second];

    for hrundel in hrundels.iter_mut() {
        if hrundel.date != today {
            let user_id = hrundel.user_id;
            let size = formulas::calculate_hryak_size(user_id);
            let biggest = _get_biggest_chat_pig_mass(user_id).await?;
            let add = size
                + biggest
                + helpers::mass_addition_on_status(hrundel.status);

            DB.hand_pig
                .update_hrundel_date_and_size(user_id, add, today)
                .await?;

            let Some(exist) = DB.hand_pig.get_hrundel(user_id).await? else {
                return Ok(None)
            };

            *hrundel = exist;
        }
    }

    Ok(Some(hrundels))
}

async fn callback_access_denied(
    bot: MyBot,
    q: CallbackQuery,
    ltag: LocaleTag,
) -> MyResult<()> {
    let text = lng("UserAccessDeniedResponse", ltag);
    bot.answer_callback_query(q.id).text(text).await?;
    Ok(())
}

async fn callback_empty(
    bot: MyBot,
    q: CallbackQuery,
    ltag: LocaleTag,
) -> MyResult<()> {
    let text = lng("ErrorUndefCallbackResponse", ltag);
    bot.answer_callback_query(q.id).text(text).await?;
    let user_id = q.from.id;
    log::error!("Empty callback from user [{}]", user_id);

    Ok(())
}

async fn callback_allow_voice(
    bot: MyBot,
    q: CallbackQuery,
    mut ltag: LocaleTag,
    data: ParsedCallbackData<'_>,
) -> MyResult<()> {
    let Some(m) = q.message else { return Ok(())};
    let user_id = data.1;

    log::info!("Allowed voice from [{}]", user_id);
    if q.from.id.0 != BOT_CONFIG.creator_id {
        let text = lng("AccessDenied", ltag);
        bot.answer_callback_query(q.id).text(text).await?;
        return Ok(());
    }
    let text = lng("VoiceAccepted", ltag);
    bot.answer_callback_query(q.id).text(text).await?;
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
        let final_mass = hrundel.weight + 250;
        let cur_date = get_date();

        DB.hand_pig
            .update_hrundel_date_and_size(user_id.0, final_mass, cur_date)
            .await?;
        ltag = tag(&hrundel.lang);
    }
    DB.other.add_voice(user_id.0, probably_url).await?;

    let voices = DB.other.get_voice(user_id.0).await?;
    let number = voices.last().map_or(0, |v| v.id);

    let text = lng("VoiceAcceptedCongrats", ltag).args(&[("number", number)]);
    bot.send_message(user_id, text).maybe_thread_id(&m).await?;

    Ok(())
}

async fn callback_disallow_voice(
    bot: MyBot,
    q: CallbackQuery,
    mut ltag: LocaleTag,
    data: ParsedCallbackData<'_>,
) -> MyResult<()> {
    let Some(m) = q.message else { return Ok(())};

    let user_id = data.1;

    log::info!("Disallowed voice from [{}]", user_id);

    if q.from.id.0 != BOT_CONFIG.creator_id {
        let text = lng("AccessDenied", ltag);
        bot.answer_callback_query(q.id).text(text).await?;
        return Ok(());
    }
    let text = lng("VoiceAccepted", ltag);
    bot.answer_callback_query(q.id).text(text).await?;

    let not_accepted = lng("NotAccepted", ltag);
    let edited_text = format!("{} {}", not_accepted, user_id);
    bot.edit_message_caption(m.chat.id, m.id)
        .caption(edited_text)
        .reply_markup(keyboards::keyboard_empty())
        .await?;

    let hrundel = DB.hand_pig.get_hrundel(user_id.0).await?;
    if let Some(hrundel) = hrundel {
        ltag = tag(&hrundel.lang);
    }

    let text = lng("VoiceNotAcceptedMsg", ltag);
    bot.send_message(user_id, text).maybe_thread_id(&m).await?;
    Ok(())
}

async fn _get_biggest_chat_pig_mass(id_user: u64) -> MyResult<i32> {
    let biggest = DB.chat_pig.get_biggest_chat_pig(id_user).await?;
    let biggest_mass = biggest.map_or(0, |b| b.mass);

    Ok(biggest_mass)
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
            DB.hand_pig.add_group_to_user(user.id, chat.id).await?;
        } else {
            DB.hand_pig.add_inline_group(chat_instance).await?;

            let Some(chat) = DB.hand_pig.get_inline_group(chat_instance).await? else {
                return Ok(());
            };
            DB.hand_pig.add_group_to_user(user.id, chat.id).await?;
        }
    }

    Ok(())
}

pub fn generate_top10_text(
    ltag: LocaleTag,
    top10_info: Vec<InlineUser>,
    chat_type: &str,
) -> String {
    let chat_type = chat_type.replace("p_", "");

    let text = lng(&format!("InlineTop10Header_{}", chat_type), ltag);
    let header = bold(&text);

    let mut result = String::with_capacity(512) + &header + "\n";

    let is_win = chat_type == "win";

    for (index, item) in top10_info.iter().enumerate() {
        let value = if is_win { item.win as i32 } else { item.weight };

        let key = format!("InlineTop10Line_{}", chat_type);

        let line = lng(&key, ltag).args(&[
            ("number", (index + 1).to_string()),
            ("flag", get_flag(&item.lang).to_string()),
            ("name", helpers::escape(&item.name)),
            ("value", value.to_string()),
        ]);

        result += &("\n".to_owned() + &line);
    }

    result
}

pub fn generate_chat_top50_text(
    ltag: LocaleTag,
    top50_info: Vec<Game>,
    offset_multiplier: i64,
) -> String {
    let text = lng("GameTop50Header", ltag);
    let header = bold(&text);

    let mut result = String::with_capacity(512) + &header;

    for (index, item) in top50_info.iter().enumerate() {
        let value = item.mass;
        let index = (index as i64) + (offset_multiplier * TOP_LIMIT);

        let line = lng("GameTop50Line", ltag).args(&[
            ("number", (index + 1).to_string()),
            ("name", helpers::escape(&item.name)),
            ("value", value.to_string()),
        ]);

        result += &("\n".to_owned() + &line);
    }

    result
}