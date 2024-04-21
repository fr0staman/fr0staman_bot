use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::{
    db::MyPool,
    models::{
        Groups, InlineGif, InlineVoice, NewGroup, NewUser, UpdateGroups,
        UpdateUser, User, UserStatus,
    },
    MyResult,
};

#[derive(Clone)]
pub struct Other {
    pool: &'static MyPool,
}

impl Other {
    pub fn new(pool: &'static MyPool) -> Self {
        Self { pool }
    }

    pub async fn register_user(&self, new_user: NewUser<'_>) -> MyResult<()> {
        use crate::schema::users::dsl::*;
        diesel::insert_or_ignore_into(users)
            .values(new_user)
            .execute(&mut self.pool.get().await?)
            .await?;
        Ok(())
    }

    pub async fn get_user(&self, id_user: u64) -> MyResult<Option<User>> {
        use crate::schema::users::dsl::*;

        let result = users
            .filter(user_id.eq(id_user))
            .select(User::as_select())
            .first(&mut self.pool.get().await?)
            .await
            .optional()?;

        Ok(result)
    }

    pub async fn get_user_by_id(&self, uid: u32) -> MyResult<Option<User>> {
        use crate::schema::users::dsl::*;

        let result = users
            .filter(id.eq(uid))
            .select(User::as_select())
            .first(&mut self.pool.get().await?)
            .await
            .optional()?;

        Ok(result)
    }

    pub async fn change_user_status(
        &self,
        id_user: u64,
        status: UserStatus,
    ) -> MyResult<()> {
        use crate::schema::users::dsl::*;
        diesel::update(users)
            .set(&status)
            .filter(user_id.eq(id_user))
            .execute(&mut self.pool.get().await?)
            .await?;

        Ok(())
    }

    pub async fn change_user_lang(
        &self,
        id_user: u64,
        status: Option<&str>,
    ) -> MyResult<()> {
        use crate::schema::users::dsl::*;
        diesel::update(users)
            .set(lang.eq(status))
            .filter(user_id.eq(id_user))
            .execute(&mut self.pool.get().await?)
            .await?;

        Ok(())
    }

    pub async fn get_inline_voice_by_id(
        &self,
        voice_id: i16,
    ) -> MyResult<Option<InlineVoice>> {
        use crate::schema::inline_voices::dsl::*;

        let results = inline_voices
            .filter(status.eq(1))
            .filter(id.eq(voice_id))
            .select(InlineVoice::as_select())
            .first(&mut self.pool.get().await?)
            .await
            .optional()?;

        Ok(results)
    }

    pub async fn get_inline_voices(&self) -> MyResult<Vec<InlineVoice>> {
        use crate::schema::inline_voices::dsl::*;

        let results = inline_voices
            .filter(status.eq(1))
            .order_by(id.desc())
            .select(InlineVoice::as_select())
            .load(&mut self.pool.get().await?)
            .await?;

        Ok(results)
    }

    pub async fn get_inline_gif_by_id(
        &self,
        voice_id: i16,
    ) -> MyResult<Option<InlineGif>> {
        use crate::schema::inline_gifs::dsl::*;

        let results = inline_gifs
            .filter(status.eq(1))
            .filter(id.eq(voice_id))
            .select(InlineGif::as_select())
            .first(&mut self.pool.get().await?)
            .await
            .optional()?;

        Ok(results)
    }

    pub async fn get_inline_gifs(&self) -> MyResult<Vec<InlineGif>> {
        use crate::schema::inline_gifs::dsl::*;

        let results = inline_gifs
            .filter(status.eq(1))
            .order_by(id.desc())
            .select(InlineGif::as_select())
            .load(&mut self.pool.get().await?)
            .await?;

        Ok(results)
    }

    pub async fn get_chat(&self, id_chat: i64) -> MyResult<Option<Groups>> {
        use crate::schema::groups::dsl::*;

        let results = groups
            .filter(chat_id.eq(id_chat))
            .select(Groups::as_select())
            .first(&mut self.pool.get().await?)
            .await
            .optional()?;

        Ok(results)
    }

    pub async fn add_chat(&self, new_group: NewGroup<'_>) -> MyResult<()> {
        use crate::schema::groups::dsl::*;

        diesel::insert_or_ignore_into(groups)
            .values(new_group)
            .execute(&mut self.pool.get().await?)
            .await?;

        Ok(())
    }

