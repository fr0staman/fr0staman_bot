//! `DB.hand_pig` — inline-mode pigs, duels, inline groups and day pig.
//!
//! Requires `TEST_DATABASE_URL`; see `src/tests/common.rs`.

use crate::tests::common::{date, datetime};

macro_rules! db {
    () => {
        match crate::tests::common::test_db().await {
            Some(t) => t,
            None => return,
        }
    };
}

const TODAY: fn() -> chrono::NaiveDate = || date(2026, 7, 28);


#[tokio::test]
async fn a_hand_pig_is_found_with_its_owner() {
    let t = db!();

    let user = t.seed_user(1_001).await;
    t.seed_hand_pig(&user, 250, TODAY()).await;

    let found = t.db.hand_pig.get_hrundel(1_001).await.unwrap();
    let (pig, owner) = found.expect("hand pig missing");

    assert_eq!(pig.weight, 250);
    assert_eq!(owner.user_id, 1_001);
}

#[tokio::test]
async fn a_user_without_a_hand_pig_yields_nothing() {
    let t = db!();

    t.seed_user(1_001).await;
    assert!(t.db.hand_pig.get_hrundel(1_001).await.unwrap().is_none());
    assert!(t.db.hand_pig.get_hrundel(999_999).await.unwrap().is_none());
}

#[tokio::test]
async fn renaming_and_reflagging_persist() {
    let t = db!();

    let user = t.seed_user(1_001).await;
    t.seed_hand_pig(&user, 100, TODAY()).await;

    t.db.hand_pig.update_hrundel_name(1_001, "Новий Хрюндель").await.unwrap();
    t.db.hand_pig.update_hrundel_flag(1_001, "gb").await.unwrap();

    let (pig, _) = t.db.hand_pig.get_hrundel(1_001).await.unwrap().unwrap();
    assert_eq!(pig.name, "Новий Хрюндель");
    assert_eq!(pig.flag, "gb");
}

#[tokio::test]
async fn the_daily_recompute_resets_the_gift_flag() {
    use crate::db::models::UpdateInlineUser;

    let t = db!();

    let user = t.seed_user(1_001).await;
    let mut pig = t.seed_hand_pig(&user, 100, date(2026, 7, 27)).await;
    pig.gifted = true;

    t.db.hand_pig
        .update_hrundel(UpdateInlineUser {
            id: pig.id,
            weight: 777,
            date: TODAY(),
            gifted: false,
        })
        .await
        .unwrap();

    let (after, _) = t.db.hand_pig.get_hrundel(1_001).await.unwrap().unwrap();
    assert_eq!(after.weight, 777);
    assert_eq!(after.date, TODAY());
    assert!(!after.gifted);
}


#[tokio::test]
async fn a_duel_win_adds_weight_and_a_win() {
    let t = db!();

    let user = t.seed_user(1_001).await;
    t.seed_hand_pig(&user, 100, TODAY()).await;

    t.db.hand_pig.update_hrundel_duel(1_001, 40, true).await.unwrap();

    let (pig, _) = t.db.hand_pig.get_hrundel(1_001).await.unwrap().unwrap();
    assert_eq!(pig.weight, 140);
    assert_eq!(pig.win, 1);
    assert_eq!(pig.rout, 0);
}

#[tokio::test]
async fn a_duel_loss_subtracts_weight_and_adds_a_rout() {
    let t = db!();

    let user = t.seed_user(1_001).await;
    t.seed_hand_pig(&user, 100, TODAY()).await;

    t.db.hand_pig.update_hrundel_duel(1_001, 40, false).await.unwrap();

    let (pig, _) = t.db.hand_pig.get_hrundel(1_001).await.unwrap().unwrap();
    assert_eq!(pig.weight, 60);
    assert_eq!(pig.win, 0);
    assert_eq!(pig.rout, 1);
}

#[tokio::test]
async fn a_losing_pig_floors_at_one_kilogram() {
    let t = db!();

    // `CASE WHEN weight > offset THEN weight - offset ELSE 1`. One pig per
    // case, all inside the single lock this test holds.
    let cases = [(100, 100, 1), (100, 500, 1), (100, 99, 1), (100, 98, 2)];

    for (i, (start, damage, expected)) in cases.iter().enumerate() {
        let telegram_id = 1_001 + i as i64;
        let user = t.seed_user(telegram_id).await;
        t.seed_hand_pig(&user, *start, TODAY()).await;

        t.db.hand_pig
            .update_hrundel_duel(telegram_id, *damage, false)
            .await
            .unwrap();

        let (pig, _) =
            t.db.hand_pig.get_hrundel(telegram_id).await.unwrap().unwrap();
        assert_eq!(
            pig.weight, *expected,
            "{start} kg losing {damage} should land on {expected}"
        );
    }
}

