use ahash::AHashMap;
use chrono::{Duration, NaiveDate};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::{
    config::consts::{CHAT_PIG_START_MASS, TOP_LIMIT, TOP_LIMIT_WITH_CHARTS},
    db::models::{Game, GrowLog, GrowLogAdd, User},
    types::{DbPool, MyResult},
    utils::date::{get_date, get_datetime},
};

#[derive(Clone)]
pub struct ChatPig {
    pool: &'static DbPool,
}

impl ChatPig {
    pub fn new(pool: &'static DbPool) -> Self {
        Self { pool }
    }

    pub async fn get_chat_pig(
        &self,
        id_user: i64,
        id_chat: i64,
    ) -> MyResult<Option<Game>> {
        use crate::db::schema::game::dsl::*;
        use crate::db::schema::groups;
        use crate::db::schema::users;

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

    pub async fn get_chat_pig_by_id(
        &self,
        id_game: i32,
    ) -> MyResult<Option<Game>> {
        use crate::db::schema::game::dsl::*;

        let results = game
            .filter(id.eq(id_game))
            .select(Game::as_select())
            .first(&mut self.pool.get().await?)
            .await
            .optional()?;

        Ok(results)
    }

    pub async fn get_biggest_chat_pig(
        &self,
        id_user: i64,
    ) -> MyResult<Option<Game>> {
        use crate::db::schema::game::dsl::*;
        use crate::db::schema::users;

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
        id_user: i64,
        id_chat: i64,
        new_name: String,
    ) -> MyResult<()> {
        use crate::db::schema::game::dsl::*;
        use crate::db::schema::groups;
        use crate::db::schema::users;

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
        id_user: i64,
        id_chat: i64,
        other_mass: i32,
        cur_date: NaiveDate,
    ) -> MyResult<()> {
        use crate::db::schema::game::dsl::*;
        use crate::db::schema::groups;
        use crate::db::schema::users;

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
        id_user: i32,
        id_group: i32,
        cur_name: &str,
        cur_date: NaiveDate,
        start_mass: i32,
    ) -> MyResult<()> {
        use crate::db::schema::game::dsl::*;

        diesel::insert_into(game)
            .values((
                uid.eq(id_user),
                group_id.eq(id_group),
                name.eq(cur_name),
                date.eq(cur_date),
                mass.eq(start_mass),
            ))
            .execute(&mut self.pool.get().await?)
            .await?;

        Ok(())
    }

    pub async fn get_top_chat_pigs(
        &self,
        id_chat: i64,
        min: i32,
        offset_multiplier: i64,
        with_chart: bool,
    ) -> MyResult<Vec<Game>> {
        use crate::db::schema::game::dsl::*;
        use crate::db::schema::groups;

        let top_limit =
            if with_chart { TOP_LIMIT_WITH_CHARTS } else { TOP_LIMIT };

        let results = game
            .filter(groups::chat_id.eq(id_chat))
            .filter(mass.gt(min))
            .order(mass.desc())
            .limit(top_limit)
            .offset(top_limit * offset_multiplier)
            .select(Game::as_select())
            .inner_join(groups::table)
            .load(&mut self.pool.get().await?)
            .await?;

        Ok(results)
    }

    pub async fn count_active_chats_by_uid(&self, id_uid: i32) -> MyResult<i64> {
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            count: i64,
        }

        let row: Row = diesel::sql_query(
            "SELECT COUNT(*) AS count \
             FROM game \
             WHERE uid = $1 \
             AND group_id IN \
               (SELECT group_id FROM game GROUP BY group_id HAVING COUNT(*) >= 4)",
        )
        .bind::<diesel::sql_types::Integer, _>(id_uid)
        .get_result(&mut self.pool.get().await?)
        .await?;

        Ok(row.count)
    }

    pub async fn count_chat_pig(
        &self,
        id_chat: i64,
        min: i32,
    ) -> MyResult<i64> {
        use crate::db::schema::game::dsl::*;
        use crate::db::schema::groups;

        let results = game
            .filter(groups::chat_id.eq(id_chat))
            .filter(mass.gt(min))
            .count()
            .inner_join(groups::table)
            .first(&mut self.pool.get().await?)
            .await?;

        Ok(results)
    }

