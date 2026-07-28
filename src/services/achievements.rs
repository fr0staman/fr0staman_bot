use std::cmp::Ordering;

use chrono::{Datelike, Duration, NaiveDateTime, Timelike};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use strum::{EnumCount, IntoStaticStr, VariantArray};

use crate::{
    db::{
        DB,
        models::{AchievementUserAdd, Game},
    },
    types::MyResult,
};

/// The pig fields the checks below actually look at.
///
/// Callers reach here already holding the `Game` — `/grow` even knows the mass
/// it just wrote — so they hand it over instead of making this re-read the row.
#[derive(Clone, Copy)]
pub struct PigSnapshot {
    pub id: i32,
    pub uid: i32,
    pub mass: i32,
}

impl From<&Game> for PigSnapshot {
    fn from(game: &Game) -> Self {
        Self { id: game.id, uid: game.uid, mass: game.mass }
    }
}

#[derive(
    PartialEq, IntoStaticStr, EnumCount, VariantArray, FromPrimitive, Clone,
)]
#[strum(const_into_str, serialize_all = "snake_case")]
pub enum Ach {
    // Simple
    FirstLoss = 101,
    KamaSutra = 102,
    Rollercoaster = 103,
    MonsterGrow = 104,

    // Numbers
    ElectricGrandpa = 201,
    YearWeight = 202,
    HundredClub = 203,
    FiveMetersOfFat = 204,
    TonOfPig = 205,
    Jackpot = 206,
    DemonPig = 207,
    AdultPig = 208,
    PlanetPig = 209,

    // Cyclic
    FeederOfTheYear = 301,
    SchrodingerPig = 302,
    EmployeeOfTheMonth = 303,
    SevenFridays = 304,
    Pendulum = 305,
    GroundhogDay = 306,
    NoChangeThreeDays = 307,
    WeeklyDedication = 308,
    Fortnight = 309,
    HungryStreak = 310,

    // Special
    InfinityWar = 401,
    EternalGenin = 402,
    NewYearPig = 403,
    PigOfTheDay = 404,
    Pigolator = 405,

    // Date or time
    ZeroHour = 501,
    Agent007 = 502,
    NewHope = 503,
    LovePig = 504,
    HalloweenPig = 505,
    PeremogaBude = 506,

    // Social
    PigInTwoChats = 601,
    PigEverywhere = 602,
    Hruklid19 = 603,
}

