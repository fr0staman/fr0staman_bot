//! Service layer against a real database. These reach for the global `DB`,
//! which the harness redirects; see `src/tests/common.rs`.

use crate::tests::common::{date, datetime};
use crate::services::achievements::{
    self, Ach, PigSnapshot, check_achievements,
};

macro_rules! db {
    () => {
        match crate::tests::common::test_db().await {
            Some(t) => t,
            None => return,
        }
    };
}

async fn stored_codes(t: &crate::tests::common::TestDb, game_id: i32) -> Vec<i16> {
    let mut codes: Vec<i16> = t
        .db
        .other
        .get_achievements_by_game_id(game_id)
        .await
        .unwrap()
        .iter()
        .map(|a| a.code)
        .collect();
    codes.sort_unstable();
    codes
}


#[tokio::test]
async fn a_newly_unlocked_achievement_is_persisted() {
    let t = db!();

    let user = t.seed_user(1_001).await;
    let group = t.seed_group(-100_001).await;
    let pig = t.seed_chat_pig(&user, &group, 100, date(2026, 7, 28)).await;

    let now = datetime(2026, 7, 28, 15, 0);
    t.seed_grow_log(pig.id, now, 5, 100).await;

    let unlocked =
        check_achievements(PigSnapshot::from(&pig), now).await.unwrap();

    assert!(unlocked.contains(&Ach::HundredClub));
    assert!(stored_codes(&t, pig.id).await.contains(&(Ach::HundredClub as i16)));
}

#[tokio::test]
async fn a_second_check_does_not_duplicate_what_is_already_unlocked() {
    let t = db!();

    let user = t.seed_user(1_001).await;
    let group = t.seed_group(-100_001).await;
    let pig = t.seed_chat_pig(&user, &group, 100, date(2026, 7, 28)).await;

    let now = datetime(2026, 7, 28, 15, 0);
    t.seed_grow_log(pig.id, now, 5, 100).await;

    let first = check_achievements(PigSnapshot::from(&pig), now).await.unwrap();
    assert!(!first.is_empty());

    let before = stored_codes(&t, pig.id).await;

    let second = check_achievements(PigSnapshot::from(&pig), now).await.unwrap();
    assert!(second.is_empty(), "nothing new the second time");
    assert_eq!(stored_codes(&t, pig.id).await, before);
}

#[tokio::test]
async fn nothing_is_written_when_nothing_is_unlocked() {
    let t = db!();

    let user = t.seed_user(1_001).await;
    let group = t.seed_group(-100_001).await;
    let pig = t.seed_chat_pig(&user, &group, 5, date(2026, 7, 15)).await;

    // A date with no calendar trigger, and a mass below every threshold.
    let now = datetime(2026, 7, 15, 13, 0);
    t.seed_grow_log(pig.id, now, 1, 5).await;

    let unlocked =
        check_achievements(PigSnapshot::from(&pig), now).await.unwrap();

    assert!(unlocked.is_empty(), "unexpectedly unlocked {unlocked:?}");
    assert!(stored_codes(&t, pig.id).await.is_empty());
}

#[tokio::test]
async fn the_evaluator_reads_the_pigs_own_history_only() {
    let t = db!();

    let group = t.seed_group(-100_001).await;
    let a = t.seed_user(1_001).await;
    let b = t.seed_user(1_002).await;
    let mine = t.seed_chat_pig(&a, &group, 5, date(2026, 7, 15)).await;
    let theirs = t.seed_chat_pig(&b, &group, 5, date(2026, 7, 15)).await;

    let now = datetime(2026, 7, 15, 13, 0);
    t.seed_grow_log(theirs.id, now, 20, 25).await;
    t.seed_grow_log(mine.id, now, 1, 5).await;

    let unlocked =
        check_achievements(PigSnapshot::from(&mine), now).await.unwrap();

    assert!(!unlocked.contains(&Ach::MonsterGrow));
}

#[tokio::test]
async fn several_achievements_unlock_in_one_pass() {
    let t = db!();

    let user = t.seed_user(1_001).await;
    let group = t.seed_group(-100_001).await;
    let pig = t.seed_chat_pig(&user, &group, 1_000, date(2026, 2, 14)).await;
    let now = datetime(2026, 2, 14, 0, 0);
    t.seed_grow_log(pig.id, now, 20, 1_000).await;

    let unlocked =
        check_achievements(PigSnapshot::from(&pig), now).await.unwrap();

    for expected in [
        Ach::HundredClub,
        Ach::FiveMetersOfFat,
        Ach::TonOfPig,
        Ach::MonsterGrow,
        Ach::LovePig,
        Ach::ZeroHour,
    ] {
        assert!(unlocked.contains(&expected), "missing {expected:?}");
    }

    assert_eq!(stored_codes(&t, pig.id).await.len(), unlocked.len());
}


