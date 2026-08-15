//! `DB.other` (users, chats, achievements, content) and `db::shortcuts`.
//!
//! Requires `TEST_DATABASE_URL`; see `src/tests/common.rs`.

use crate::tests::common::datetime;
use crate::config::consts::INLINE_CONTENT_APPROVED;
use crate::db::models::{
    AchievementUserAdd, UpdateGroups, UpdateUser, UserStatus,
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
async fn a_user_is_found_by_telegram_id_and_by_internal_id() {
    let t = db!();

    let user = t.seed_user(1_001).await;

    let by_telegram = t.db.other.get_user(1_001).await.unwrap().unwrap();
    let by_internal = t.db.other.get_user_by_id(user.id).await.unwrap().unwrap();

    assert_eq!(by_telegram.id, user.id);
    assert_eq!(by_internal.user_id, 1_001);
    assert!(t.db.other.get_user(999_999).await.unwrap().is_none());
}

#[tokio::test]
async fn a_user_status_change_keeps_the_other_flags() {
    let t = db!();

    t.seed_user(1_001).await;

    t.db.other
        .change_user_status(
            1_001,
            UserStatus {
                started: true,
                banned: true,
                supported: true,
                subscribed: false,
            },
        )
        .await
        .unwrap();

    let user = t.db.other.get_user(1_001).await.unwrap().unwrap();
    assert!(user.banned);
    assert!(user.supported);
    assert!(!user.subscribed);
}

#[tokio::test]
async fn a_language_override_can_be_set_and_cleared() {
    let t = db!();

    t.seed_user(1_001).await;
    assert!(t.db.other.get_user(1_001).await.unwrap().unwrap().lang.is_none());

    t.db.other.change_user_lang(1_001, Some("en")).await.unwrap();
    assert_eq!(
        t.db.other.get_user(1_001).await.unwrap().unwrap().lang.as_deref(),
        Some("en")
    );

    t.db.other.change_user_lang(1_001, None).await.unwrap();
    assert!(t.db.other.get_user(1_001).await.unwrap().unwrap().lang.is_none());
}

#[tokio::test]
async fn updating_a_user_rewrites_the_profile_fields() {
    let t = db!();

    let user = t.seed_user(1_001).await;

    t.db.other
        .update_user(
            1_001,
            UpdateUser {
                first_name: "Renamed".to_owned(),
                last_name: Some("Surname".to_owned()),
                username: Some("handle".to_owned()),
                ..user.to_update()
            },
        )
        .await
        .unwrap();

    let after = t.db.other.get_user(1_001).await.unwrap().unwrap();
    assert_eq!(after.first_name, "Renamed");
    assert_eq!(after.last_name.as_deref(), Some("Surname"));
    assert_eq!(after.username.as_deref(), Some("handle"));
}


#[tokio::test]
async fn a_chat_is_stored_with_its_defaults() {
    let t = db!();

    let group = t.seed_group(-100_001).await;

    let found = t.db.other.get_chat(-100_001).await.unwrap().unwrap();
    assert_eq!(found.id, group.id);
    assert!(found.active);
    assert_eq!(found.settings, 0);
    assert_eq!(found.top10_setting, 0);
    assert!(found.reset_at.is_none());
}

#[tokio::test]
async fn the_epyc_settings_are_persisted_independently() {
    let t = db!();

    t.seed_group(-100_001).await;

    t.db.other.set_chat_settings(-100_001, 1).await.unwrap();
    t.db.other.set_top10_setting(-100_001, 42).await.unwrap();

    let group = t.db.other.get_chat(-100_001).await.unwrap().unwrap();
    assert_eq!(group.settings, 1, "greetings disabled");
    assert_eq!(group.top10_setting, 42);
}

#[tokio::test]
async fn a_chat_migration_repoints_the_chat_id() {
    let t = db!();

    let group = t.seed_group(-100_001).await;

    t.db.other.update_chat_id(-100_001, -1_001_000_000_001).await.unwrap();

    assert!(t.db.other.get_chat(-100_001).await.unwrap().is_none());
    let migrated =
        t.db.other.get_chat(-1_001_000_000_001).await.unwrap().unwrap();
    assert_eq!(migrated.id, group.id);
}

#[tokio::test]
async fn a_chat_can_be_deactivated_and_reactivated() {
    let t = db!();

    let group = t.seed_group(-100_001).await;

    t.db.other
        .update_chat(-100_001, UpdateGroups { active: false, ..group.to_update() })
        .await
        .unwrap();
    assert!(!t.db.other.get_chat(-100_001).await.unwrap().unwrap().active);

    let group = t.db.other.get_chat(-100_001).await.unwrap().unwrap();
    t.db.other
        .update_chat(-100_001, UpdateGroups { active: true, ..group.to_update() })
        .await
        .unwrap();
    assert!(t.db.other.get_chat(-100_001).await.unwrap().unwrap().active);
}

#[tokio::test]
async fn a_chat_language_override_can_be_set_and_cleared() {
    // `!epyc мова <code>` and `!epyc мова -` both go through `update_chat`
    // with a `UpdateGroups { lang, ..to_update() }`. Clearing used to be a
    // silent no-op, because diesel skipped the `None`.
    let t = db!();

    let group = t.seed_group(-100_001).await;
    assert!(group.lang.is_none());

    t.db.other
        .update_chat(
            -100_001,
            UpdateGroups { lang: Some("en".to_owned()), ..group.to_update() },
        )
        .await
        .unwrap();
    let with = t.db.other.get_chat(-100_001).await.unwrap().unwrap();
    assert_eq!(with.lang.as_deref(), Some("en"));

    t.db.other
        .update_chat(-100_001, UpdateGroups { lang: None, ..with.to_update() })
        .await
        .unwrap();

    let cleared = t.db.other.get_chat(-100_001).await.unwrap().unwrap();
    assert!(cleared.lang.is_none(), "the chat language override survived");
    assert_eq!(cleared.title, "Test chat");
    assert!(cleared.active);
}

#[tokio::test]
async fn the_reset_timestamp_is_stamped_on_the_group() {
    let t = db!();

    let group = t.seed_group(-100_001).await;
    let now = datetime(2026, 7, 28, 15, 0);

    t.db.other.set_group_reset_at(group.id, now).await.unwrap();

    assert_eq!(
        t.db.other.get_chat(-100_001).await.unwrap().unwrap().reset_at,
        Some(now)
    );
}

#[tokio::test]
async fn listing_chats_and_users_returns_everything_seeded() {
    let t = db!();

    for i in 0..3i64 {
        t.seed_user(1_001 + i).await;
        t.seed_group(-100_001 - i).await;
    }

    assert_eq!(t.db.other.get_users().await.unwrap().len(), 3);
    assert_eq!(t.db.other.get_chats().await.unwrap().len(), 3);
}


#[tokio::test]
async fn achievements_are_stored_per_pig() {
    let t = db!();

    let user = t.seed_user(1_001).await;
    let group = t.seed_group(-100_001).await;
    let pig =
        t.seed_chat_pig(&user, &group, 100, crate::tests::common::date(2026, 7, 28)).await;

    t.db.other
        .add_achievements(&[
            AchievementUserAdd {
                game_id: pig.id,
                code: 203,
                created_at: datetime(2026, 7, 28, 12, 0),
            },
            AchievementUserAdd {
                game_id: pig.id,
                code: 102,
                created_at: datetime(2026, 7, 28, 12, 0),
            },
        ])
        .await
        .unwrap();

    let mut codes: Vec<i16> = t
        .db
        .other
        .get_achievements_by_game_id(pig.id)
        .await
        .unwrap()
        .iter()
        .map(|a| a.code)
        .collect();
    codes.sort_unstable();

    assert_eq!(codes, vec![102, 203]);
}

#[tokio::test]
async fn adding_an_empty_achievement_batch_is_a_no_op() {
    let t = db!();

    t.db.other.add_achievements(&[]).await.unwrap();
}

#[tokio::test]
async fn the_notice_counts_split_this_chat_from_the_global_unique_set() {
    let t = db!();

    let user = t.seed_user(1_001).await;
    let here = t.seed_group(-100_001).await;
    let elsewhere = t.seed_group(-100_002).await;
    let pig_here =
        t.seed_chat_pig(&user, &here, 100, crate::tests::common::date(2026, 7, 28)).await;
    let pig_there =
        t.seed_chat_pig(&user, &elsewhere, 100, crate::tests::common::date(2026, 7, 28)).await;

    let at = datetime(2026, 7, 28, 12, 0);

    // Two here, and one of them plus a new one over there.
    t.db.other
        .add_achievements(&[
            AchievementUserAdd { game_id: pig_here.id, code: 203, created_at: at },
            AchievementUserAdd { game_id: pig_here.id, code: 102, created_at: at },
            AchievementUserAdd { game_id: pig_there.id, code: 203, created_at: at },
            AchievementUserAdd { game_id: pig_there.id, code: 305, created_at: at },
        ])
        .await
        .unwrap();

    let (in_this_chat, global_unique) = t
        .db
        .other
        .count_achievements_for_notice(pig_here.id, user.id)
        .await
        .unwrap();

    assert_eq!(in_this_chat, 2);
    assert_eq!(global_unique, 3, "203 is counted once across both chats");
}

#[tokio::test]
async fn achievements_by_uid_span_every_chat() {
    let t = db!();

    let user = t.seed_user(1_001).await;
    let other = t.seed_user(1_002).await;
    let group = t.seed_group(-100_001).await;
    let mine =
        t.seed_chat_pig(&user, &group, 100, crate::tests::common::date(2026, 7, 28)).await;
    let theirs =
        t.seed_chat_pig(&other, &group, 100, crate::tests::common::date(2026, 7, 28)).await;

    let at = datetime(2026, 7, 28, 12, 0);
    t.db.other
        .add_achievements(&[
            AchievementUserAdd { game_id: mine.id, code: 203, created_at: at },
            AchievementUserAdd { game_id: theirs.id, code: 102, created_at: at },
        ])
        .await
        .unwrap();

    let mine_only = t.db.other.get_achievements_by_uid(user.id).await.unwrap();

    assert_eq!(mine_only.len(), 1);
}


#[tokio::test]
async fn an_approved_voice_is_stored_and_immediately_browsable() {
    // Content rows are only written once the creator has approved them, so
    // they go in already approved. `status` and `caption` are NOT NULL with
    // no column default and must both be supplied.
    let t = db!();

    let user = t.seed_user(1_001).await;

    t.db.other.add_voice(user.id, "clip.ogg".to_owned()).await.unwrap();

    let mine = t.db.other.get_voices_by_user(user.id).await.unwrap();
    assert_eq!(mine.len(), 1);
    assert_eq!(mine[0].url, "clip.ogg");
    assert_eq!(mine[0].status, INLINE_CONTENT_APPROVED);
    assert_eq!(mine[0].caption, "");

    // The inline browser filters on `status = 1`, so it is listed.
    let browsable = t.db.other.get_inline_voices().await.unwrap();
    assert_eq!(browsable.len(), 1);
    assert_eq!(browsable[0].id, mine[0].id);

    assert_eq!(
        t.db.other.get_inline_voice_by_id(mine[0].id).await.unwrap().map(|v| v.url),
        Some("clip.ogg".to_owned())
    );
}

#[tokio::test]
async fn an_approved_gif_is_stored_and_immediately_browsable() {
    let t = db!();

    let user = t.seed_user(1_001).await;

    t.db.other
        .add_gif(user.id, "file-id-1".to_owned(), "unique-1".to_owned())
        .await
        .unwrap();

    let mine = t.db.other.get_gifs_by_user(user.id).await.unwrap();
    assert_eq!(mine.len(), 1);
    assert_eq!(mine[0].file_id, "file-id-1");
    assert_eq!(mine[0].status, INLINE_CONTENT_APPROVED);

    let browsable = t.db.other.get_inline_gifs().await.unwrap();
    assert_eq!(browsable.len(), 1);

    assert!(t.db.other.get_inline_gif_by_id(mine[0].id).await.unwrap().is_some());
}

#[tokio::test]
async fn the_id_reported_back_to_the_submitter_is_the_row_just_inserted() {
    // Both approval handlers announce the assigned number with
    // `get_*_by_user(..).last()`, so the newest row must come last. The
    // queries order by id explicitly rather than trusting physical order.
    let t = db!();

    let user = t.seed_user(1_001).await;

    for i in 0..5 {
        t.db.other.add_voice(user.id, format!("clip{i}.ogg")).await.unwrap();
        t.db.other
            .add_gif(user.id, format!("file{i}"), format!("unique{i}"))
            .await
            .unwrap();
    }

    let voices = t.db.other.get_voices_by_user(user.id).await.unwrap();
    assert_eq!(voices.len(), 5);
    assert_eq!(voices.last().unwrap().url, "clip4.ogg");

    let gifs = t.db.other.get_gifs_by_user(user.id).await.unwrap();
    assert_eq!(gifs.last().unwrap().file_id, "file4");
    assert!(voices.windows(2).all(|w| w[0].id < w[1].id));
    assert!(gifs.windows(2).all(|w| w[0].id < w[1].id));
}

#[tokio::test]
async fn the_submitter_listing_survives_rows_being_rewritten() {
    // An UPDATE can move a row's physical position in the heap, which is
    // exactly when an unordered query starts returning a different order.
    let t = db!();

    let user = t.seed_user(1_001).await;

    for i in 0..4 {
        t.db.other.add_voice(user.id, format!("clip{i}.ogg")).await.unwrap();
    }

    // Rewrite an early row so it is relocated to the end of the heap.
    // Scoped import: `RunQueryDsl` at file scope would shadow slice methods
    // like `first()` used further down.
    use diesel_async::RunQueryDsl as _;

    let mut conn = t.conn().await;
    diesel::sql_query(
        "UPDATE inline_voices SET url = 'clip0-rewritten.ogg' \
         WHERE url = 'clip0.ogg'",
    )
    .execute(&mut conn)
    .await
    .unwrap();
    drop(conn);

    let voices = t.db.other.get_voices_by_user(user.id).await.unwrap();

    // Indexing rather than `.first()`: `RunQueryDsl` is in scope here and
    // would resolve `first` to diesel's.
    assert_eq!(voices.len(), 4);
    assert_eq!(voices[0].url, "clip0-rewritten.ogg");
    assert_eq!(voices[3].url, "clip3.ogg");
    assert!(voices.windows(2).all(|w| w[0].id < w[1].id));
}

#[tokio::test]
async fn browse_listings_are_empty_before_anything_is_approved() {
    let t = db!();

    assert!(t.db.other.get_inline_voices().await.unwrap().is_empty());
    assert!(t.db.other.get_inline_gifs().await.unwrap().is_empty());
    assert!(t.db.other.get_inline_voice_by_id(1).await.unwrap().is_none());
    assert!(t.db.other.get_inline_gif_by_id(1).await.unwrap().is_none());
}

#[tokio::test]
async fn a_gif_is_deduplicated_by_its_file_unique_id() {
    // The submission handler rejects a re-upload by looking the unique id up
    // before queueing it
    let t = db!();

    let user = t.seed_user(1_001).await;

    t.db.other
        .add_gif(user.id, "file-id-1".to_owned(), "unique-1".to_owned())
        .await
        .unwrap();

    let existing =
        t.db.other.get_gif_by_file_unique_id("unique-1").await.unwrap();
    assert_eq!(existing.map(|g| g.file_id), Some("file-id-1".to_owned()));

    assert!(
        t.db.other.get_gif_by_file_unique_id("unique-2").await.unwrap().is_none()
    );
}


mod shortcuts {
    use super::*;
    use crate::db::shortcuts;
    use teloxide::types::{Chat, User as TelegramUser};

    fn telegram_user(
        id: u64,
        first: &str,
        last: Option<&str>,
        username: Option<&str>,
    ) -> TelegramUser {
        let last = last
            .map(|l| format!(r#""last_name": "{l}","#))
            .unwrap_or_default();
        let username = username
            .map(|u| format!(r#""username": "{u}","#))
            .unwrap_or_default();

        let json = format!(
            r#"{{
                "id": {id},
                "is_bot": false,
                {last}
                {username}
                "first_name": "{first}"
            }}"#
        );

        serde_json::from_str(&json).expect("bad User fixture")
    }

    fn telegram_chat(id: i64, title: &str, username: Option<&str>) -> Chat {
        let username = username
            .map(|u| format!(r#""username": "{u}","#))
            .unwrap_or_default();

        let json = format!(
            r#"{{
                "id": {id},
                "type": "supergroup",
                {username}
                "title": "{title}"
            }}"#
        );

        serde_json::from_str(&json).expect("bad Chat fixture")
    }

    #[tokio::test]
    async fn an_unknown_user_is_inserted() {
        let t = db!();

        let from = telegram_user(1_001, "Tester", None, None);
        let inserted = shortcuts::maybe_get_or_insert_user(&from, false)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(inserted.user_id, 1_001);
        assert!(!inserted.started);
        assert!(t.db.other.get_user(1_001).await.unwrap().is_some());
    }

    #[tokio::test]
    async fn an_unchanged_user_is_returned_without_an_update() {
        let t = db!();

        let from = telegram_user(1_001, "Tester", None, None);
        shortcuts::maybe_get_or_insert_user(&from, false).await.unwrap();

        // Mark the row so an UPDATE would be visible.
        t.db.other
            .change_user_status(
                1_001,
                UserStatus {
                    started: true,
                    banned: false,
                    supported: true,
                    subscribed: false,
                },
            )
            .await
            .unwrap();

        let again = shortcuts::maybe_get_or_insert_user(&from, false)
            .await
            .unwrap()
            .unwrap();

        assert!(again.supported, "the row must not have been rewritten");
    }

    #[tokio::test]
    async fn a_renamed_user_is_updated_and_the_new_name_is_returned() {
        let t = db!();

        let before = telegram_user(1_001, "Old", None, None);
        shortcuts::maybe_get_or_insert_user(&before, false).await.unwrap();

        let after = telegram_user(1_001, "New", Some("Surname"), Some("handle"));
        let returned = shortcuts::maybe_get_or_insert_user(&after, false)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(returned.first_name, "New");
        assert_eq!(returned.last_name.as_deref(), Some("Surname"));
        assert_eq!(returned.username.as_deref(), Some("handle"));

        let stored = t.db.other.get_user(1_001).await.unwrap().unwrap();
        assert_eq!(stored.first_name, "New");
        assert_eq!(stored.username.as_deref(), Some("handle"));
    }

    #[tokio::test]
    async fn a_dropped_username_and_last_name_are_cleared() {
        // `UpdateUser` sets `treat_none_as_null`, so a `None` field writes
        // NULL instead of being skipped. Without it the stale handle would
        // survive every future update.
        let t = db!();

        let with =
            telegram_user(1_001, "Tester", Some("Surname"), Some("handle"));
        shortcuts::maybe_get_or_insert_user(&with, false).await.unwrap();

        let without = telegram_user(1_001, "Tester", None, None);
        let returned = shortcuts::maybe_get_or_insert_user(&without, false)
            .await
            .unwrap()
            .unwrap();

        assert!(returned.username.is_none());
        assert!(returned.last_name.is_none());

        let stored = t.db.other.get_user(1_001).await.unwrap().unwrap();
        assert!(stored.username.is_none(), "the stale username survived");
        assert!(stored.last_name.is_none(), "the stale last name survived");
    }

    #[tokio::test]
    async fn setting_a_username_does_persist() {
        let t = db!();

        let without = telegram_user(1_001, "Tester", None, None);
        shortcuts::maybe_get_or_insert_user(&without, false).await.unwrap();

        let with = telegram_user(1_001, "Tester", None, Some("handle"));
        shortcuts::maybe_get_or_insert_user(&with, false).await.unwrap();

        let stored = t.db.other.get_user(1_001).await.unwrap().unwrap();
        assert_eq!(stored.username.as_deref(), Some("handle"));
    }

    #[tokio::test]
    async fn a_profile_update_does_not_clobber_unrelated_columns() {
        // Every changeset is built from `..row.to_update()`, so writing NULL
        // for the absent fields must not disturb the rest of the row.
        let t = db!();

        let with = telegram_user(1_001, "Tester", None, Some("handle"));
        shortcuts::maybe_get_or_insert_user(&with, false).await.unwrap();

        t.db.other.change_user_lang(1_001, Some("en")).await.unwrap();
        t.db.other
            .change_user_status(
                1_001,
                UserStatus {
                    started: true,
                    banned: false,
                    supported: true,
                    subscribed: true,
                },
            )
            .await
            .unwrap();
        let without = telegram_user(1_001, "Tester", None, None);
        shortcuts::maybe_get_or_insert_user(&without, false).await.unwrap();

        let stored = t.db.other.get_user(1_001).await.unwrap().unwrap();
        assert!(stored.username.is_none());
        assert_eq!(stored.lang.as_deref(), Some("en"), "lang was clobbered");
        assert!(stored.supported, "supported was clobbered");
        assert!(stored.subscribed, "subscribed was clobbered");
    }

    #[tokio::test]
    async fn a_dropped_chat_username_is_cleared() {
        let t = db!();

        let with = telegram_chat(-100_001, "Test chat", Some("handle"));
        shortcuts::maybe_get_or_insert_chat(&with).await.unwrap();

        let without = telegram_chat(-100_001, "Test chat", None);
        shortcuts::maybe_get_or_insert_chat(&without).await.unwrap();

        let stored = t.db.other.get_chat(-100_001).await.unwrap().unwrap();
        assert!(stored.username.is_none());
    }

    #[tokio::test]
    async fn a_chat_update_preserves_the_inline_group_link_and_reset_stamp() {
        // `groups.ig_id` and `groups.reset_at` are `Option` columns written
        // by their own setters; a title change must carry them through.
        let t = db!();

        let chat = telegram_chat(-100_001, "Old title", None);
        let group =
            shortcuts::maybe_get_or_insert_chat(&chat).await.unwrap().unwrap();

        let inline_group = t.seed_inline_group(111).await;
        t.db.other
            .update_chat_ig_id(-100_001, Some(inline_group.id))
            .await
            .unwrap();
        let stamped = datetime(2026, 7, 28, 12, 0);
        t.db.other.set_group_reset_at(group.id, stamped).await.unwrap();

        let renamed = telegram_chat(-100_001, "New title", None);
        shortcuts::maybe_get_or_insert_chat(&renamed).await.unwrap();

        let stored = t.db.other.get_chat(-100_001).await.unwrap().unwrap();
        assert_eq!(stored.title, "New title");
        assert_eq!(stored.ig_id, Some(inline_group.id), "ig_id was cleared");
        assert_eq!(stored.reset_at, Some(stamped), "reset_at was cleared");
    }

    #[tokio::test]
    async fn an_unknown_chat_is_inserted() {
        let t = db!();

        let chat = telegram_chat(-100_001, "Test chat", Some("testchat"));
        let inserted =
            shortcuts::maybe_get_or_insert_chat(&chat).await.unwrap().unwrap();

        assert_eq!(inserted.chat_id, -100_001);
        assert!(inserted.active);
        assert!(t.db.other.get_chat(-100_001).await.unwrap().is_some());
    }

    #[tokio::test]
    async fn a_retitled_chat_is_updated() {
        let t = db!();

        let before = telegram_chat(-100_001, "Old title", None);
        shortcuts::maybe_get_or_insert_chat(&before).await.unwrap();

        let after = telegram_chat(-100_001, "New title", Some("newhandle"));
        let returned =
            shortcuts::maybe_get_or_insert_chat(&after).await.unwrap().unwrap();

        assert_eq!(returned.title, "New title");
        assert_eq!(returned.username.as_deref(), Some("newhandle"));

        let stored = t.db.other.get_chat(-100_001).await.unwrap().unwrap();
        assert_eq!(stored.title, "New title");
    }

    #[tokio::test]
    async fn seeing_a_deactivated_chat_again_reactivates_it() {
        let t = db!();

        let chat = telegram_chat(-100_001, "Test chat", None);
        let group =
            shortcuts::maybe_get_or_insert_chat(&chat).await.unwrap().unwrap();
        t.db.other
            .update_chat(
                -100_001,
                UpdateGroups { active: false, ..group.to_update() },
            )
            .await
            .unwrap();
        let returned =
            shortcuts::maybe_get_or_insert_chat(&chat).await.unwrap().unwrap();

        assert!(returned.active);
        assert!(t.db.other.get_chat(-100_001).await.unwrap().unwrap().active);
    }

    #[tokio::test]
    async fn an_unchanged_active_chat_is_left_alone() {
        let t = db!();

        let chat = telegram_chat(-100_001, "Test chat", None);
        shortcuts::maybe_get_or_insert_chat(&chat).await.unwrap();

        t.db.other.set_top10_setting(-100_001, 42).await.unwrap();

        shortcuts::maybe_get_or_insert_chat(&chat).await.unwrap();

        let stored = t.db.other.get_chat(-100_001).await.unwrap().unwrap();
        assert_eq!(stored.top10_setting, 42, "the row must not be rewritten");
    }
}
