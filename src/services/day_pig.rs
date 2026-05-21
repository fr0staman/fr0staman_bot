use ahash::{AHashMap, AHashSet};
use chrono::NaiveDate;
use strum::EnumCount as _;
use teloxide::{
    prelude::Requester,
    types::{ChatId, UserId},
};

use crate::{
    config::consts::DEFAULT_LANG_TAG,
    db::{
        DB,
        models::{Game, InlineUsersGroup, NewInlineUser, User},
    },
    lang::{InnerLang, LocaleTag, lng},
    services::achievements::{self, Ach},
    types::{MyBot, MyResult},
    utils::{
        date::get_date,
        formulas::calculate_hryak_size,
        helpers::{escape, truncate},
    },
};

pub struct SelectedDayPig {
    pub user: User,
    pub game_id: Option<i32>,
    pub new_achievements: Vec<Ach>,
    #[allow(dead_code)]
    pub has_chat_pig: bool,
}

pub enum DayPigSelectResult {
    Selected(SelectedDayPig),
    Escaped,
}

struct Candidate {
    user: User,
    iug: Option<InlineUsersGroup>,
    game: Option<Game>,
}

pub async fn select_and_record(
    bot: &MyBot,
    ig_id: i32,
    group_id: i32,
    telegram_chat_id: Option<ChatId>,
    cur_date: NaiveDate,
) -> MyResult<Option<DayPigSelectResult>> {
    let Some(inline_group) = DB.hand_pig.get_inline_group_by_id(ig_id).await?
    else {
        return Ok(None);
    };
    let chat_instance = inline_group.chat_instance.to_string();

    if DB
        .hand_pig
        .get_hryak_day_in_chat(&chat_instance, cur_date)
        .await?
        .is_some()
    {
        return Ok(None);
    }

    let hand_users = DB
        .hand_pig
        .get_inline_users_with_user_by_chat(&chat_instance)
        .await?;
    let chat_users = DB.chat_pig.get_game_users_by_group(group_id).await?;

    let mut candidates: AHashMap<i32, Candidate> = AHashMap::new();

    for (iug, user) in hand_users {
        candidates
            .entry(user.id)
            .or_insert_with(|| Candidate { user, iug: None, game: None })
            .iug = Some(iug);
    }

    for (game, user) in chat_users {
        let entry = candidates
            .entry(user.id)
            .or_insert_with(|| Candidate { user, iug: None, game: None });
        entry.game = Some(game);
    }

    if candidates.is_empty() {
        return Ok(None);
    }

    let chosen_user_id = {
        use rand::seq::IteratorRandom as _;
        let mut rng = rand::rng();
        candidates.keys().copied().choose(&mut rng)
    };
    let Some(uid) = chosen_user_id else {
        return Ok(None);
    };
    let Some(chosen) = candidates.remove(&uid) else {
        return Ok(None);
    };

    let iug_id = match &chosen.iug {
        Some(iug) => iug.id,
        None => {
            let user = &chosen.user;

            let existing = DB.hand_pig.get_hrundel(user.user_id).await?;
            let inline_user_id = if let Some((iu, _)) = existing {
                iu.id
            } else {
                let game_mass = chosen.game.as_ref().map_or(0, |g| g.mass);
                let weight = calculate_hryak_size(user.user_id) + game_mass;
                let escaped = escape(&user.first_name);
                let name = truncate(&escaped, 64).0;
                let flag = user.lang.as_deref().unwrap_or(DEFAULT_LANG_TAG);

                let new_hand_pig = NewInlineUser {
                    uid: user.id,
                    weight,
                    date: get_date(),
                    flag,
                    win: 0,
                    rout: 0,
                    name,
                    gifted: false,
                };
                DB.hand_pig.add_hrundel(new_hand_pig).await?;

                DB.hand_pig
                    .get_hrundel(user.user_id)
                    .await?
                    .ok_or_else(|| {
                        crate::types::MyError::Unknown(
                            "InlineUser not found after creation".to_owned(),
                        )
                    })?
                    .0
                    .id
            };

            let iug =
                DB.hand_pig.get_or_create_iug(inline_user_id, ig_id).await?;
            iug.id
        },
    };

    if let Some(chat_id) = telegram_chat_id {
        let user_id = UserId(chosen.user.user_id as u64);
        let in_chat = match bot.get_chat_member(chat_id, user_id).await {
            Ok(member) => member.is_present(),
            Err(_) => true,
        };
        if !in_chat {
            return Ok(Some(DayPigSelectResult::Escaped));
        }
    }

    DB.hand_pig.add_hryak_day_to_chat(iug_id, cur_date).await?;

    let has_chat_pig = chosen.game.is_some();
    let game_id = chosen.game.as_ref().map(|g| g.id);

    let user = DB
        .hand_pig
        .get_hryak_day_in_chat(&chat_instance, cur_date)
        .await?
        .map(|(_, _, _, user)| user)
        .ok_or_else(|| {
            crate::types::MyError::Unknown(
                "Day pig not found after recording".to_owned(),
            )
        })?;

    let new_achievements = if let Some(gid) = game_id {
        achievements::check_day_pig_achievement(gid).await.unwrap_or_default()
    } else {
        vec![]
    };

    Ok(Some(DayPigSelectResult::Selected(SelectedDayPig {
        user,
        game_id,
        new_achievements,
        has_chat_pig,
    })))
}

pub async fn notify_achievements(
    bot: &MyBot,
    chat_id: ChatId,
    ltag: LocaleTag,
    game_id: i32,
    uid: i32,
    achievements: &[Ach],
) -> MyResult<()> {
    if achievements.is_empty() {
        return Ok(());
    }

    let all_in_db = DB.other.get_achievements_by_uid(uid).await?;
    let in_this_chat: Vec<_> =
        all_in_db.iter().filter(|v| v.game_id == game_id).collect();
    let global_unique: AHashSet<_> =
        all_in_db.iter().map(|v| v.code).collect();

    let all_count = Ach::COUNT.to_string();
    let chat_count = in_this_chat.len().to_string();
    let global_count = global_unique.len().to_string();

    for ach in achievements {
        let achievement_name =
            lng(&format!("Achievement_{}", ach.clone() as i16), ltag);
        let text = lng("NewAchievementUnlocked", ltag).args(&[
            ("achievement_name", &achievement_name),
            ("chat_count", &chat_count),
            ("chat_all_count", &all_count),
            ("global_count", &global_count),
            ("global_all_count", &all_count),
        ]);
        bot.send_message(chat_id, text).await?;
    }

    Ok(())
}