#[tokio::test]
async fn a_duel_only_touches_the_pig_it_names() {
    let t = db!();

    let a = t.seed_user(1_001).await;
    let b = t.seed_user(1_002).await;
    t.seed_hand_pig(&a, 100, TODAY()).await;
    t.seed_hand_pig(&b, 100, TODAY()).await;

    t.db.hand_pig.update_hrundel_duel(1_001, 10, true).await.unwrap();

    let (untouched, _) = t.db.hand_pig.get_hrundel(1_002).await.unwrap().unwrap();
    assert_eq!(untouched.weight, 100);
    assert_eq!(untouched.win, 0);
}


#[tokio::test]
async fn an_inline_group_is_found_by_its_chat_instance() {
    let t = db!();

    let group = t.seed_inline_group(7_777_777).await;

    let by_instance =
        t.db.hand_pig.get_inline_group("7777777").await.unwrap().unwrap();
    let by_id =
        t.db.hand_pig.get_inline_group_by_id(group.id).await.unwrap().unwrap();

    assert_eq!(by_instance.id, group.id);
    assert_eq!(by_id.chat_instance, 7_777_777);
}

#[tokio::test]
async fn an_unparseable_chat_instance_matches_nothing() {
    // Every call site used to do `.parse::<i64>().unwrap_or(1)`, so any
    // non-numeric instance string silently shared inline group `1` with
    // every other unparseable instance bot-wide.
    let t = db!();
    let bucket = t.seed_inline_group(1).await;
    let user = t.seed_user(1_001).await;
    let pig = t.seed_hand_pig(&user, 100, TODAY()).await;
    t.link_hand_pig_to_inline_group(&pig, &bucket).await;

    for junk in ["not-a-number", "", "abc", "9999999999999999999999", " 1"] {
        assert!(
            t.db.hand_pig.get_inline_group(junk).await.unwrap().is_none(),
            "{junk:?} resolved to a group"
        );
        assert!(
            t.db.hand_pig
                .get_inline_users_with_user_by_chat(junk)
                .await
                .unwrap()
                .is_empty(),
            "{junk:?} listed members"
        );
        assert!(
            t.db.hand_pig.get_top10_chat(junk, TODAY()).await.unwrap().is_none(),
            "{junk:?} produced a leaderboard"
        );
        assert!(
            t.db.hand_pig
                .get_hryak_day_in_chat(junk, TODAY())
                .await
                .unwrap()
                .is_none(),
            "{junk:?} resolved a day pig"
        );
        assert!(
            t.db.hand_pig.get_group_user(junk, 1_001).await.unwrap().is_none(),
            "{junk:?} resolved a member"
        );
    }
    assert_eq!(
        t.db.hand_pig.get_inline_group("1").await.unwrap().map(|g| g.id),
        Some(bucket.id)
    );
}

#[tokio::test]
async fn an_inline_group_is_not_created_for_an_unparseable_instance() {
    let t = db!();

    let result = t
        .db
        .hand_pig
        .add_inline_group("not-a-number", datetime(2026, 1, 1, 0, 0))
        .await;

    assert!(result.is_err(), "a junk instance created a group");
    assert!(t.db.hand_pig.get_inline_group("1").await.unwrap().is_none());
}

#[tokio::test]
async fn linking_a_hand_pig_to_an_inline_group_is_idempotent() {
    let t = db!();

    let user = t.seed_user(1_001).await;
    let pig = t.seed_hand_pig(&user, 100, TODAY()).await;
    let group = t.seed_inline_group(7_777_777).await;

    let first = t.db.hand_pig.get_or_create_iug(pig.id, group.id).await.unwrap();
    let second = t.db.hand_pig.get_or_create_iug(pig.id, group.id).await.unwrap();

    assert_eq!(first.id, second.id, "get_or_create_iug must not duplicate");

    let members = t
        .db
        .hand_pig
        .get_inline_users_with_user_by_chat("7777777")
        .await
        .unwrap();
    assert_eq!(members.len(), 1);
}

