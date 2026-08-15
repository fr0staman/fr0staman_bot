//! `DB.chat_pig` — the group pig game's queries.
//!
//! Requires `TEST_DATABASE_URL`; see `src/tests/common.rs`.

use crate::tests::common::{date, datetime};
use crate::config::consts::{
    ACTIVE_GROUP_MIN_PIGS, CHAT_PIG_START_MASS, TOP_LIMIT,
    TOP_LIMIT_WITH_CHARTS,
};

macro_rules! db {
    () => {
        match crate::tests::common::test_db().await {
            Some(t) => t,
            None => return,
        }
    };
}

#[tokio::test]
async fn a_chat_pig_is_found_by_its_telegram_ids() {
    let t = db!();

    let user = t.seed_user(1_001).await;
    let group = t.seed_group(-100_001).await;
    let pig = t.seed_chat_pig(&user, &group, 42, date(2026, 7, 28)).await;

    let found = t.db.chat_pig.get_chat_pig(1_001, -100_001).await.unwrap();

    assert_eq!(found.map(|g| (g.id, g.mass)), Some((pig.id, 42)));
}

#[tokio::test]
async fn a_pig_is_scoped_to_one_chat() {
    let t = db!();

    let user = t.seed_user(1_001).await;
    let a = t.seed_group(-100_001).await;
    let b = t.seed_group(-100_002).await;

    t.seed_chat_pig(&user, &a, 10, date(2026, 7, 28)).await;

    assert!(t.db.chat_pig.get_chat_pig(1_001, -100_001).await.unwrap().is_some());
    assert!(t.db.chat_pig.get_chat_pig(1_001, -100_002).await.unwrap().is_none());

    // A pig in the other chat is a separate row with its own mass.
    t.seed_chat_pig(&user, &b, 99, date(2026, 7, 28)).await;
    let in_b = t.db.chat_pig.get_chat_pig(1_001, -100_002).await.unwrap();
    assert_eq!(in_b.unwrap().mass, 99);
}

#[tokio::test]
async fn an_unknown_user_or_chat_yields_nothing() {
    let t = db!();

    assert!(t.db.chat_pig.get_chat_pig(999, -999).await.unwrap().is_none());
}

#[tokio::test]
async fn the_biggest_chat_pig_is_the_heaviest_across_all_chats() {
    let t = db!();

    let user = t.seed_user(1_001).await;
    for (i, mass) in [(1, 10), (2, 500), (3, 120)] {
        let group = t.seed_group(-100_000 - i).await;
        t.seed_chat_pig(&user, &group, mass, date(2026, 7, 28)).await;
    }

    let biggest = t.db.chat_pig.get_biggest_chat_pig(1_001).await.unwrap();

    assert_eq!(biggest.unwrap().mass, 500);
}

#[tokio::test]
async fn creating_a_pig_stores_the_starting_mass_and_last_fed_date() {
    let t = db!();

    let user = t.seed_user(1_001).await;
    let group = t.seed_group(-100_001).await;

    let pig = t
        .db
        .chat_pig
        .create_chat_pig(
            user.id,
            group.id,
            "Хрюндель",
            date(2026, 7, 27),
            CHAT_PIG_START_MASS,
        )
        .await
        .unwrap();

    assert_eq!(pig.mass, CHAT_PIG_START_MASS);
    assert_eq!(pig.date, date(2026, 7, 27));
    assert_eq!(pig.name, "Хрюндель");
}

#[tokio::test]
async fn feeding_updates_both_mass_and_date() {
    let t = db!();

    let user = t.seed_user(1_001).await;
    let group = t.seed_group(-100_001).await;
    let pig = t.seed_chat_pig(&user, &group, 10, date(2026, 7, 27)).await;

    t.db.chat_pig
        .set_chat_pig_mass_n_date(pig.id, 25, date(2026, 7, 28))
        .await
        .unwrap();

    let after = t.db.chat_pig.get_chat_pig(1_001, -100_001).await.unwrap().unwrap();
    assert_eq!(after.mass, 25);
    assert_eq!(after.date, date(2026, 7, 28));
}

