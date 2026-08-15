//! Harness for the Postgres tests. Opt-in: without `TEST_DATABASE_URL` they
//! return early, so `cargo test` works with no database. Setup is in
//! `scripts/setup_test_db.sh`.
//!
//! Isolation is TRUNCATE under a process-wide lock, not a rolled-back
//! transaction: every `DB.*` call takes a fresh connection, so
//! `begin_test_transaction` would be invisible to the next one.

#![allow(dead_code)]

use std::sync::Arc;

use diesel_async::RunQueryDsl;
use crate::{
    db::{
        init::{DBScheme, Database},
        models::{
            Game, Groups, GrowLogAdd, InlineGroup, InlineUser, InlineUsersGroup,
            NewGroup, NewInlineUser, NewUser, User,
        },
    },
    types::DbPool,
};
use tokio::sync::{Mutex, MutexGuard, OnceCell};

use chrono::{NaiveDate, NaiveDateTime};

/// One statement so `CASCADE` sorts out the foreign keys.
const TRUNCATE_ALL: &str = "TRUNCATE \
    achievements_users, \
    game, \
    groups, \
    grow_log, \
    hryak_day, \
    inline_gifs, \
    inline_groups, \
    inline_users, \
    inline_users_groups, \
    inline_voices, \
    users \
    RESTART IDENTITY CASCADE";

static LOCK: Mutex<()> = Mutex::const_new(());
static POOL: OnceCell<Option<Arc<DbPool>>> = OnceCell::const_new();

fn test_database_url() -> Option<String> {
    let _ = dotenvy::dotenv();
    std::env::var("TEST_DATABASE_URL").ok().filter(|u| !u.is_empty())
}

async fn pool() -> Option<Arc<DbPool>> {
    POOL.get_or_init(|| async {
        let url = test_database_url()?;

        // `db::shortcuts` and `services::*` reach for the global `DB`, built
        // from `BOT_CONFIG.database_url`. Redirect before anything reads it.
        //
        // SAFETY: once, inside a `OnceCell`, before the first DB access.
        unsafe { std::env::set_var("DATABASE_URL", &url) };
        crate::test_support::init_env();

        assert_eq!(
            crate::config::env::BOT_CONFIG.database_url.as_str(),
            url,
            "BOT_CONFIG was initialised before the test harness could \
             redirect it — a test touched the database before calling \
             `test_db()`"
        );

        // One pool for both the global `DB` and each test's `DBScheme`.
        Some(Database::get_or_init_pool())
    })
    .await
    .clone()
}

async fn pool_handle() -> Option<Arc<DbPool>> {
    pool().await
}

/// Holds the process-wide test lock for the lifetime of one test.
pub struct TestDb {
    pub db: DBScheme,
    _guard: MutexGuard<'static, ()>,
}

/// An empty test database, or `None` when `TEST_DATABASE_URL` is unset.
/// Must be the first line of a test — see the module docs on the redirect.
pub async fn test_db() -> Option<TestDb> {
    let Some(pool) = pool().await else {
        eprintln!(
            "skipping: set TEST_DATABASE_URL to run the database tests"
        );
        return None;
    };

    // A nested `test_db()` would deadlock; fail fast instead.
    let guard = tokio::time::timeout(
        std::time::Duration::from_secs(60),
        LOCK.lock(),
    )
    .await
    .expect(
        "timed out waiting for the test lock — is `test_db()` called twice \
         in one test?",
    );

    let mut conn = pool
        .get()
        .await
        .expect("could not reach TEST_DATABASE_URL — is it migrated?");
    diesel::sql_query(TRUNCATE_ALL)
        .execute(&mut conn)
        .await
        .expect("TRUNCATE failed — did you run scripts/setup_test_db.sh?");
    drop(conn);

    Some(TestDb { db: DBScheme::new(pool), _guard: guard })
}


pub fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

