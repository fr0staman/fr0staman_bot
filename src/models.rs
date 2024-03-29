// Generated by diesel_ext
use chrono::NaiveDate;
use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::schema::*;

#[derive(Queryable, Debug)]
#[diesel(table_name = counter)]
pub struct Counter {
    pub id: i32,
    pub count: i32,
}

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = game)]
pub struct Game {
    pub id: i32,
    pub uid: u32,
    pub group_id: i32,
    pub mass: i32,
    pub date: NaiveDate,
    pub name: String,
    pub f_name: String,
}

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = groups)]
pub struct Groups {
    pub id: i32,
    pub chat_id: i64,
    pub date: NaiveDateTime,
    pub settings: i8,
    pub top10_setting: i32,
    pub lang: Option<String>,
    pub active: bool,
    pub ig_id: Option<i32>,
}

#[derive(Queryable, AsChangeset)]
#[diesel(table_name = groups)]
pub struct UpdateGroups {
    pub settings: i8,
    pub top10_setting: i32,
    pub lang: Option<String>,
    pub active: bool,
}

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = hryak_day)]
pub struct HryakDay {
    pub id: i32,
    pub iug_id: i32,
    pub date: NaiveDate,
}

#[derive(Queryable, Selectable, Identifiable, Debug)]
#[diesel(table_name = inline_groups)]
pub struct InlineGroup {
    pub id: i32,
    pub chat_instance: i64,
    pub invited_at: NaiveDateTime,
}

#[derive(Queryable, Selectable, Identifiable, Debug)]
#[diesel(table_name = inline_users)]
pub struct InlineUser {
    pub id: i32,
    pub uid: u32,
    pub f_name: String,
    pub weight: i32,
    pub date: NaiveDate,
    pub flag: String,
    pub win: u16,
    pub rout: u16,
    pub name: String,
    pub gifted: bool,
}

#[derive(Queryable, Selectable, Identifiable, Debug)]
#[diesel(table_name = inline_users_groups)]
pub struct InlineUsersGroup {
    pub id: i32,
    pub iu_id: i32,
    pub ig_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = inline_users)]
pub struct NewInlineUser<'a> {
    pub uid: u32,
    pub f_name: &'a str,
    pub weight: i32,
    pub date: NaiveDate,
    pub flag: &'a str,
    pub name: &'a str,
}

#[derive(Queryable)]
#[diesel(table_name = inline_users)]
pub struct UpdateInlineUser<'a> {
    pub id: i32,
    pub f_name: &'a str,
    pub weight: i32,
    pub date: NaiveDate,
    pub gifted: bool,
}

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = inline_voices)]
pub struct InlineVoice {
    pub id: i16,
    pub url: String,
    pub uid: u32,
    pub caption: String,
    pub status: i16,
}

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = inline_gifs)]
pub struct InlineGif {
    pub id: i16,
    pub file_id: String,
    pub file_unique_id: String,
    pub uid: u32,
    pub status: i16,
}

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = users)]
pub struct User {
    pub id: u32,
    pub user_id: u64,
    pub started: bool,
    pub banned: bool,
    pub supported: bool,
    pub subscribed: bool,
    pub created_at: NaiveDateTime,
    pub lang: Option<String>,
}

#[derive(Queryable, AsChangeset)]
#[diesel(table_name = users)]
pub struct UserStatus {
    pub started: bool,
    pub banned: bool,
    pub supported: bool,
    pub subscribed: bool,
}