pub async fn check_achievements(
    chat_pig: PigSnapshot,
    message_time: NaiveDateTime,
) -> MyResult<Vec<Ach>> {
    let grow_log = DB.chat_pig.get_grow_log_by_game(chat_pig.id).await?;
    let achieved = DB.other.get_achievements_by_game_id(chat_pig.id).await?;

    let achieved: Vec<_> =
        achieved.iter().filter_map(|v| Ach::from_i16(v.code)).collect();

    let now = message_time;

    let mut new = vec![];

    // 1. "Ой..." — втратити вагу вперше
    let first_loss = || {
        let Some(stats) = grow_log.last() else {
            return false;
        };
        let previous_weight = stats.current_weight - stats.weight_change;
        stats.current_weight < previous_weight
    };

    // 2. "Камасутра" — набрати 69 кг
    let kama_sutra = || chat_pig.mass == 69;

    // 3. "Американські гірки" — за тиждень хоча б 1 раз набрати, 1 раз схуднути і 1 раз без змін
    let rollercoaster = || {
        let last_7_feeds = &grow_log[grow_log.len().saturating_sub(7)..];

        if last_7_feeds.len() != 7 {
            return false;
        }

        if last_7_feeds.first().unwrap().created_at
            != last_7_feeds.last().unwrap().created_at - Duration::days(7 - 1)
        {
            return false;
        };

        // minus, equal, plus
        let mut results: (bool, bool, bool) = (false, false, false);

        for i in last_7_feeds {
            if i.weight_change == 0 {
                results.1 = true;
            } else if i.weight_change > 0 {
                results.2 = true;
            } else {
                results.0 = true;
            }
        }

        results.0 && results.1 && results.2
    };

    // 4. "MONSTER GROW" — отримати максимальний приріст +20 кг
    let monster_grow =
        || grow_log.last().is_some_and(|c| c.weight_change >= 20);

    // Циферні:
    // 5. "Дід був електриком" — набрати 1488 кг
    let electric_grandpa = || chat_pig.mass == 1488;

    // 6. "Набитий рік" — набрати стільки кг, як поточний рік
    let year_weight = || chat_pig.mass == now.year();

    // 7. "Соточка" — набрати 100+ кг
    let hundred_club = || chat_pig.mass >= 100;

    // 8. "5 метрів сала" — набрати 500+ кг
    let five_meters_of_fat = || chat_pig.mass >= 500;

    // 9. "Хрякотонна" — набрати 1000+ кг
    let ton_of_pig = || chat_pig.mass >= 1000;

    // 10. "Джекпот" — набрати 777 кг
    let jackpot = || chat_pig.mass == 777;

    // 11. "Демон" — набрати 666 кг
    let demon_pig = || chat_pig.mass == 666;

    // 12. "Дорослий" — набрати 18 кг
    let adult_pig = || chat_pig.mass == 18;

    // 13. "Мати-Земля" — набрати 5000+ кг
    let planet_pig = || chat_pig.mass >= 5000;

    // 1. "Годувальник року" — 5 разів підряд +20 кг
    let feeder_of_the_year = || {
        grow_log
            .windows(5)
            .last()
            .is_some_and(|days| days.iter().all(|d| d.weight_change >= 20))
    };

    // 2. "Свиня Шрьодінгера" — 3 дні: +, -, 0
    let schrodinger_pig = || {
        grow_log.windows(3).last().is_some_and(|last| {
            let (d1, d2, d3) = (&last[0], &last[1], &last[2]);
            matches!(
                (
                    d1.weight_change.cmp(&0),
                    d2.weight_change.cmp(&0),
                    d3.weight_change.cmp(&0)
                ),
                (Ordering::Greater, Ordering::Less, Ordering::Equal)
                    | (Ordering::Less, Ordering::Greater, Ordering::Equal)
                    | (Ordering::Equal, Ordering::Greater, Ordering::Less)
                    | (Ordering::Equal, Ordering::Less, Ordering::Greater)
            )
        })
    };

    // 3. "Кращий працівник місяця" — годувати 30 днів підряд
    let employee_of_the_month = || {
        let n = 30;

        if grow_log.len() < n {
            return false;
        }

        let last_n = &grow_log[grow_log.len() - n..];

        let first = &last_n[0];
        let last = &last_n[n - 1];

        let correct_range = last.created_at - Duration::days((n - 1) as i64)
            == first.created_at;
        let ends_today = last.created_at == now;

        correct_range && ends_today
    };

    // 4. "7 п'ятниць на тиждень" — кожен день тижня з приростом
    let seven_fridays = || {
        let last_7_days = &grow_log[grow_log.len().saturating_sub(7)..];
        let mut gained_days = [false; 7];

        if last_7_days.len() != 7 {
            return false;
        }

        let [first, .., last] = &last_7_days else { todo!() };

        let is_calendar_week =
            first.created_at.weekday().num_days_from_monday() == 0
                && first.created_at == last.created_at - Duration::days(6);

        if !is_calendar_week {
            return false;
        }

        for d in last_7_days {
            if d.weight_change > 0 {
                gained_days
                    [d.created_at.weekday().num_days_from_monday() as usize] =
                    true;
            }
        }

        gained_days.iter().all(|&v| v)
    };

    // 5. "Маятник" — +20 кг і -20 кг за 2 дні
    let pendulum = || {
        grow_log.windows(2).last().is_some_and(|w| {
            (w[0].weight_change >= 20 && w[1].weight_change <= -20)
                || (w[0].weight_change <= -20 && w[1].weight_change >= 20)
        })
    };

    // 6. "День Бабака" — 3 дні поспіль втрата
    let groundhog_day = || {
        grow_log
            .windows(3)
            .last()
            .is_some_and(|last| last.iter().all(|d| d.weight_change < 0))
    };

    // 7. "Годував, але не допомогло" — 3 дні без змін
    let no_change_three_days = || {
        grow_log
            .windows(3)
            .last()
            .is_some_and(|last| last.iter().all(|v| v.weight_change == 0))
    };

    // 8. "Тижнева відданість" — 7 днів підряд
    let weekly_dedication = || {
        let n = 7usize;
        if grow_log.len() < n {
            return false;
        }
        let last_n = &grow_log[grow_log.len() - n..];
        let first_date = last_n[0].created_at.date();
        let last_date = last_n[n - 1].created_at.date();
        last_date == now.date()
            && last_date - first_date == Duration::days((n - 1) as i64)
    };

    // 9. "Два тижні" — 14 днів підряд
    let fortnight = || {
        let n = 14usize;
        if grow_log.len() < n {
            return false;
        }
        let last_n = &grow_log[grow_log.len() - n..];
        let first_date = last_n[0].created_at.date();
        let last_date = last_n[n - 1].created_at.date();
        last_date == now.date()
            && last_date - first_date == Duration::days((n - 1) as i64)
    };

    // 10. "Голодний стрік" — 5 днів підряд будь-який приріст
    let hungry_streak = || {
        grow_log
            .windows(5)
            .last()
            .is_some_and(|days| days.iter().all(|d| d.weight_change > 0))
    };

    // 8. "Війна Хрюконечності" — схуд до 1 кг і останній дельта -20
    let infinity_war = || {
        grow_log
            .last()
            .is_some_and(|d| d.current_weight == 1 && d.weight_change <= -20)
    };

    // 9. "Вічний Генін" — 7 днів поспіль у межах 0–10 кг
    let eternal_genin = || {
        grow_log.windows(7).last().is_some_and(|last_week| {
            last_week.iter().all(|d| d.current_weight <= 10)
        })
    };

    // 10. "Свиня у вас минулорічна" — годувати 31.12 і 01.01
    let new_year_pig = || {
        grow_log.windows(2).last().is_some_and(|last| {
            let (d1, d2) = (&last[0], &last[1]);
            d1.created_at.month() == 12
                && d1.created_at.day() == 31
                && d2.created_at.month() == 1
                && d2.created_at.day() == 1
        })
    };

    // 1. "Тут як тут" — погодувати в 00:00
    let zero_hour = || now.hour() == 0 && now.minute() == 0;

    // 2. "Агент 007" — 7 місяця 7 числа о 7:00
    let agent_007 =
        || now.month() == 7 && now.day() == 7 && now.time().hour() == 7;

    // 3. "Нова надія" — погодувати 1 числа
    let new_hope = || now.day() == 1;

    // 4. "Свиня кохання" — погодувати 14 лютого
    let love_pig = || now.month() == 2 && now.day() == 14;

    // 5. "Хеловін" — погодувати 31 жовтня
    let halloween_pig = || now.month() == 10 && now.day() == 31;

    // 6. "Перемога буде" — погодувати 15 травня
    let peremoga_bude = || now.month() == 5 && now.day() == 15;

    // Try to economy compute, in future database requests
    push_if(&mut new, first_loss, Ach::FirstLoss, &achieved);
    push_if(&mut new, kama_sutra, Ach::KamaSutra, &achieved);
    push_if(&mut new, monster_grow, Ach::MonsterGrow, &achieved);
    push_if(&mut new, electric_grandpa, Ach::ElectricGrandpa, &achieved);
    push_if(&mut new, year_weight, Ach::YearWeight, &achieved);
    push_if(&mut new, hundred_club, Ach::HundredClub, &achieved);
    push_if(&mut new, five_meters_of_fat, Ach::FiveMetersOfFat, &achieved);
    push_if(&mut new, ton_of_pig, Ach::TonOfPig, &achieved);
    push_if(&mut new, jackpot, Ach::Jackpot, &achieved);
    push_if(&mut new, demon_pig, Ach::DemonPig, &achieved);
    push_if(&mut new, adult_pig, Ach::AdultPig, &achieved);
    push_if(&mut new, planet_pig, Ach::PlanetPig, &achieved);
    push_if(&mut new, rollercoaster, Ach::Rollercoaster, &achieved);
    push_if(&mut new, feeder_of_the_year, Ach::FeederOfTheYear, &achieved);
    push_if(&mut new, schrodinger_pig, Ach::SchrodingerPig, &achieved);
    push_if(
        &mut new,
        employee_of_the_month,
        Ach::EmployeeOfTheMonth,
        &achieved,
    );
    push_if(&mut new, seven_fridays, Ach::SevenFridays, &achieved);
    push_if(&mut new, pendulum, Ach::Pendulum, &achieved);
    push_if(&mut new, groundhog_day, Ach::GroundhogDay, &achieved);
    push_if(&mut new, no_change_three_days, Ach::NoChangeThreeDays, &achieved);
    push_if(&mut new, weekly_dedication, Ach::WeeklyDedication, &achieved);
    push_if(&mut new, fortnight, Ach::Fortnight, &achieved);
    push_if(&mut new, hungry_streak, Ach::HungryStreak, &achieved);
    push_if(&mut new, infinity_war, Ach::InfinityWar, &achieved);
    push_if(&mut new, eternal_genin, Ach::EternalGenin, &achieved);
    push_if(&mut new, new_year_pig, Ach::NewYearPig, &achieved);
    push_if(&mut new, zero_hour, Ach::ZeroHour, &achieved);
    push_if(&mut new, agent_007, Ach::Agent007, &achieved);
    push_if(&mut new, new_hope, Ach::NewHope, &achieved);
    push_if(&mut new, love_pig, Ach::LovePig, &achieved);
    push_if(&mut new, halloween_pig, Ach::HalloweenPig, &achieved);
    push_if(&mut new, peremoga_bude, Ach::PeremogaBude, &achieved);

    if !achieved.contains(&Ach::PigInTwoChats)
        || !achieved.contains(&Ach::PigEverywhere)
        || !achieved.contains(&Ach::Hruklid19)
    {
        let chat_count =
            DB.chat_pig.count_active_chats_by_uid(chat_pig.uid).await?;
        if !achieved.contains(&Ach::PigInTwoChats) && chat_count >= 2 {
            new.push(Ach::PigInTwoChats);
        }
        if !achieved.contains(&Ach::PigEverywhere) && chat_count >= 5 {
            new.push(Ach::PigEverywhere);
        }
        if !achieved.contains(&Ach::Hruklid19) && chat_count >= 10 {
            new.push(Ach::Hruklid19);
        }
    }

    let to_insert: Vec<_> = new
        .iter()
        .map(|new_achievement| AchievementUserAdd {
            game_id: chat_pig.id,
            created_at: now,
            code: new_achievement.clone() as i16,
        })
        .collect();

    DB.other.add_achievements(&to_insert).await?;

    Ok(new)
}

