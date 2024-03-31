use diesel::{debug_query, mysql::Mysql, query_builder::DebugQuery};
use futures::TryStreamExt;
use std::iter;
use teloxide::{
    net::Download,
    types::{ChatKind, PublicChatKind, UserId},
    utils::html,
};
use unicode_width::UnicodeWidthChar;

use std::hash::Hasher;
use url::Url;

use crate::{
    config::BOT_CONFIG,
    enums::{CbActions, Image},
    models::User,
    types::ParsedCallbackData,
    MyBot,
};

const SEPARATOR: char = ':';

pub fn encode_callback_data<U>(
    action: CbActions,
    id_user: UserId,
    second: U,
) -> String
where
    U: Into<String>,
{
    let mut capacity = String::with_capacity(64);

    capacity.push_str(action.as_ref());
    capacity.push(SEPARATOR);
    capacity.push_str(&id_user.to_string());
    capacity.push(SEPARATOR);
    capacity.push_str(&second.into());
    capacity
}

pub fn decode_callback_data(data: &str) -> Option<ParsedCallbackData> {
    let splitted: Vec<&str> = data.splitn(3, SEPARATOR).collect();

    if splitted.len() < 3 {
        return None;
    }

    let action = splitted[0];
    let Ok(id_user) = splitted[1].parse::<u64>() else {
        return None;
    };

    let payload = splitted[2];

    Some((action, UserId(id_user), payload))
}

pub fn get_photostock(append: Image) -> Url {
    let url = &BOT_CONFIG.photostock_url;

    url.join(append.as_ref()).unwrap_or_else(|_| url.clone())
}

// Truncate to some width with emoji check, because it can be 2 bytes or even 4 bytes
pub fn truncate(s: &str, width: usize) -> (&str, usize) {
    let (bidx, new_width) = s
        .char_indices()
        .map(|(bidx, c)| (bidx, c.width().unwrap_or(0)))
        .chain(iter::once((s.len(), 0)))
        .scan(0, |w, (bidx, cw)| {
            let curr_w = *w;
            *w += cw;
            Some((bidx, curr_w))
        })
        .take_while(|&(_, w)| w <= width)
        .last()
        .unwrap_or((0, 0));
    (s.get(..bidx).unwrap(), new_width)
}

// Purpose: for logging
pub fn get_chat_kind(kind: &ChatKind) -> &str {
    match kind {
        ChatKind::Public(kind) => match kind.kind {
            PublicChatKind::Channel(_) => "channel",
            PublicChatKind::Group(_) => "group",
            PublicChatKind::Supergroup(_) => "supergroup",
        },
        ChatKind::Private(_) => "private",
    }
}

pub fn get_hash<T>(value: T) -> u64
where
    T: std::hash::Hash,
{
    let mut hasher = ahash::AHasher::default();
    value.hash(&mut hasher);
    hasher.finish()
}

pub fn escape(s: &str) -> String {
    html::escape(s)
        .replace(['@', '\'', '\"'], "")
        .replace("t.me", "")
        .replace("telegram.me", "")
}

// Plural rule for languages, may some inaccurate - but it works!
pub fn plural(n: i64) -> i64 {
    if n % 10 == 1 && n % 100 != 11 {
        0
    } else if n % 10 >= 2 && n % 10 <= 4 && (n % 100 < 10 || n % 100 >= 20) {
        1
    } else {
        2
    }
}

#[allow(unused)]
pub fn db_debug<T>(query: &T) -> DebugQuery<'_, T, Mysql> {
    debug_query::<Mysql, _>(query)
}

pub fn mass_addition_on_status(user: &User) -> i32 {
    if user.supported {
        500
    } else if user.subscribed {
        100
    } else {
        0
    }
}

pub async fn get_file_from_stream(
    bot: &MyBot,
    file: &teloxide::types::File,
) -> Option<bytes::Bytes> {
    bot.download_file_stream(&file.path)
        .try_collect()
        .await
        .map(bytes::BytesMut::freeze)
        .ok()
}
