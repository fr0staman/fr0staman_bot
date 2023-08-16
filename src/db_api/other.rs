use crate::{
    db::MyPool,
    models::{Groups, InlineVoice, User},
    MyResult,
};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

#[derive(Clone)]
pub struct Other {
    pool: &'static MyPool,
}

impl Other {
    pub fn new(pool: &'static MyPool) -> Self {
        Self { pool }
    }

    pub async fn register_user(&self, id_user: u64) -> MyResult<()> {
        use crate::schema::users::dsl::*;
        diesel::insert_or_ignore_into(users)
            .values(user_id.eq(id_user))
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
    pub async fn change_user_status(
        &self,
        id_user: u64,
        s: i8,
    ) -> MyResult<()> {
        use crate::schema::users::dsl::*;
        diesel::update(users)
            .set(status.eq(s))
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

    pub async fn get_50_inline_voices(&self) -> MyResult<Vec<InlineVoice>> {
        use crate::schema::inline_voices::dsl::*;
        sql_function!(fn rand() -> Text);

        let results = inline_voices
            .filter(status.eq(1))
            .limit(50)
            .order_by(rand())
            .select(InlineVoice::as_select())
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

    pub async fn add_chat(
        &self,
        id_chat: i64,
        cur_datetime: NaiveDateTime,
    ) -> MyResult<()> {
        use crate::schema::groups::dsl::*;

        diesel::insert_or_ignore_into(groups)
            .values((chat_id.eq(id_chat), date.eq(cur_datetime)))
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
        id_user: u64,
        new_url: String,
    ) -> MyResult<()> {
        use crate::schema::inline_voices::dsl::*;

        diesel::insert_into(inline_voices)
            .values((url.eq(new_url), user_id.eq(id_user)))
            .execute(&mut self.pool.get().await?)
            .await?;

        Ok(())
    }

    pub async fn get_voice(&self, id_user: u64) -> MyResult<Vec<InlineVoice>> {
        use crate::schema::inline_voices::dsl::*;

        let results = inline_voices
            .filter(user_id.eq(id_user))
            .select(InlineVoice::as_select())
            .load(&mut self.pool.get().await?)
            .await?;

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
}
