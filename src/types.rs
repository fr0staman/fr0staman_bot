use diesel_async::pooled_connection::deadpool::{Pool, PoolError};
use teloxide::{adaptors::DefaultParseMode, prelude::*};

#[derive(thiserror::Error, Debug)]
pub enum MyError {
    #[error(transparent)]
    Database(#[from] diesel::result::Error),

    #[error(transparent)]
    DatabaseConnectionError(#[from] diesel::ConnectionError),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error(transparent)]
    RequestError(#[from] teloxide::RequestError),

    #[error(transparent)]
    PoolError(#[from] PoolError),

    #[allow(unused)]
    #[error("unknown error: {0}")]
    Unknown(String),
}

pub type MyBot = DefaultParseMode<Bot>;
pub type MyResult<T> = Result<T, MyError>;

pub type ParsedCallbackData<'a> = (&'a str, UserId, &'a str);

pub type DbConn = diesel_async::AsyncMysqlConnection;
pub type DbPool = Pool<DbConn>;
