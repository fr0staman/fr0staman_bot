use chrono::NaiveDate;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::db::MyPool;
use crate::models::Game;
use crate::schema::game::dsl::*;
use crate::{MyResult, TOP_LIMIT};

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
        use crate::schema::groups;
        let results = game
            .filter(user_id.eq(&id_user))
            .filter(groups::chat_id.eq(&id_chat))
            .select(Game::as_select())
            .inner_join(groups::table)
            .first(&mut self.pool.get().await?)
            .await
            .optional()?;

        Ok(results)
    }

    pub async fn get_chat_pigs(
        &self,
        id_user: u64,
    ) -> MyResult<Option<Vec<Game>>> {
        let results: Vec<Game> = game
            .filter(user_id.eq(&id_user))
            .select(Game::as_select())
            .load(&mut self.pool.get().await?)
            .await?;

        if results.is_empty() {
            return Ok(None);
        }
        Ok(Some(results))
    }

    pub async fn get_biggest_chat_pig(
        &self,
        id_user: u64,
    ) -> MyResult<Option<Game>> {
        let results: Option<Game> = game
            .filter(user_id.eq(&id_user))
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
        use crate::schema::groups;

        diesel::update(game)
            .set(name.eq(new_name))
            .filter(user_id.eq(id_user))
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
        use crate::schema::groups;

        diesel::update(game)
            .set((mass.eq(other_mass), date.eq(cur_date)))
            .filter(user_id.eq(&id_user))
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
        id_user: u64,
        id_group: i32,
        cur_name: &str,
        cur_date: NaiveDate,
    ) -> MyResult<()> {
        diesel::insert_into(game)
            .values((
                user_id.eq(id_user),
                group_id.eq(id_group),
                name.eq(cur_name),
                f_name.eq(cur_name),
                date.eq(cur_date),
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
