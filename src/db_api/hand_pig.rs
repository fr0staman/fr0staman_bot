use chrono::{NaiveDate, NaiveDateTime};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::{
    db::MyPool,
    models::{
        HryakDay, InlineGroup, InlineUser, InlineUsersGroup, NewInlineUser,
        UpdateInlineUser, User,
    },
    MyResult,
};

#[derive(Clone)]
pub struct HandPig {
    pool: &'static MyPool,
}

impl HandPig {
    pub fn new(pool: &'static MyPool) -> Self {
        Self { pool }
    }

    pub async fn update_hrundel_duel(
        &self,
        id_user: u64,
        offset: i32,
        is_win: bool,
    ) -> MyResult<()> {
        use crate::schema::inline_users::dsl::*;
        use crate::schema::users;
        use diesel::dsl::sql;

        if is_win {
            diesel::update(inline_users)
                .filter(
                    uid.eq_any(
                        users::table
                            .select(users::id)
                            .filter(users::user_id.eq(&id_user)),
                    ),
                )
                .set((weight.eq(weight + offset), win.eq(win + 1)))
                .execute(&mut self.pool.get().await?)
                .await?;
        } else {
            let condition = sql(&format!(
                "IF(weight > {}, weight - {}, weight - (weight - 1))",
                offset, offset
            ));

            diesel::update(inline_users)
                .filter(
                    uid.eq_any(
                        users::table
                            .select(users::id)
                            .filter(users::user_id.eq(&id_user)),
                    ),
                )
                .set((weight.eq(condition), rout.eq(rout + 1)))
                .execute(&mut self.pool.get().await?)
                .await?;
        }
        Ok(())
    }

    pub async fn update_hrundel_date_and_size(
        &self,
        id_user: u64,
        size: i32,
        cur_date: NaiveDate,
    ) -> MyResult<()> {
        use crate::schema::inline_users::dsl::*;
        use crate::schema::users;

        diesel::update(inline_users)
            .filter(
                uid.eq_any(
                    users::table
                        .select(users::id)
                        .filter(users::user_id.eq(&id_user)),
                ),
            )
            .set((date.eq(cur_date), weight.eq(size)))
            .execute(&mut self.pool.get().await?)
            .await?;
        Ok(())
    }

    pub async fn update_hrundel_name(
        &self,
        id_user: u64,
        new_name: &str,
    ) -> MyResult<()> {
        use crate::schema::inline_users::dsl::*;
        use crate::schema::users;

        diesel::update(inline_users)
            .filter(
                uid.eq_any(
                    users::table
                        .select(users::id)
                        .filter(users::user_id.eq(&id_user)),
                ),
            )
            .set(name.eq(new_name))
            .execute(&mut self.pool.get().await?)
            .await?;

        Ok(())
    }

    pub async fn update_hrundel_flag(
        &self,
        id_user: u64,
        new_flag: &str,
    ) -> MyResult<()> {
        use crate::schema::inline_users::dsl::*;
        use crate::schema::users;

        diesel::update(inline_users)
            .filter(
                uid.eq_any(
                    users::table
                        .select(users::id)
                        .filter(users::user_id.eq(&id_user)),
                ),
            )
            .set(flag.eq(new_flag))
            .execute(&mut self.pool.get().await?)
            .await?;

        Ok(())
    }

    pub async fn update_hrundel(&self, data: UpdateInlineUser) -> MyResult<()> {
        use crate::schema::inline_users::dsl::*;

        diesel::update(inline_users.find(data.id))
            .set((
                date.eq(data.date),
                weight.eq(data.weight),
                gifted.eq(data.gifted),
            ))
            .execute(&mut self.pool.get().await?)
            .await?;

        Ok(())
    }

    pub async fn add_hrundel(
        &self,
        hrundel: NewInlineUser<'_>,
    ) -> MyResult<()> {
        use crate::schema::inline_users::dsl::*;

        diesel::insert_into(inline_users)
            .values(&hrundel)
            .execute(&mut self.pool.get().await?)
            .await?;

        Ok(())
    }

    pub async fn get_hrundel(
        &self,
        id_user: u64,
    ) -> MyResult<Option<(InlineUser, User)>> {
        use crate::schema::inline_users;
        use crate::schema::users;

        let results = inline_users::table
            .filter(users::user_id.eq(id_user))
            .inner_join(users::table)
            .select((InlineUser::as_select(), User::as_select()))
            .first(&mut self.pool.get().await?)
            .await
            .optional()?;

        Ok(results)
    }

    pub async fn get_hryak_day_in_chat(
        &self,
        chat_instance: &str,
        the_date: NaiveDate,
    ) -> MyResult<Option<(InlineGroup, HryakDay, InlineUser, User)>> {
        use crate::schema::hryak_day;
        use crate::schema::inline_groups;
        use crate::schema::inline_users;
        use crate::schema::inline_users_groups;
        use crate::schema::users;

        let parsed_instance = chat_instance.parse::<i64>().unwrap_or(1);
        let results = hryak_day::table
            .filter(hryak_day::date.eq(the_date))
            .filter(inline_groups::chat_instance.eq(parsed_instance))
            .inner_join(
                inline_users_groups::table
                    .inner_join(inline_groups::table)
                    .inner_join(inline_users::table.inner_join(users::table)),
            )
            .select((
                InlineGroup::as_select(),
                HryakDay::as_select(),
                InlineUser::as_select(),
                User::as_select(),
            ))
            .first(&mut self.pool.get().await?)
            .await
            .optional()?;

        Ok(results)
    }

