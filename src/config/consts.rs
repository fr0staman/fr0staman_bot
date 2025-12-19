use std::sync::{Arc, LazyLock};

use ahash::{HashMap, HashSet};
use teloxide::types::ParseMode;
use tokio::sync::{Mutex, RwLock};

pub const BOT_PARSE_MODE: ParseMode = ParseMode::Html;
pub const DEFAULT_LANG_TAG: &str = "uk";
pub const TOP_LIMIT: i64 = 50;
pub const TOP_LIMIT_WITH_CHARTS: i64 = 30;

pub const INLINE_QUERY_LIMIT: usize = 50;

pub const SUBSCRIBE_GIFT: i32 = 100;
pub const DAILY_GIFT_AMOUNT: i32 = 500;

pub const CHAT_PIG_START_MASS: i32 = 1;
// I'm too lazy to do this properly
pub const IGNORED_COMMANDS: [&str; 4] = ["/lang", "/p", "/start", "/id"];
pub const LOUDER_DEFAULT_VOICE_LIMIT: u32 = 60;
pub const LOUDER_PREMIUM_VOICE_LIMIT: u32 = 1200;
pub const LOUDER_DEFAULT_RATIO: f32 = 2.0;
pub const INLINE_NAME_SET_LIMIT: usize = 20;
pub const INLINE_VOICE_REWARD_KG: i32 = 250;
pub const INLINE_GIF_REWARD_KG: i32 = 250;
pub const HAND_PIG_ADDITION_ON_SUPPORTED: i32 = 500;
pub const HAND_PIG_ADDITION_ON_SUBSCRIBED: i32 = 100;

#[allow(clippy::type_complexity)]
pub static DUEL_LOCKS: LazyLock<RwLock<HashMap<u64, Arc<Mutex<Vec<u64>>>>>> =
    LazyLock::new(|| RwLock::new(HashMap::default()));
pub static DUEL_LIST: LazyLock<RwLock<HashSet<u64>>> =
    LazyLock::new(|| RwLock::new(HashSet::default()));