#[tokio::test]
async fn a_duplicate_link_is_rejected_by_the_database() {
    // The unique index on `(iu_id, ig_id)` is what makes
    // `get_or_create_iug`'s select-then-insert race-safe.
    let t = db!();

    let user = t.seed_user(1_001).await;
    let pig = t.seed_hand_pig(&user, 100, TODAY()).await;
    let group = t.seed_inline_group(7_777_777).await;

    t.db.hand_pig.add_group_to_user(pig.id, group.id).await.unwrap();
    let second = t.db.hand_pig.add_group_to_user(pig.id, group.id).await;

    assert!(second.is_err(), "a duplicate link row was created");
    let members = t
        .db
        .hand_pig
        .get_inline_users_with_user_by_chat("7777777")
        .await
        .unwrap();
    assert_eq!(members.len(), 1);
}

#[tokio::test]
async fn concurrent_links_settle_on_a_single_row() {
    // What `get_or_create_iug` is actually for: two racing callers must end
    // up sharing one link, not two.
    let t = db!();

    let user = t.seed_user(1_001).await;
    let pig = t.seed_hand_pig(&user, 100, TODAY()).await;
    let group = t.seed_inline_group(7_777_777).await;

    let (a, b) = tokio::join!(
        t.db.hand_pig.get_or_create_iug(pig.id, group.id),
        t.db.hand_pig.get_or_create_iug(pig.id, group.id),
    );

    // Whichever ordering the two took, at most one row exists afterwards.
    if let (Ok(a), Ok(b)) = (&a, &b) {
        assert_eq!(a.id, b.id);
    }

    let members = t
        .db
        .hand_pig
        .get_inline_users_with_user_by_chat("7777777")
        .await
        .unwrap();
    assert_eq!(members.len(), 1, "a duplicate link survived");
}

#[tokio::test]
async fn the_same_pig_can_still_join_several_inline_groups() {
    // The index is on the pair, not on `iu_id` alone.
    let t = db!();

    let user = t.seed_user(1_001).await;
    let pig = t.seed_hand_pig(&user, 100, TODAY()).await;
    let one = t.seed_inline_group(111).await;
    let two = t.seed_inline_group(222).await;

    let a = t.db.hand_pig.get_or_create_iug(pig.id, one.id).await.unwrap();
    let b = t.db.hand_pig.get_or_create_iug(pig.id, two.id).await.unwrap();

    assert_ne!(a.id, b.id);
}

#[tokio::test]
async fn inline_group_membership_is_scoped_to_the_instance() {
    let t = db!();

    let a = t.seed_user(1_001).await;
    let b = t.seed_user(1_002).await;
    let pig_a = t.seed_hand_pig(&a, 100, TODAY()).await;
    let pig_b = t.seed_hand_pig(&b, 100, TODAY()).await;

    let one = t.seed_inline_group(111).await;
    let two = t.seed_inline_group(222).await;

    t.link_hand_pig_to_inline_group(&pig_a, &one).await;
    t.link_hand_pig_to_inline_group(&pig_b, &two).await;

    let in_one =
        t.db.hand_pig.get_inline_users_with_user_by_chat("111").await.unwrap();
    assert_eq!(in_one.len(), 1);
    assert_eq!(in_one[0].1.user_id, 1_001);
}


#[tokio::test]
async fn the_global_board_is_ordered_by_weight_and_capped_at_ten() {
    let t = db!();

    for i in 0..14i64 {
        let user = t.seed_user(1_001 + i).await;
        t.seed_hand_pig(&user, (14 - i) as i32 * 100, TODAY()).await;
    }

    let board = t.db.hand_pig.get_top10_global(TODAY()).await.unwrap().unwrap();

    assert_eq!(board.len(), 10);
    assert_eq!(board[0].weight, 1_400);
    assert!(board.windows(2).all(|w| w[0].weight >= w[1].weight));
}

#[tokio::test]
async fn the_weight_boards_silently_drop_pigs_not_touched_today() {
    // CHARACTERISATION. A heavy pig disappears from the board
    // on any day its owner has not used inline mode.
    let t = db!();

    let stale_owner = t.seed_user(1_001).await;
    let fresh_owner = t.seed_user(1_002).await;
    t.seed_hand_pig(&stale_owner, 9_999, date(2026, 7, 27)).await;
    t.seed_hand_pig(&fresh_owner, 10, TODAY()).await;

    let board = t.db.hand_pig.get_top10_global(TODAY()).await.unwrap().unwrap();

    assert_eq!(board.len(), 1);
    assert_eq!(board[0].weight, 10, "yesterday's 9999 kg pig is not listed");
}

