use std::sync::{Arc, LazyLock, OnceLock};

use crate::config::env::BOT_CONFIG;
use crate::db::api::{chat_pig::ChatPig, hand_pig::HandPig, other::Other};
use crate::types::{DbConn, DbPool};

use diesel_async::pooled_connection::{
    AsyncDieselConnectionManager, deadpool::Pool,
};

/// Tests build their own [`DBScheme`] instead, so they never touch this
/// static — nor [`BOT_CONFIG`] through it.
pub static DB: LazyLock<DBScheme> =
    LazyLock::new(|| DBScheme::new(Database::get_or_init_pool()));

pub struct DBScheme {
    pub hand_pig: HandPig,
    pub chat_pig: ChatPig,
    pub other: Other,
}

impl DBScheme {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self {
            hand_pig: HandPig::new(pool.clone()),
            chat_pig: ChatPig::new(pool.clone()),
            other: Other::new(pool),
        }
    }
}

pub struct Database;

impl Database {
    pub fn get_or_init_pool() -> Arc<DbPool> {
        static POOL: OnceLock<Arc<DbPool>> = OnceLock::new();

        POOL.get_or_init(|| {
            Arc::new(Self::build_pool(BOT_CONFIG.database_url.as_str()))
        })
        .clone()
    }

    pub fn build_pool(database_url: &str) -> DbPool {
        Pool::builder(Self::get_config(database_url))
            .build()
            .expect("Something wrong with Pool manager!")
    }

    pub fn get_config(
        database_url: &str,
    ) -> AsyncDieselConnectionManager<DbConn> {
        AsyncDieselConnectionManager::<DbConn>::new(database_url)
    }
}