pub async fn check_name_achievements(id_game: i32) -> MyResult<Vec<Ach>> {
    use crate::utils::date::get_datetime;

    let achieved = DB.other.get_achievements_by_game_id(id_game).await?;
    let achieved: Vec<_> =
        achieved.iter().filter_map(|v| Ach::from_i16(v.code)).collect();

    if achieved.contains(&Ach::Pigolator) {
        return Ok(vec![]);
    }

    DB.other
        .add_achievements(&[AchievementUserAdd {
            game_id: id_game,
            created_at: get_datetime(),
            code: Ach::Pigolator as i16,
        }])
        .await?;

    Ok(vec![Ach::Pigolator])
}

pub async fn check_day_pig_achievement(id_game: i32) -> MyResult<Vec<Ach>> {
    use crate::utils::date::get_datetime;

    let achieved = DB.other.get_achievements_by_game_id(id_game).await?;
    let achieved: Vec<_> =
        achieved.iter().filter_map(|v| Ach::from_i16(v.code)).collect();

    if achieved.contains(&Ach::PigOfTheDay) {
        return Ok(vec![]);
    }

    DB.other
        .add_achievements(&[AchievementUserAdd {
            game_id: id_game,
            created_at: get_datetime(),
            code: Ach::PigOfTheDay as i16,
        }])
        .await?;

    Ok(vec![Ach::PigOfTheDay])
}

fn push_if<F: FnOnce() -> bool>(
    result: &mut Vec<Ach>,
    check: F,
    id: Ach,
    already_new: &[Ach],
) {
    if !already_new.contains(&id) && check() {
        result.push(id);
    }
}
