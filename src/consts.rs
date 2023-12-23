use std::sync::Arc;

use ahash::HashMap;
use dashmap::DashSet;
use once_cell::sync::Lazy;
use teloxide::types::ParseMode;
use tokio::sync::{Mutex, RwLock};

pub const BOT_PARSE_MODE: ParseMode = ParseMode::Html;
pub const DEFAULT_LANG_TAG: &str = "uk";
pub const TOP_LIMIT: i64 = 50;
pub const INLINE_QUERY_LIMIT: usize = 50;

pub const SUBSCRIBE_GIFT: i32 = 100;
pub const DAILY_GIFT_AMOUNT: i32 = 500;

pub const CHAT_PIG_START_MASS: i32 = 1;
// I'm too lazy to do this properly
pub const IGNORED_COMMANDS: [&str; 4] = ["/lang", "/p", "/start", "/id"];

#[allow(clippy::type_complexity)]
pub static DUEL_LOCKS: Lazy<RwLock<HashMap<u64, Arc<Mutex<Vec<u64>>>>>> =
    Lazy::new(|| RwLock::new(HashMap::default()));
pub static DUEL_LIST: Lazy<DashSet<u64>> = Lazy::new(DashSet::new);