#[tokio::test]
async fn renaming_only_touches_the_pig_in_that_chat() {
    let t = db!();

    let user = t.seed_user(1_001).await;
    let a = t.seed_group(-100_001).await;
    let b = t.seed_group(-100_002).await;
    t.seed_chat_pig(&user, &a, 10, date(2026, 7, 28)).await;
    t.seed_chat_pig(&user, &b, 10, date(2026, 7, 28)).await;

    t.db.chat_pig
        .set_chat_pig_name(1_001, -100_001, "Renamed".to_owned())
        .await
        .unwrap();

    let in_a = t.db.chat_pig.get_chat_pig(1_001, -100_001).await.unwrap().unwrap();
    let in_b = t.db.chat_pig.get_chat_pig(1_001, -100_002).await.unwrap().unwrap();

    assert_eq!(in_a.name, "Renamed");
    assert_eq!(in_b.name, "Test pig");
}


#[tokio::test]
async fn the_top_is_ordered_by_mass_descending() {
    let t = db!();

    let group = t.seed_group(-100_001).await;
    for (i, mass) in [(1, 30), (2, 10), (3, 50)] {
        let user = t.seed_user(1_000 + i).await;
        t.seed_chat_pig(&user, &group, mass, date(2026, 7, 28)).await;
    }

    let top =
        t.db.chat_pig.get_top_chat_pigs(-100_001, 0, 0, false).await.unwrap();

    let masses: Vec<i32> = top.iter().map(|g| g.mass).collect();
    assert_eq!(masses, vec![50, 30, 10]);
}

#[tokio::test]
async fn the_top_excludes_pigs_at_or_below_the_minimum() {
    let t = db!();

    let group = t.seed_group(-100_001).await;
    for (i, mass) in [(1, 5), (2, 10), (3, 20)] {
        let user = t.seed_user(1_000 + i).await;
        t.seed_chat_pig(&user, &group, mass, date(2026, 7, 28)).await;
    }
    let top =
        t.db.chat_pig.get_top_chat_pigs(-100_001, 10, 0, false).await.unwrap();

    assert_eq!(top.len(), 1);
    assert_eq!(top[0].mass, 20);
}

#[tokio::test]
async fn the_top_pages_by_the_chart_aware_limit() {
    let t = db!();

    let group = t.seed_group(-100_001).await;
    let total = TOP_LIMIT_WITH_CHARTS + 5;
    for i in 0..total {
        let user = t.seed_user(2_000 + i).await;
        t.seed_chat_pig(&user, &group, (total - i) as i32, date(2026, 7, 28))
            .await;
    }

    let page_one =
        t.db.chat_pig.get_top_chat_pigs(-100_001, 0, 0, true).await.unwrap();
    assert_eq!(page_one.len() as i64, TOP_LIMIT_WITH_CHARTS);

    let page_two =
        t.db.chat_pig.get_top_chat_pigs(-100_001, 0, 1, true).await.unwrap();
    assert_eq!(page_two.len(), 5);
    let plain =
        t.db.chat_pig.get_top_chat_pigs(-100_001, 0, 0, false).await.unwrap();
    assert_eq!(plain.len() as i64, total.min(TOP_LIMIT));
}

#[tokio::test]
async fn counting_pigs_respects_the_same_minimum_as_the_top() {
    let t = db!();

    let group = t.seed_group(-100_001).await;
    for (i, mass) in [(1, 5), (2, 10), (3, 20)] {
        let user = t.seed_user(1_000 + i).await;
        t.seed_chat_pig(&user, &group, mass, date(2026, 7, 28)).await;
    }

    assert_eq!(t.db.chat_pig.count_chat_pig(-100_001, 0).await.unwrap(), 3);
    assert_eq!(t.db.chat_pig.count_chat_pig(-100_001, 10).await.unwrap(), 1);
    assert_eq!(t.db.chat_pig.count_active_pigs(group.id).await.unwrap(), 3);
}


#[tokio::test]
async fn only_chats_with_enough_pigs_count_as_active() {
    let t = db!();

    let owner = t.seed_user(1_001).await;
    let below = (ACTIVE_GROUP_MIN_PIGS - 2) as i32;
    let at = (ACTIVE_GROUP_MIN_PIGS - 1) as i32;
    let above = ACTIVE_GROUP_MIN_PIGS as i32;

    t.seed_group_with_pigs(-100_001, &owner, below).await;
    assert_eq!(
        t.db.chat_pig.count_active_chats_by_uid(owner.id).await.unwrap(),
        0,
        "a chat with {} pigs is below the threshold",
        below + 1
    );

    t.seed_group_with_pigs(-100_002, &owner, at).await;
    assert_eq!(
        t.db.chat_pig.count_active_chats_by_uid(owner.id).await.unwrap(),
        1,
        "a chat with exactly {ACTIVE_GROUP_MIN_PIGS} pigs counts"
    );

    t.seed_group_with_pigs(-100_003, &owner, above).await;
    assert_eq!(
        t.db.chat_pig.count_active_chats_by_uid(owner.id).await.unwrap(),
        2
    );
}

