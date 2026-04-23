use teloxide::types::{Chat, User as TelegramUser};

use crate::db::DB;
use crate::db::models::{
    Groups, NewGroup, NewUser, UpdateGroups, UpdateUser, User,
};
use crate::types::MyResult;

use crate::utils::date::get_datetime;

pub async fn maybe_get_or_insert_user(
    from: &TelegramUser,
    started: bool,
) -> MyResult<Option<User>> {
    if let Some(user) = DB.other.get_user(from.id.0 as i64).await? {
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

            DB.other.update_user(from.id.0 as i64, update_info).await?;

            return Ok(Some(User {
                first_name: first_name.clone(),
                last_name: last_name.clone(),
                username: username.clone(),
                ..user
            }));
        }

        return Ok(Some(user));
    }

    let new_user = NewUser {
        user_id: from.id.0 as i64,
        created_at: get_datetime(),
        first_name: &from.first_name,
        last_name: from.last_name.as_deref(),
        username: from.username.as_deref(),
        lang: None,
        started,
        banned: false,
        supported: false,
        subscribed: false,
    };

    let user = DB.other.register_user(new_user).await?;
    Ok(Some(user))
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

        // Сheck "active" chat status from db for accuracy and update
        if !group_info.active {
            update = true
        }

        if update {
            let title_str = title.to_string();
            let username_opt = username.map(|v| v.to_string());
            let update_group = UpdateGroups {
                title: title_str.clone(),
                username: username_opt.clone(),
                active: true,
                ..group_info.to_update()
            };
            DB.other.update_chat(chat.id.0, update_group).await?;

            return Ok(Some(Groups {
                title: title_str,
                username: username_opt,
                active: true,
                ..group_info
            }));
        }

        return Ok(Some(group_info));
    }

    let new_chat = NewGroup {
        chat_id: chat.id.0,
        date: get_datetime(),
        title: chat.title().unwrap_or(""),
        username: chat.username(),
        lang: None,
        active: true,
        top10_setting: 0,
        settings: 0,
        ig_id: None,
    };

    let group = DB.other.add_chat(new_chat).await?;
    Ok(Some(group))
}