    #[allow(unused)]
    pub async fn get_grow_log_by_game(
        &self,
        id_game: i32,
    ) -> MyResult<Vec<GrowLog>> {
        use crate::db::schema::grow_log::dsl::*;

        let results = grow_log
            .filter(game_id.eq(id_game))
            .load(&mut self.pool.get().await?)
            .await?;

        Ok(results)
    }

    pub async fn get_grow_log_by_game_14days(
        &self,
        id_game: i32,
    ) -> MyResult<Vec<GrowLog>> {
        use crate::db::schema::grow_log::dsl::*;

        let today = get_datetime();
        let start_date = today - Duration::days(13);

        let results = grow_log
            .filter(game_id.eq(id_game))
            .filter(created_at.ge(start_date).and(created_at.le(today)))
            .load(&mut self.pool.get().await?)
            .await?;

        Ok(results)
    }

    pub async fn add_grow_log_by_game(
        &self,
        about_grow: GrowLogAdd,
    ) -> MyResult<()> {
        use crate::db::schema::grow_log::dsl::*;

        diesel::insert_into(grow_log)
            .values(about_grow)
            .execute(&mut self.pool.get().await?)
            .await?;

        Ok(())
    }

    pub async fn get_game_users_by_group(
        &self,
        group_id_val: i32,
    ) -> MyResult<Vec<(Game, User)>> {
        use crate::db::schema::game::dsl::*;
        use crate::db::schema::users;

        let results = game
            .inner_join(users::table)
            .filter(group_id.eq(group_id_val))
            .select((Game::as_select(), User::as_select()))
            .load(&mut self.pool.get().await?)
            .await?;

        Ok(results)
    }

    pub async fn get_pig_user_ids_by_group(
        &self,
        group_id_val: i32,
    ) -> MyResult<Vec<i64>> {
        use crate::db::schema::game::dsl::*;
        use crate::db::schema::users;

        let results = game
            .inner_join(users::table)
            .filter(group_id.eq(group_id_val))
            .select(users::user_id)
            .load(&mut self.pool.get().await?)
            .await?;

        Ok(results)
    }

    pub async fn count_active_pigs(&self, group_id_val: i32) -> MyResult<i64> {
        use crate::db::schema::game::dsl::*;

        let result = game
            .filter(group_id.eq(group_id_val))
            .count()
            .first(&mut self.pool.get().await?)
            .await?;

        Ok(result)
    }

    pub async fn soft_reset_pigs(&self, group_id_val: i32) -> MyResult<()> {
        use crate::db::schema::achievements_users::dsl::*;
        use crate::db::schema::game;

        let conn = &mut self.pool.get().await?;
        let today = get_date();

        diesel::update(game::table)
            .set((game::mass.eq(CHAT_PIG_START_MASS), game::date.eq(today)))
            .filter(game::group_id.eq(group_id_val))
            .execute(conn)
            .await?;

        diesel::delete(achievements_users)
            .filter(
                game_id.eq_any(
                    game::table
                        .select(game::id)
                        .filter(game::group_id.eq(group_id_val)),
                ),
            )
            .execute(conn)
            .await?;

        Ok(())
    }

    pub async fn get_top10_by_14days_growth(
        &self,
        id_chat: i64,
    ) -> MyResult<Vec<(Game, Vec<GrowLog>)>> {
        use crate::db::schema::game::dsl::*;
        use crate::db::schema::groups;
        use crate::db::schema::grow_log::dsl::*;

        let pool = &mut self.pool.get().await?;
        let today = get_datetime();
        let start_date = today - Duration::days(13);

        let top_users = game
            .filter(groups::chat_id.eq(id_chat))
            .order_by(mass.desc())
            .limit(10)
            .select(Game::as_select())
            .inner_join(groups::table)
            .load(pool)
            .await?;

        let top_user_ids: Vec<_> = top_users.iter().map(|g| g.id).collect();

        let grow_logs: Vec<GrowLog> = grow_log
            .filter(game_id.eq_any(&top_user_ids))
            .filter(created_at.ge(start_date).and(created_at.le(today)))
            .load(pool)
            .await?;

        let mut logs_by_game: AHashMap<_, Vec<_>> = AHashMap::default();

        for log in grow_logs {
            logs_by_game.entry(log.game_id).or_default().push(log);
        }

        let result: Vec<_> = top_users
            .into_iter()
            .map(|v| {
                let logs = logs_by_game.remove(&v.id).unwrap_or_default();
                (v, logs)
            })
            .collect();

        Ok(result)
    }
}