#[tokio::test]
async fn social_achievements_unlock_from_the_qualifying_chat_count() {
    use crate::config::consts::ACTIVE_GROUP_MIN_PIGS;

    let t = db!();

    let owner = t.seed_user(1_001).await;
    let fillers = ACTIVE_GROUP_MIN_PIGS as i32 - 1;
    let (_, first_pig) =
        t.seed_group_with_pigs(-100_001, &owner, fillers).await;
    let now = datetime(2026, 7, 15, 13, 0);

    let unlocked =
        check_achievements(PigSnapshot::from(&first_pig), now).await.unwrap();
    assert!(!unlocked.contains(&Ach::PigInTwoChats));
    t.seed_group_with_pigs(-100_002, &owner, fillers).await;

    let unlocked =
        check_achievements(PigSnapshot::from(&first_pig), now).await.unwrap();
    assert!(unlocked.contains(&Ach::PigInTwoChats));
    assert!(!unlocked.contains(&Ach::PigEverywhere));
}

#[tokio::test]
async fn a_chat_below_the_pig_threshold_does_not_count_towards_social() {
    use crate::config::consts::ACTIVE_GROUP_MIN_PIGS;

    let t = db!();

    let owner = t.seed_user(1_001).await;

    let (_, pig) = t
        .seed_group_with_pigs(-100_001, &owner, ACTIVE_GROUP_MIN_PIGS as i32 - 1)
        .await;
    t.seed_group_with_pigs(-100_002, &owner, 0).await;

    let now = datetime(2026, 7, 15, 13, 0);
    let unlocked =
        check_achievements(PigSnapshot::from(&pig), now).await.unwrap();

    assert!(!unlocked.contains(&Ach::PigInTwoChats));
}


#[tokio::test]
async fn the_rename_achievement_is_awarded_once() {
    let t = db!();

    let user = t.seed_user(1_001).await;
    let group = t.seed_group(-100_001).await;
    let pig = t.seed_chat_pig(&user, &group, 10, date(2026, 7, 28)).await;

    let first = achievements::check_name_achievements(pig.id).await.unwrap();
    assert_eq!(first, vec![Ach::Pigolator]);

    let second = achievements::check_name_achievements(pig.id).await.unwrap();
    assert!(second.is_empty());

    assert_eq!(
        stored_codes(&t, pig.id).await,
        vec![Ach::Pigolator as i16]
    );
}

#[tokio::test]
async fn the_day_pig_achievement_is_awarded_once() {
    let t = db!();

    let user = t.seed_user(1_001).await;
    let group = t.seed_group(-100_001).await;
    let pig = t.seed_chat_pig(&user, &group, 10, date(2026, 7, 28)).await;

    let first = achievements::check_day_pig_achievement(pig.id).await.unwrap();
    assert_eq!(first, vec![Ach::PigOfTheDay]);

    let second = achievements::check_day_pig_achievement(pig.id).await.unwrap();
    assert!(second.is_empty());

    assert_eq!(stored_codes(&t, pig.id).await, vec![Ach::PigOfTheDay as i16]);
}


mod day_pig {
    use super::*;
    use crate::services::day_pig::{
        DayPigSelectResult, select_and_record,
    };

    /// Only used for the membership check, skipped via
    /// `telegram_chat_id: None`.
    fn bot() -> crate::types::MyBot {
        use teloxide::prelude::*;
        Bot::new("0000000000:TEST").parse_mode(
            crate::config::consts::BOT_PARSE_MODE,
        )
    }