#[tokio::test]
async fn active_chats_are_counted_per_user() {
    let t = db!();

    let owner = t.seed_user(1_001).await;
    let bystander = t.seed_user(1_002).await;

    t.seed_group_with_pigs(-100_001, &owner, ACTIVE_GROUP_MIN_PIGS as i32).await;

    assert_eq!(
        t.db.chat_pig.count_active_chats_by_uid(owner.id).await.unwrap(),
        1
    );
    assert_eq!(
        t.db.chat_pig.count_active_chats_by_uid(bystander.id).await.unwrap(),
        0
    );
}


#[tokio::test]
async fn the_grow_log_comes_back_oldest_first() {
    let t = db!();

    let user = t.seed_user(1_001).await;
    let group = t.seed_group(-100_001).await;
    let pig = t.seed_chat_pig(&user, &group, 10, date(2026, 7, 28)).await;
    t.seed_grow_log(pig.id, datetime(2026, 7, 28, 12, 0), 3, 10).await;
    t.seed_grow_log(pig.id, datetime(2026, 7, 26, 12, 0), 5, 5).await;
    t.seed_grow_log(pig.id, datetime(2026, 7, 27, 12, 0), 2, 7).await;

    let log = t.db.chat_pig.get_grow_log_by_game(pig.id).await.unwrap();

    let weights: Vec<i32> = log.iter().map(|l| l.current_weight).collect();
    assert_eq!(weights, vec![5, 7, 10]);
}

#[tokio::test]
async fn the_fourteen_day_window_drops_older_entries() {
    let t = db!();

    let user = t.seed_user(1_001).await;
    let group = t.seed_group(-100_001).await;
    let pig = t.seed_chat_pig(&user, &group, 10, date(2026, 7, 28)).await;

    let today = datetime(2026, 7, 28, 12, 0);
    t.seed_grow_log(pig.id, today, 1, 30).await;
    t.seed_grow_log(pig.id, today - chrono::Duration::days(13), 1, 20).await;
    t.seed_grow_log(pig.id, today - chrono::Duration::days(14), 1, 10).await;

    let window =
        t.db.chat_pig.get_grow_log_by_game_14days(pig.id, today).await.unwrap();

    let weights: Vec<i32> = window.iter().map(|l| l.current_weight).collect();
    assert_eq!(weights, vec![20, 30], "the 14-day-old row must be excluded");
    assert_eq!(
        t.db.chat_pig.get_grow_log_by_game(pig.id).await.unwrap().len(),
        3
    );
}

#[tokio::test]
async fn the_fourteen_day_window_excludes_the_future() {
    let t = db!();

    let user = t.seed_user(1_001).await;
    let group = t.seed_group(-100_001).await;
    let pig = t.seed_chat_pig(&user, &group, 10, date(2026, 7, 28)).await;

    let today = datetime(2026, 7, 28, 12, 0);
    t.seed_grow_log(pig.id, today + chrono::Duration::hours(1), 1, 11).await;

    let window =
        t.db.chat_pig.get_grow_log_by_game_14days(pig.id, today).await.unwrap();

    assert!(window.is_empty());
}

#[tokio::test]
async fn a_grow_log_belongs_to_exactly_one_pig() {
    let t = db!();

    let group = t.seed_group(-100_001).await;
    let a = t.seed_user(1_001).await;
    let b = t.seed_user(1_002).await;
    let pig_a = t.seed_chat_pig(&a, &group, 10, date(2026, 7, 28)).await;
    let pig_b = t.seed_chat_pig(&b, &group, 10, date(2026, 7, 28)).await;

    t.seed_grow_log(pig_a.id, datetime(2026, 7, 28, 12, 0), 1, 11).await;

    assert_eq!(
        t.db.chat_pig.get_grow_log_by_game(pig_a.id).await.unwrap().len(),
        1
    );
    assert!(
        t.db.chat_pig.get_grow_log_by_game(pig_b.id).await.unwrap().is_empty()
    );
}


