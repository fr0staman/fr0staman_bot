use std::cmp::Ordering;

use chrono::{Datelike, Duration, NaiveDateTime, Timelike};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use strum::{EnumCount, IntoStaticStr, VariantArray};

use crate::{
    db::{DB, models::AchievementUserAdd},
    types::MyResult,
};

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

    // Cyclic
    FeederOfTheYear = 301,
    SchrodingerPig = 302,
    EmployeeOfTheMonth = 303,
    SevenFridays = 304,
    Pendulum = 305,
    GroundhogDay = 306,
    NoChangeThreeDays = 307,

    // Special
    InfinityWar = 401,
    EternalGenin = 402,
    NewYearPig = 403,
    //PigOfTheDay = 404,

    // Date or time
    ZeroHour = 501,
    Agent007 = 502,
    NewHope = 503,
}

pub async fn check_achievements(
    id_game: i32,
    message_time: NaiveDateTime,
) -> MyResult<Vec<Ach>> {
    let Some(chat_pig) = DB.chat_pig.get_chat_pig_by_id(id_game).await? else {
        return Ok(vec![]);
    };

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
    push_if(&mut new, infinity_war, Ach::InfinityWar, &achieved);
    push_if(&mut new, eternal_genin, Ach::EternalGenin, &achieved);
    push_if(&mut new, new_year_pig, Ach::NewYearPig, &achieved);
    push_if(&mut new, zero_hour, Ach::ZeroHour, &achieved);
    push_if(&mut new, agent_007, Ach::Agent007, &achieved);
    push_if(&mut new, new_hope, Ach::NewHope, &achieved);

    for new_achievement in &new {
        let new_achievement = AchievementUserAdd {
            game_id: chat_pig.id,
            created_at: now,
            code: new_achievement.clone() as i16,
        };
        DB.other.add_achievement(new_achievement).await?;
    }

    Ok(new)
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