pub fn datetime(y: i32, m: u32, d: u32, h: u32, min: u32) -> NaiveDateTime {
    date(y, m, d).and_hms_opt(h, min, 0).unwrap()
}

impl TestDb {
    /// For tests needing SQL the API layer does not expose.
    pub async fn conn(
        &self,
    ) -> diesel_async::pooled_connection::deadpool::Object<
        crate::types::DbConn,
    > {
        pool_handle().await.expect("no test pool").get().await.expect("no conn")
    }

    pub async fn seed_user(&self, telegram_id: i64) -> User {
        self.db
            .other
            .register_user(NewUser {
                user_id: telegram_id,
                started: true,
                banned: false,
                supported: false,
                subscribed: false,
                created_at: datetime(2026, 1, 1, 0, 0),
                lang: None,
                username: None,
                first_name: "Tester",
                last_name: None,
            })
            .await
            .expect("seed_user")
    }

    pub async fn seed_group(&self, chat_id: i64) -> Groups {
        self.db
            .other
            .add_chat(NewGroup {
                chat_id,
                date: datetime(2026, 1, 1, 0, 0),
                settings: 0,
                top10_setting: 0,
                lang: None,
                active: true,
                ig_id: None,
                username: None,
                title: "Test chat",
            })
            .await
            .expect("seed_group")
    }

    pub async fn seed_chat_pig(
        &self,
        user: &User,
        group: &Groups,
        mass: i32,
        fed_on: NaiveDate,
    ) -> Game {
        self.db
            .chat_pig
            .create_chat_pig(user.id, group.id, "Test pig", fed_on, mass)
            .await
            .expect("seed_chat_pig")
    }

    pub async fn seed_grow_log(
        &self,
        game_id: i32,
        created_at: NaiveDateTime,
        weight_change: i32,
        current_weight: i32,
    ) {
        self.db
            .chat_pig
            .add_grow_log_by_game(GrowLogAdd {
                game_id,
                created_at,
                weight_change,
                current_weight,
            })
            .await
            .expect("seed_grow_log");
    }

    pub async fn seed_hand_pig(
        &self,
        user: &User,
        weight: i32,
        on: NaiveDate,
    ) -> InlineUser {
        self.db
            .hand_pig
            .add_hrundel(NewInlineUser {
                uid: user.id,
                weight,
                date: on,
                flag: "uk",
                win: 0,
                rout: 0,
                name: "Hand pig",
                gifted: false,
            })
            .await
            .expect("seed_hand_pig")
    }

    pub async fn seed_inline_group(&self, chat_instance: i64) -> InlineGroup {
        let instance = chat_instance.to_string();

        self.db
            .hand_pig
            .add_inline_group(&instance, datetime(2026, 1, 1, 0, 0))
            .await
            .expect("seed_inline_group");

        self.db
            .hand_pig
            .get_inline_group(&instance)
            .await
            .expect("seed_inline_group lookup")
            .expect("inline group missing after insert")
    }

    pub async fn link_hand_pig_to_inline_group(
        &self,
        hand_pig: &InlineUser,
        inline_group: &InlineGroup,
    ) -> InlineUsersGroup {
        self.db
            .hand_pig
            .get_or_create_iug(hand_pig.id, inline_group.id)
            .await
            .expect("link_hand_pig_to_inline_group")
    }

    /// A group with enough pigs to count as "active" for the social
    /// achievements.
    pub async fn seed_group_with_pigs(
        &self,
        chat_id: i64,
        owner: &User,
        others: i32,
    ) -> (Groups, Game) {
        let group = self.seed_group(chat_id).await;
        let pig = self.seed_chat_pig(owner, &group, 10, date(2026, 7, 28)).await;

        for i in 0..others {
            let filler =
                self.seed_user(chat_id.abs() * 1_000 + i as i64 + 1).await;
            self.seed_chat_pig(&filler, &group, 10, date(2026, 7, 28)).await;
        }

        (group, pig)
    }
}
