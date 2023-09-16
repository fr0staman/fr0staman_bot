use dashmap::{DashMap, DashSet};
use once_cell::sync::Lazy;
use teloxide::types::ParseMode;
use tokio::sync::Mutex;

pub const BOT_PARSE_MODE: ParseMode = ParseMode::Html;
pub const DEFAULT_LANG_TAG: &str = "ru";
pub const TOP_LIMIT: i64 = 50;
pub const INLINE_QUERY_LIMIT: usize = 50;

pub const SUBSCRIBE_GIFT: i32 = 100;
pub const DAILY_GIFT_AMOUNT: i32 = 500;

pub const CHAT_PIG_START_MASS: i32 = 1;

pub static DUEL_LOCKS: Lazy<DashMap<u64, Mutex<Vec<u64>>>> =
    Lazy::new(DashMap::new);
pub static DUEL_LIST: Lazy<DashSet<u64>> = Lazy::new(DashSet::new);
