use std::fmt::Debug;
use std::sync::Arc;

use futures::future::BoxFuture;
use futures::FutureExt;
use teloxide::{
    error_handlers::ErrorHandler, requests::Requester, types::ChatId,
};

use crate::config::BOT_CONFIG;

pub struct MyErrorHandler {
    text: String,
}

impl MyErrorHandler {
    pub fn with_custom_text<T>(text: T) -> Arc<Self>
    where
        T: Into<String>,
    {
        Arc::new(Self { text: text.into() })
    }

    pub fn new() -> Arc<Self> {
        Self::with_custom_text("Error".to_owned())
    }
}

impl<E> ErrorHandler<E> for MyErrorHandler
where
    E: Debug,
{
    fn handle_error(self: Arc<Self>, error: E) -> BoxFuture<'static, ()> {
        let error_text = format!("{text}: {:?}", error, text = self.text);
        log::error!("{}", &error_text);

        log_error(error_text).boxed()
    }
}

pub async fn log_error(text: String) {
    let _ = BOT_CONFIG
        .bot
        .send_message(ChatId(BOT_CONFIG.log_group_id), text)
        .await;
}

pub fn fire_log_error(text: String) {
    tokio::spawn(log_error(text));
}

#[macro_export]
macro_rules! myerr {
    ($($arg:tt)+) => {
        {
            let error_text = format!($($arg)+);
            log::error!("{}", &error_text);
            $crate::utils::mylog::fire_log_error(error_text);
        }
    };
}
