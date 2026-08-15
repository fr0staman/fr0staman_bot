//! Fixtures and one-time global init shared by the test modules.

use std::path::Path;

use chrono::{NaiveDate, NaiveDateTime};
use teloxide::types::Me;

use crate::{
    config::env::BOT_ME,
    db::models::{Game, GrowLog, InlineUser, User},
    lang::{LANG, Locale, LocaleTag},
};


/// `get_or_init`, not `setup_lang`'s `set().expect()` — tests share one
/// process.
pub fn init_lang() -> LocaleTag {
    LANG.get_or_init(|| {
        Locale::load_from(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("locales"),
            crate::config::consts::DEFAULT_LANG_TAG,
        )
    });

    crate::lang::tag(crate::config::consts::DEFAULT_LANG_TAG)
}

const ME_JSON: &str = r#"{
    "id": 1234567890,
    "is_bot": true,
    "first_name": "fr0staman test bot",
    "username": "fr0staman_bot",
    "can_join_groups": true,
    "can_read_all_group_messages": false,
    "supports_inline_queries": true,
    "can_connect_to_business": false,
    "has_main_web_app": false
}"#;

/// `BOT_ME` from a fixture instead of a live `get_me()`.
pub fn init_bot_me() -> &'static Me {
    BOT_ME.get_or_init(|| {
        serde_json::from_str(ME_JSON).expect("bad Me fixture")
    })
}

/// Placeholders for everything `BOT_CONFIG` reads, so it can be touched
/// without a `.env`. Existing vars win. Call before the first access.
pub fn init_env() {
    use std::sync::Once;
    static ONCE: Once = Once::new();

    ONCE.call_once(|| {
        let defaults = [
            ("TELOXIDE_TOKEN", "0000000000:TEST_TOKEN"),
            ("WEBHOOK_URL", "https://example.invalid/"),
            ("WEBHOOK_PORT", "8080"),
            ("DATABASE_URL", "postgres://test@localhost/test"),
            ("PHOTOSTOCK_URL", "https://example.invalid/photos/"),
            ("CHANNEL_ID", "-1001000000000"),
            ("CHANNEL_NAME", "test_channel"),
            ("CONTENT_CHECK_CHANNEL_ID", "-1001000000001"),
            ("CONTENT_CHECK_CHANNEL_NAME", "test_content"),
            ("CREATOR_ID", "1"),
            ("PROMETHEUS_TOKEN", "test_token"),
            ("GIF_CONTENT_CHANNEL_ID", "-1001000000002"),
            ("CHAT_LINK", "test_chat"),
            ("LOG_GROUP_ID", "-1001000000003"),
        ];

        for (key, value) in defaults {
            if std::env::var_os(key).is_none() {
                // SAFETY: inside a `Once`, before any thread reads the env.
                unsafe { std::env::set_var(key, value) };
            }
        }
    });
}

/// For tests that render text.
pub fn init_all() -> LocaleTag {
    init_env();
    init_bot_me();
    init_lang()
}


pub fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).expect("bad test date")
}

pub fn datetime(y: i32, m: u32, d: u32, h: u32, min: u32) -> NaiveDateTime {
    date(y, m, d).and_hms_opt(h, min, 0).expect("bad test time")
}


pub fn game(mass: i32) -> Game {
    Game {
        id: 1,
        uid: 1,
        group_id: 1,
        mass,
        date: date(2026, 7, 28),
        name: "Test pig".to_owned(),
    }
}

pub fn grow_log(
    created_at: NaiveDateTime,
    weight_change: i32,
    current_weight: i32,
) -> GrowLog {
    GrowLog { game_id: 1, created_at, weight_change, current_weight }
}

/// Consecutive daily feeds ending at `last_day`, deltas oldest-first.
pub fn daily_grow_log(
    last_day: NaiveDateTime,
    start_mass: i32,
    deltas: &[i32],
) -> Vec<GrowLog> {
    let mut mass = start_mass;
    let n = deltas.len() as i64;

    deltas
        .iter()
        .enumerate()
        .map(|(i, &delta)| {
            mass += delta;
            let day = last_day - chrono::Duration::days(n - 1 - i as i64);
            grow_log(day, delta, mass)
        })
        .collect()
}

pub fn user(id: i32, user_id: i64) -> User {
    User {
        id,
        user_id,
        started: true,
        banned: false,
        supported: false,
        subscribed: false,
        created_at: datetime(2026, 1, 1, 0, 0),
        lang: None,
        username: None,
        first_name: "Tester".to_owned(),
        last_name: None,
    }
}

pub fn inline_user(id: i32, uid: i32, weight: i32) -> InlineUser {
    InlineUser {
        id,
        uid,
        weight,
        date: date(2026, 7, 28),
        flag: "uk".to_owned(),
        win: 0,
        rout: 0,
        name: "Hand pig".to_owned(),
        gifted: false,
    }
}