#[tokio::test]
async fn an_empty_board_is_reported_as_none() {
    let t = db!();

    assert!(t.db.hand_pig.get_top10_global(TODAY()).await.unwrap().is_none());
    assert!(t.db.hand_pig.get_top10_chat("111", TODAY()).await.unwrap().is_none());
    assert!(t.db.hand_pig.get_top10_win().await.unwrap().is_none());
}

#[tokio::test]
async fn the_chat_board_only_lists_pigs_seen_in_that_chat() {
    let t = db!();

    let inside = t.seed_user(1_001).await;
    let outside = t.seed_user(1_002).await;
    let pig_in = t.seed_hand_pig(&inside, 500, TODAY()).await;
    t.seed_hand_pig(&outside, 9_000, TODAY()).await;

    let group = t.seed_inline_group(111).await;
    t.link_hand_pig_to_inline_group(&pig_in, &group).await;

    let board =
        t.db.hand_pig.get_top10_chat("111", TODAY()).await.unwrap().unwrap();

    assert_eq!(board.len(), 1);
    assert_eq!(board[0].weight, 500);
}

#[tokio::test]
async fn the_win_board_is_cumulative_and_ignores_the_date() {
    let t = db!();

    let veteran = t.seed_user(1_001).await;
    let rookie = t.seed_user(1_002).await;
    t.seed_hand_pig(&veteran, 10, date(2020, 1, 1)).await;
    t.seed_hand_pig(&rookie, 9_999, TODAY()).await;

    for _ in 0..3 {
        t.db.hand_pig.update_hrundel_duel(1_001, 1, true).await.unwrap();
    }

    let board = t.db.hand_pig.get_top10_win().await.unwrap().unwrap();

    assert_eq!(board.len(), 2);
    assert_eq!(board[0].win, 3, "the veteran leads on wins despite being light");
}


#[tokio::test]
async fn a_day_pig_is_recorded_once_per_chat_per_day() {
    let t = db!();

    let user = t.seed_user(1_001).await;
    let pig = t.seed_hand_pig(&user, 100, TODAY()).await;
    let group = t.seed_inline_group(111).await;
    let link = t.link_hand_pig_to_inline_group(&pig, &group).await;

    assert!(
        t.db.hand_pig.get_hryak_day_in_chat("111", TODAY()).await.unwrap().is_none()
    );

    t.db.hand_pig.add_hryak_day_to_chat(link.id, TODAY()).await.unwrap();

    let winner =
        t.db.hand_pig.get_hryak_day_in_chat("111", TODAY()).await.unwrap();
    let (_, day, _, owner) = winner.expect("no day pig recorded");

    assert_eq!(day.date, TODAY());
    assert_eq!(owner.user_id, 1_001);
    assert!(
        t.db.hand_pig
            .get_hryak_day_in_chat("111", date(2026, 7, 29))
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn the_day_pig_leaderboard_counts_wins_per_hand_pig() {
    let t = db!();

    let a = t.seed_user(1_001).await;
    let b = t.seed_user(1_002).await;
    let pig_a = t.seed_hand_pig(&a, 100, TODAY()).await;
    let pig_b = t.seed_hand_pig(&b, 100, TODAY()).await;
    let group = t.seed_inline_group(111).await;
    let link_a = t.link_hand_pig_to_inline_group(&pig_a, &group).await;
    let link_b = t.link_hand_pig_to_inline_group(&pig_b, &group).await;

    for day in 20..23u32 {
        t.db.hand_pig
            .add_hryak_day_to_chat(link_a.id, date(2026, 7, day))
            .await
            .unwrap();
    }
    t.db.hand_pig
        .add_hryak_day_to_chat(link_b.id, date(2026, 7, 25))
        .await
        .unwrap();

    let counts =
        t.db.hand_pig.get_day_pig_counts_by_chat(group.id).await.unwrap();

    assert_eq!(counts.len(), 2);
    assert_eq!(counts[0].2, 3, "the most frequent winner comes first");
    assert_eq!(counts[1].2, 1);
}