#[tokio::test]
async fn a_groups_players_are_listed_with_their_users() {
    let t = db!();

    let group = t.seed_group(-100_001).await;
    for i in 0..3i64 {
        let user = t.seed_user(1_001 + i).await;
        t.seed_chat_pig(&user, &group, 10, date(2026, 7, 28)).await;
    }

    let rows = t.db.chat_pig.get_game_users_by_group(group.id).await.unwrap();
    assert_eq!(rows.len(), 3);

    let mut ids: Vec<i64> =
        t.db.chat_pig.get_pig_user_ids_by_group(group.id).await.unwrap();
    ids.sort_unstable();
    assert_eq!(ids, vec![1_001, 1_002, 1_003]);
}


#[tokio::test]
async fn a_reset_zeroes_every_pig_and_wipes_their_achievements() {
    use crate::db::models::AchievementUserAdd;

    let t = db!();

    let group = t.seed_group(-100_001).await;
    let other_group = t.seed_group(-100_002).await;

    let a = t.seed_user(1_001).await;
    let b = t.seed_user(1_002).await;
    let pig_a = t.seed_chat_pig(&a, &group, 500, date(2026, 7, 20)).await;
    let pig_b = t.seed_chat_pig(&b, &group, 300, date(2026, 7, 20)).await;
    let outsider =
        t.seed_chat_pig(&a, &other_group, 900, date(2026, 7, 20)).await;

    for pig in [&pig_a, &pig_b, &outsider] {
        t.db.other
            .add_achievements(&[AchievementUserAdd {
                game_id: pig.id,
                code: 203,
                created_at: datetime(2026, 7, 20, 12, 0),
            }])
            .await
            .unwrap();
    }

    t.db.chat_pig.soft_reset_pigs(group.id, date(2026, 7, 28)).await.unwrap();

    for (telegram_id, chat_id) in [(1_001, -100_001), (1_002, -100_001)] {
        let pig =
            t.db.chat_pig.get_chat_pig(telegram_id, chat_id).await.unwrap().unwrap();
        assert_eq!(pig.mass, CHAT_PIG_START_MASS);
        assert_eq!(pig.date, date(2026, 7, 28));
        assert!(
            t.db.other
                .get_achievements_by_game_id(pig.id)
                .await
                .unwrap()
                .is_empty()
        );
    }

    let survivor =
        t.db.chat_pig.get_chat_pig(1_001, -100_002).await.unwrap().unwrap();
    assert_eq!(survivor.mass, 900);
    assert_eq!(
        t.db.other.get_achievements_by_game_id(outsider.id).await.unwrap().len(),
        1
    );
}

#[tokio::test]
async fn resetting_an_empty_group_is_a_no_op() {
    let t = db!();

    let group = t.seed_group(-100_001).await;
    t.db.chat_pig.soft_reset_pigs(group.id, date(2026, 7, 28)).await.unwrap();
}


#[tokio::test]
async fn the_growth_chart_query_returns_each_pig_with_its_window() {
    let t = db!();

    let group = t.seed_group(-100_001).await;
    let today = datetime(2026, 7, 28, 12, 0);

    for i in 0..3i64 {
        let user = t.seed_user(1_001 + i).await;
        let pig =
            t.seed_chat_pig(&user, &group, (i as i32 + 1) * 10, date(2026, 7, 28))
                .await;
        t.seed_grow_log(pig.id, today, 1, (i as i32 + 1) * 10).await;
        t.seed_grow_log(pig.id, today - chrono::Duration::days(20), 1, 1).await;
    }

    let data =
        t.db.chat_pig.get_top10_by_14days_growth(-100_001, today).await.unwrap();

    assert_eq!(data.len(), 3);
    assert_eq!(data[0].0.mass, 30);
    for (_, logs) in &data {
        assert_eq!(logs.len(), 1, "only the in-window row");
    }
}

#[tokio::test]
async fn the_growth_chart_query_caps_at_ten_pigs() {
    let t = db!();

    let group = t.seed_group(-100_001).await;
    for i in 0..14i64 {
        let user = t.seed_user(1_001 + i).await;
        t.seed_chat_pig(&user, &group, (14 - i) as i32, date(2026, 7, 28)).await;
    }

    let data = t
        .db
        .chat_pig
        .get_top10_by_14days_growth(-100_001, datetime(2026, 7, 28, 12, 0))
        .await
        .unwrap();

    assert_eq!(data.len(), 10);
}
