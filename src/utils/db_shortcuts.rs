use teloxide::types::{Chat, User as TelegramUser};

use crate::db::DB;
use crate::models::{
    Groups, NewGroup, NewUser, UpdateGroups, UpdateUser, User,
};
use crate::MyResult;

use super::date::get_datetime;

pub async fn maybe_get_or_insert_user(
    from: &TelegramUser,
    started: bool,
) -> MyResult<Option<User>> {
    if let Some(user) = DB.other.get_user(from.id.0).await? {
        let mut update = false;

        let TelegramUser { first_name, last_name, username, .. } = from;

        if *first_name != user.first_name {
            update = true;
        }

        if *username != user.username {
            update = true;
        }

        if *last_name != user.last_name {
            update = true;
        }

        if update {
            let update_info = UpdateUser {
                first_name: first_name.clone(),
                last_name: last_name.clone(),
                username: username.clone(),
                ..user.to_update()
            };

            DB.other.update_user(from.id.0, update_info).await?;

            return DB.other.get_user(from.id.0).await;
        }

        return Ok(Some(user));
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
    if let Some(group_info) = DB.other.get_chat(chat.id.0).await? {
        let mut update = false;

        let title = chat.title().unwrap_or("");
        let username = chat.username();

        if title != group_info.title {
            update = true;
        };

        if username != group_info.username.as_deref() {
            update = true;
        }

        if update {
            let update_group = UpdateGroups {
                title: title.to_string(),
                username: username.map(|v| v.to_string()),
                ..group_info.to_update()
            };
            DB.other.update_chat(chat.id.0, update_group).await?;

            return DB.other.get_chat(chat.id.0).await;
        }

        return Ok(Some(group_info));
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