    pub async fn update_chat(
        &self,
        id_chat: i64,
        chat_info: UpdateGroups,
    ) -> MyResult<()> {
        use crate::schema::groups::dsl::*;

        diesel::update(groups)
            .set(chat_info)
            .filter(chat_id.eq(id_chat))
            .execute(&mut self.pool.get().await?)
            .await?;

        Ok(())
    }

    pub async fn set_chat_settings(
        &self,
        id_chat: i64,
        setting: i8,
    ) -> MyResult<()> {
        use crate::schema::groups::dsl::*;

        diesel::update(groups)
            .set(settings.eq(setting))
            .filter(chat_id.eq(id_chat))
            .execute(&mut self.pool.get().await?)
            .await?;

        Ok(())
    }

    pub async fn set_top10_setting(
        &self,
        id_chat: i64,
        setting: i32,
    ) -> MyResult<()> {
        use crate::schema::groups::dsl::*;

        diesel::update(groups)
            .set(top10_setting.eq(setting))
            .filter(chat_id.eq(id_chat))
            .execute(&mut self.pool.get().await?)
            .await?;

        Ok(())
    }

    pub async fn add_voice(
        &self,
        iv_uid: u32,
        new_url: String,
    ) -> MyResult<()> {
        use crate::schema::inline_voices::dsl::*;

        diesel::insert_into(inline_voices)
            .values((url.eq(new_url), uid.eq(iv_uid)))
            .execute(&mut self.pool.get().await?)
            .await?;

        Ok(())
    }

    pub async fn get_voices_by_user(
        &self,
        iv_uid: u32,
    ) -> MyResult<Vec<InlineVoice>> {
        use crate::schema::inline_voices::dsl::*;

        let results = inline_voices
            .filter(uid.eq(iv_uid))
            .select(InlineVoice::as_select())
            .load(&mut self.pool.get().await?)
            .await?;

        Ok(results)
    }

    pub async fn add_gif(
        &self,
        iv_uid: u32,
        new_file_id: String,
        new_file_unique_id: String,
    ) -> MyResult<()> {
        use crate::schema::inline_gifs::dsl::*;

        diesel::insert_into(inline_gifs)
            .values((
                file_id.eq(new_file_id),
                file_unique_id.eq(new_file_unique_id),
                uid.eq(iv_uid),
            ))
            .execute(&mut self.pool.get().await?)
            .await?;

        Ok(())
    }

    pub async fn get_gifs_by_user(
        &self,
        iv_uid: u32,
    ) -> MyResult<Vec<InlineGif>> {
        use crate::schema::inline_gifs::dsl::*;

        let results = inline_gifs
            .filter(uid.eq(iv_uid))
            .select(InlineGif::as_select())
            .load(&mut self.pool.get().await?)
            .await?;

        Ok(results)
    }

    pub async fn get_gif_by_file_unique_id(
        &self,
        id_file_unique: &str,
    ) -> MyResult<Option<InlineGif>> {
        use crate::schema::inline_gifs::dsl::*;

        let results = inline_gifs
            .filter(file_unique_id.eq(id_file_unique))
            .select(InlineGif::as_select())
            .first(&mut self.pool.get().await?)
            .await
            .optional()?;

        Ok(results)
    }

    pub async fn update_chat_id(
        &self,
        from_id: i64,
        to_id: i64,
    ) -> MyResult<()> {
        use crate::schema::groups::dsl::*;

        diesel::update(groups)
            .filter(chat_id.eq(from_id))
            .set(chat_id.eq(to_id))
            .execute(&mut self.pool.get().await?)
            .await?;

        Ok(())
    }

    pub async fn update_chat_ig_id(
        &self,
        id_chat: i64,
        my_ig_id: Option<i32>,
    ) -> MyResult<()> {
        use crate::schema::groups::dsl::*;

        diesel::update(groups)
            .filter(chat_id.eq(id_chat))
            .set(ig_id.eq(my_ig_id))
            .execute(&mut self.pool.get().await?)
            .await?;

        Ok(())
    }

    pub async fn update_user(
        &self,
        id_user: u64,
        chat_info: UpdateUser,
    ) -> MyResult<()> {
        use crate::schema::users::dsl::*;

        diesel::update(users)
            .set(chat_info)
            .filter(user_id.eq(id_user))
            .execute(&mut self.pool.get().await?)
            .await?;

        Ok(())
    }
}
