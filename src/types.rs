use teloxide::{adaptors::DefaultParseMode, prelude::*};

pub type MyBot = DefaultParseMode<Bot>;
pub type MyResult<T> = Result<T, Error>;

use diesel_async::pooled_connection::deadpool::PoolError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
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

    #[error("unknown error: {0}")]
    Unknown(String),
}
