use teloxide::types::{Chat, User as TelegramUser};

use crate::db::DB;
use crate::models::{Groups, NewGroup, NewUser, User};
use crate::MyResult;

use super::date::get_datetime;

pub async fn maybe_get_or_insert_user(
    from: &TelegramUser,
    started: bool,
) -> MyResult<Option<User>> {
    let user = DB.other.get_user(from.id.0).await?;

    if user.is_some() {
        return Ok(user);
    }

    let new_user = NewUser {
        user_id: from.id.0,
        created_at: get_datetime(),
        first_name: &from.first_name,
        last_name: from.last_name.as_deref(),
        username: from.username.as_deref(),
        started,
    };

    DB.other.register_user(new_user).await?;
    DB.other.get_user(from.id.0).await
}

pub async fn maybe_get_or_insert_chat(chat: &Chat) -> MyResult<Option<Groups>> {
    let group_info = DB.other.get_chat(chat.id.0).await?;

    if group_info.is_some() {
        return Ok(group_info);
    }

    let new_chat = NewGroup {
        chat_id: chat.id.0,
        date: get_datetime(),
        title: chat.title().unwrap_or(""),
        username: chat.username(),
    };

    DB.other.add_chat(new_chat).await?;
    DB.other.get_chat(chat.id.0).await
}
