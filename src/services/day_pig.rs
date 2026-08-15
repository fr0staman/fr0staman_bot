use ahash::AHashMap;
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

#[cfg_attr(test, derive(Debug))]
pub struct Candidate {
    pub user: User,
    pub iug: Option<InlineUsersGroup>,
    pub game: Option<Game>,
}

/// Hand pigs seen in this inline chat plus chat pigs in the linked group,
/// keyed by user id so someone in both is drawn once.
pub fn build_candidates(
    hand_users: Vec<(InlineUsersGroup, User)>,
    chat_users: Vec<(Game, User)>,
) -> AHashMap<i32, Candidate> {
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

    candidates
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

    let (hand_users, chat_users) = tokio::try_join!(
        DB.hand_pig.get_inline_users_with_user_by_chat(&chat_instance),
        DB.chat_pig.get_game_users_by_group(group_id),
    )?;

    let mut candidates = build_candidates(hand_users, chat_users);

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
                DB.hand_pig.add_hrundel(new_hand_pig).await?.id
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

    let (in_this_chat, global_unique) =
        DB.other.count_achievements_for_notice(game_id, uid).await?;

    let all_count = Ach::COUNT.to_string();
    let chat_count = in_this_chat.to_string();
    let global_count = global_unique.to_string();

    for ach in achievements {
        let achievement_name =
            lng(&format!("Achievement_{}", *ach as i16), ltag);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models::InlineUsersGroup;
    use crate::test_support::{game, user};

    fn iug(id: i32, iu_id: i32) -> InlineUsersGroup {
        InlineUsersGroup { id, iu_id, ig_id: 1 }
    }

    #[test]
    fn hand_pig_only_users_become_candidates() {
        let candidates =
            build_candidates(vec![(iug(1, 1), user(10, 1000))], vec![]);

        assert_eq!(candidates.len(), 1);
        let entry = &candidates[&10];
        assert!(entry.iug.is_some());
        assert!(entry.game.is_none());
    }

    #[test]
    fn chat_pig_only_users_become_candidates() {
        let candidates = build_candidates(vec![], vec![(game(50), user(11, 1001))]);

        assert_eq!(candidates.len(), 1);
        let entry = &candidates[&11];
        assert!(entry.iug.is_none());
        assert_eq!(entry.game.as_ref().unwrap().mass, 50);
    }

    #[test]
    fn a_user_in_both_pools_is_drawn_only_once() {
        let candidates = build_candidates(
            vec![(iug(1, 1), user(10, 1000))],
            vec![(game(50), user(10, 1000))],
        );

        assert_eq!(candidates.len(), 1, "the same user must not be duplicated");
        let entry = &candidates[&10];
        assert!(entry.iug.is_some());
        assert!(entry.game.is_some());
    }

    #[test]
    fn the_pools_are_merged_by_internal_id() {
        let candidates = build_candidates(
            vec![(iug(1, 1), user(10, 1000)), (iug(2, 2), user(20, 2000))],
            vec![(game(50), user(20, 2000)), (game(60), user(30, 3000))],
        );

        assert_eq!(candidates.len(), 3);
        assert!(candidates[&10].game.is_none());
        assert!(candidates[&20].iug.is_some() && candidates[&20].game.is_some());
        assert!(candidates[&30].iug.is_none());
    }

    #[test]
    fn empty_pools_produce_no_candidates() {
        assert!(build_candidates(vec![], vec![]).is_empty());
    }

    #[test]
    fn a_repeated_row_does_not_duplicate_a_candidate() {
        let candidates = build_candidates(
            vec![(iug(1, 1), user(10, 1000)), (iug(2, 1), user(10, 1000))],
            vec![],
        );

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[&10].iug.as_ref().unwrap().id, 2);
    }
}
