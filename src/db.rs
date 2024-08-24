use std::sync::OnceLock;

use diesel_async::pooled_connection::{
    deadpool::Pool, AsyncDieselConnectionManager,
};
use std::sync::LazyLock;

use crate::config::BOT_CONFIG;
use crate::db_api::{chat_pig::ChatPig, hand_pig::HandPig, other::Other};

pub type DbConn = diesel_async::AsyncMysqlConnection;
pub type MyPool = Pool<DbConn>;

pub static DB: LazyLock<DBScheme> = LazyLock::new(|| DBScheme {
    hand_pig: HandPig::new(Database::get_or_init_pool()),
    chat_pig: ChatPig::new(Database::get_or_init_pool()),
    other: Other::new(Database::get_or_init_pool()),
});

pub struct DBScheme {
    pub hand_pig: HandPig,
    pub chat_pig: ChatPig,
    pub other: Other,
}

pub struct Database {
    pub pool: &'static MyPool,
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}

impl Database {
    pub fn new() -> Database {
        Database { pool: Self::get_or_init_pool() }
    }

    fn get_or_init_pool() -> &'static MyPool {
        static POOL: OnceLock<MyPool> = OnceLock::new();

        POOL.get_or_init(|| {
            let config = Self::get_config();
            Pool::builder(config)
                .build()
                .expect("Something wrong with Pool manager!")
        })
    }

    fn get_config() -> AsyncDieselConnectionManager<DbConn> {
        AsyncDieselConnectionManager::<DbConn>::new(
            BOT_CONFIG.database_url.to_string(),
        )
    }
}