    #[tokio::test]
    async fn a_chat_with_no_candidates_draws_nobody() {
        let t = db!();

        let group = t.seed_group(-100_001).await;
        let inline_group = t.seed_inline_group(111).await;

        let result = select_and_record(
            &bot(),
            inline_group.id,
            group.id,
            None,
            date(2026, 7, 28),
        )
        .await
        .unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn a_lone_hand_pig_owner_always_wins() {
        let t = db!();

        let user = t.seed_user(1_001).await;
        let group = t.seed_group(-100_001).await;
        let hand_pig = t.seed_hand_pig(&user, 100, date(2026, 7, 28)).await;
        let inline_group = t.seed_inline_group(111).await;
        t.link_hand_pig_to_inline_group(&hand_pig, &inline_group).await;

        let result = select_and_record(
            &bot(),
            inline_group.id,
            group.id,
            None,
            date(2026, 7, 28),
        )
        .await
        .unwrap()
        .expect("no draw happened");

        match result {
            DayPigSelectResult::Selected(pig) => {
                assert_eq!(pig.user.user_id, 1_001);
                assert!(pig.game_id.is_none(), "no chat pig for this user");
            },
            DayPigSelectResult::Escaped => panic!("unexpectedly escaped"),
        }
    }

    #[tokio::test]
    async fn the_draw_happens_only_once_per_chat_per_day() {
        let t = db!();

        let user = t.seed_user(1_001).await;
        let group = t.seed_group(-100_001).await;
        let hand_pig = t.seed_hand_pig(&user, 100, date(2026, 7, 28)).await;
        let inline_group = t.seed_inline_group(111).await;
        t.link_hand_pig_to_inline_group(&hand_pig, &inline_group).await;

        let today = date(2026, 7, 28);

        assert!(
            select_and_record(&bot(), inline_group.id, group.id, None, today)
                .await
                .unwrap()
                .is_some()
        );

        // A second call the same day is a no-op...
        assert!(
            select_and_record(&bot(), inline_group.id, group.id, None, today)
                .await
                .unwrap()
                .is_none()
        );

        // ...but tomorrow is open again.
        assert!(
            select_and_record(
                &bot(),
                inline_group.id,
                group.id,
                None,
                date(2026, 7, 29)
            )
            .await
            .unwrap()
            .is_some()
        );
    }

    #[tokio::test]
    async fn a_chat_pig_owner_gets_a_hand_pig_created_on_the_fly() {
        let t = db!();

        let user = t.seed_user(1_001).await;
        let group = t.seed_group(-100_001).await;
        t.seed_chat_pig(&user, &group, 250, date(2026, 7, 28)).await;
        let inline_group = t.seed_inline_group(111).await;

        assert!(t.db.hand_pig.get_hrundel(1_001).await.unwrap().is_none());

        let result = select_and_record(
            &bot(),
            inline_group.id,
            group.id,
            None,
            date(2026, 7, 28),
        )
        .await
        .unwrap()
        .expect("no draw happened");

        match result {
            DayPigSelectResult::Selected(pig) => {
                assert_eq!(pig.user.user_id, 1_001);
                assert!(pig.game_id.is_some());
                assert!(
                    pig.new_achievements.contains(&Ach::PigOfTheDay),
                    "a chat-pig winner earns PigOfTheDay"
                );
            },
            DayPigSelectResult::Escaped => panic!("unexpectedly escaped"),
        }

        let created =
            t.db.hand_pig.get_hrundel(1_001).await.unwrap().expect("no hand pig");
        assert!(
            created.0.weight >= 250,
            "the new hand pig is seeded from the chat pig's mass"
        );
    }

    #[tokio::test]
    async fn a_user_in_both_pools_is_only_a_single_candidate() {
        let t = db!();

        let user = t.seed_user(1_001).await;
        let group = t.seed_group(-100_001).await;
        t.seed_chat_pig(&user, &group, 50, date(2026, 7, 28)).await;
        let hand_pig = t.seed_hand_pig(&user, 100, date(2026, 7, 28)).await;
        let inline_group = t.seed_inline_group(111).await;
        t.link_hand_pig_to_inline_group(&hand_pig, &inline_group).await;

        let result = select_and_record(
            &bot(),
            inline_group.id,
            group.id,
            None,
            date(2026, 7, 28),
        )
        .await
        .unwrap()
        .expect("no draw happened");

        match result {
            DayPigSelectResult::Selected(pig) => {
                // Both halves of the candidate were merged.
                assert_eq!(pig.user.user_id, 1_001);
                assert!(pig.game_id.is_some(), "the chat pig side was kept");
            },
            DayPigSelectResult::Escaped => panic!("unexpectedly escaped"),
        }

        // Exactly one row in `hryak_day` for the day.
        assert!(
            t.db.hand_pig
                .get_hryak_day_in_chat("111", date(2026, 7, 28))
                .await
                .unwrap()
                .is_some()
        );
    }

    #[tokio::test]
    async fn an_unknown_inline_group_draws_nobody() {
        let t = db!();

        let group = t.seed_group(-100_001).await;

        let result = select_and_record(
            &bot(),
            9_999,
            group.id,
            None,
            date(2026, 7, 28),
        )
        .await
        .unwrap();

        assert!(result.is_none());
    }
}
