use std::sync::Arc;

use ahash::{AHashSet, HashMap, HashSet};
use teloxide::types::{ParseMode, UserId};
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

pub struct ResetVoteState {
    pub initiator_id: UserId,
    pub yes_votes: AHashSet<u64>,
    pub total_players: i64,
    pub quorum: i64,
    pub completed: bool,
}

pub struct GameState {
    #[allow(clippy::type_complexity)]
    pub duel_locks: RwLock<HashMap<u64, Arc<Mutex<Vec<u64>>>>>,
    pub duel_list: RwLock<HashSet<u64>>,
    pub reset_votes: RwLock<HashMap<i64, Arc<Mutex<ResetVoteState>>>>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            duel_locks: RwLock::new(HashMap::default()),
            duel_list: RwLock::new(HashSet::default()),
            reset_votes: RwLock::new(HashMap::default()),
        }
    }
}