    pub async fn get_inline_group(
        &self,
        instance_chat: &str,
    ) -> MyResult<Option<InlineGroup>> {
        use crate::schema::inline_groups::dsl::*;

        let parsed_instance = instance_chat.parse::<i64>().unwrap_or(1);
        let results = inline_groups
            .filter(chat_instance.eq(parsed_instance))
            .select(InlineGroup::as_select())
            .first(&mut self.pool.get().await?)
            .await
            .optional()?;

        Ok(results)
    }

    pub async fn get_inline_group_by_id(
        &self,
        primary_id: i32,
    ) -> MyResult<Option<InlineGroup>> {
        use crate::schema::inline_groups::dsl::*;

        let results = inline_groups
            .filter(id.eq(primary_id))
            .select(InlineGroup::as_select())
            .first(&mut self.pool.get().await?)
            .await
            .optional()?;

        Ok(results)
    }

    pub async fn add_inline_group(
        &self,
        instance_chat: &str,
        cur_datetime: NaiveDateTime,
    ) -> MyResult<()> {
        use crate::schema::inline_groups::dsl::*;

        diesel::insert_into(inline_groups)
            .values((
                chat_instance.eq(instance_chat.parse::<i64>().unwrap_or(1)),
                invited_at.eq(cur_datetime),
            ))
            .execute(&mut self.pool.get().await?)
            .await?;

        Ok(())
    }

    pub async fn get_rand_inline_user_group_by_chat(
        &self,
        instance_chat: &str,
    ) -> MyResult<Option<InlineUsersGroup>> {
        use crate::schema::inline_groups;
        use crate::schema::inline_users_groups;

        sql_function!(fn rand() -> Text);

        let parsed_instance = instance_chat.parse::<i64>().unwrap_or(1);
        let results = inline_users_groups::table
            .inner_join(inline_groups::table)
            .filter(inline_groups::chat_instance.eq(parsed_instance))
            .select(InlineUsersGroup::as_select())
            .order_by(rand())
            .first(&mut self.pool.get().await?)
            .await
            .optional()?;

        Ok(results)
    }

    pub async fn get_group_user(
        &self,
        instance_chat: &str,
        id_user: u64,
    ) -> MyResult<Option<InlineUsersGroup>> {
        use crate::schema::inline_groups;
        use crate::schema::inline_users;
        use crate::schema::inline_users_groups;
        use crate::schema::users;

        let parsed_instance = instance_chat.parse::<i64>().unwrap_or(1);
        let results = inline_users_groups::table
            .inner_join(inline_groups::table)
            .inner_join(inline_users::table)
            .filter(inline_groups::chat_instance.eq(parsed_instance))
            .filter(
                inline_users::uid.eq_any(
                    users::table
                        .select(users::id)
                        .filter(users::user_id.eq(&id_user)),
                ),
            )
            .select(InlineUsersGroup::as_select())
            .first(&mut self.pool.get().await?)
            .await
            .optional()?;

        Ok(results)
    }

    pub async fn add_group_to_user(
        &self,
        id_iu: i32,
        id_ig: i32,
    ) -> MyResult<()> {
        use crate::schema::inline_users_groups::dsl::*;

        diesel::insert_into(inline_users_groups)
            .values((iu_id.eq(id_iu), ig_id.eq(id_ig)))
            .execute(&mut self.pool.get().await?)
            .await?;

        Ok(())
    }

    pub async fn add_hryak_day_to_chat(
        &self,
        user_to_chat_id: i32,
        current_date: NaiveDate,
    ) -> MyResult<()> {
        use crate::schema::hryak_day::dsl::*;

        diesel::insert_into(hryak_day)
            .values((iug_id.eq(user_to_chat_id), date.eq(current_date)))
            .execute(&mut self.pool.get().await?)
            .await?;

        Ok(())
    }

    pub async fn get_top10_chat(
        &self,
        chat_instance: &str,
        cur_date: NaiveDate,
    ) -> MyResult<Option<Vec<InlineUser>>> {
        use crate::schema::inline_groups;
        use crate::schema::inline_users;
        use crate::schema::inline_users_groups;

        let parsed_instance = chat_instance.parse::<i64>().unwrap_or(1);

        let results = inline_users::table
            .filter(inline_users::date.eq(cur_date))
            .filter(inline_groups::chat_instance.eq(parsed_instance))
            .inner_join(
                inline_users_groups::table.inner_join(inline_groups::table),
            )
            .order_by(inline_users::weight.desc())
            .limit(10)
            .select(InlineUser::as_select())
            .load(&mut self.pool.get().await?)
            .await?;

        if results.is_empty() {
            return Ok(None);
        }

        Ok(Some(results))
    }

    pub async fn get_top10_global(
        &self,
        cur_date: NaiveDate,
    ) -> MyResult<Option<Vec<InlineUser>>> {
        use crate::schema::inline_users::dsl::*;

        let results = inline_users
            .filter(date.eq(cur_date))
            .order_by(weight.desc())
            .limit(10)
            .select(InlineUser::as_select())
            .load(&mut self.pool.get().await?)
            .await?;

        if results.is_empty() {
            return Ok(None);
        }

        Ok(Some(results))
    }

    pub async fn get_top10_win(&self) -> MyResult<Option<Vec<InlineUser>>> {
        use crate::schema::inline_users::dsl::*;

        let results = inline_users
            .order_by(win.desc())
            .limit(10)
            .select(InlineUser::as_select())
            .load(&mut self.pool.get().await?)
            .await?;

        if results.is_empty() {
            return Ok(None);
        }

        Ok(Some(results))
    }
}
