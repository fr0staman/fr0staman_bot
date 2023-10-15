use chrono::NaiveDate;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::{consts::TOP_LIMIT, db::MyPool, models::Game, MyResult};

#[derive(Clone)]
pub struct ChatPig {
    pool: &'static MyPool,
}

impl ChatPig {
    pub fn new(pool: &'static MyPool) -> Self {
        Self { pool }
    }

    pub async fn get_chat_pig(
        &self,
        id_user: u64,
        id_chat: i64,
    ) -> MyResult<Option<Game>> {
        use crate::schema::game::dsl::*;
        use crate::schema::groups;
        use crate::schema::users;

        let results = game
            .inner_join(groups::table)
            .inner_join(users::table)
            .filter(users::user_id.eq(&id_user))
            .filter(groups::chat_id.eq(&id_chat))
            .select(Game::as_select())
            .first(&mut self.pool.get().await?)
            .await
            .optional()?;

        Ok(results)
    }

    pub async fn get_biggest_chat_pig(
        &self,
        id_user: u64,
    ) -> MyResult<Option<Game>> {
        use crate::schema::game::dsl::*;
        use crate::schema::users;

        let results: Option<Game> = game
            .inner_join(users::table)
            .filter(users::user_id.eq(&id_user))
            .order(mass.desc())
            .limit(1)
            .select(Game::as_select())
            .first(&mut self.pool.get().await?)
            .await
            .optional()?;

        Ok(results)
    }

    pub async fn set_chat_pig_name(
        &self,
        id_user: u64,
        id_chat: i64,
        new_name: String,
    ) -> MyResult<()> {
        use crate::schema::game::dsl::*;
        use crate::schema::groups;
        use crate::schema::users;

        diesel::update(game)
            .set(name.eq(new_name))
            .filter(
                uid.eq_any(
                    users::table
                        .select(users::id)
                        .filter(users::user_id.eq(&id_user)),
                ),
            )
            .filter(
                group_id.eq_any(
                    groups::table
                        .select(groups::id)
                        .filter(groups::chat_id.eq(id_chat)),
                ),
            )
            .execute(&mut self.pool.get().await?)
            .await?;

        Ok(())
    }

    pub async fn set_chat_pig_mass_n_date(
        &self,
        id_user: u64,
        id_chat: i64,
        other_mass: i32,
        cur_date: NaiveDate,
    ) -> MyResult<()> {
        use crate::schema::game::dsl::*;
        use crate::schema::groups;
        use crate::schema::users;

        diesel::update(game)
            .set((mass.eq(other_mass), date.eq(cur_date)))
            .filter(
                uid.eq_any(
                    users::table
                        .select(users::id)
                        .filter(users::user_id.eq(&id_user)),
                ),
            )
            .filter(
                group_id.eq_any(
                    groups::table
                        .select(groups::id)
                        .filter(groups::chat_id.eq(id_chat)),
                ),
            )
            .execute(&mut self.pool.get().await?)
            .await?;

        Ok(())
    }

    pub async fn create_chat_pig(
        &self,
        id_user: u32,
        id_group: i32,
        cur_name: &str,
        cur_date: NaiveDate,
        start_mass: i32,
    ) -> MyResult<()> {
        use crate::schema::game::dsl::*;

        diesel::insert_into(game)
            .values((
                uid.eq(id_user),
                group_id.eq(id_group),
                name.eq(cur_name),
                f_name.eq(cur_name),
                date.eq(cur_date),
                mass.eq(start_mass),
            ))
            .execute(&mut self.pool.get().await?)
            .await?;

        Ok(())
    }

    pub async fn get_top50_chat_pigs(
        &self,
        id_chat: i64,
        min: i32,
        offset_multiplier: i64,
    ) -> MyResult<Vec<Game>> {
        use crate::schema::game::dsl::*;
        use crate::schema::groups;

        let results = game
            .filter(groups::chat_id.eq(id_chat))
            .filter(mass.gt(min))
            .order(mass.desc())
            .limit(TOP_LIMIT)
            .offset(TOP_LIMIT * offset_multiplier)
            .select(Game::as_select())
            .inner_join(groups::table)
            .load(&mut self.pool.get().await?)
            .await?;

        Ok(results)
    }

    pub async fn count_chat_pig(
        &self,
        id_chat: i64,
        min: i32,
    ) -> MyResult<i64> {
        use crate::schema::game::dsl::*;
        use crate::schema::groups;

        let results = game
            .filter(groups::chat_id.eq(id_chat))
            .filter(mass.gt(min))
            .count()
            .inner_join(groups::table)
            .first(&mut self.pool.get().await?)
            .await?;

        Ok(results)
    }
}
