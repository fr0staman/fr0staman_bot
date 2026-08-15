use std::cmp::Ordering;

use chrono::{Datelike, Duration, NaiveDateTime, Timelike};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use strum::{EnumCount, IntoStaticStr, VariantArray};

use crate::{
    db::{
        DB,
        models::{AchievementUserAdd, Game, GrowLog},
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
    PartialEq,
    IntoStaticStr,
    EnumCount,
    VariantArray,
    FromPrimitive,
    Clone,
    Copy,
)]
#[cfg_attr(test, derive(Debug, Eq, Hash))]
#[strum(const_into_str, serialize_all = "snake_case")]
pub enum Ach {
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

    let mut new =
        evaluate_achievements(chat_pig, &grow_log, &achieved, message_time);

    // Skips a second query once all three are unlocked.
    if needs_chat_count(&achieved) {
        let chat_count =
            DB.chat_pig.count_active_chats_by_uid(chat_pig.uid).await?;
        new.extend(evaluate_social_achievements(chat_count, &achieved));
    }

    let to_insert: Vec<_> = new
        .iter()
        .map(|new_achievement| AchievementUserAdd {
            game_id: chat_pig.id,
            created_at: message_time,
            code: *new_achievement as i16,
        })
        .collect();

    DB.other.add_achievements(&to_insert).await?;

    Ok(new)
}

/// Everything decidable from the pig's own feed history. The social ones
/// need a separate count — see [`evaluate_social_achievements`].
pub fn evaluate_achievements(
    chat_pig: PigSnapshot,
    grow_log: &[GrowLog],
    achieved: &[Ach],
    now: NaiveDateTime,
) -> Vec<Ach> {
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

        // Dates, not timestamps — feeds land at arbitrary times of day.
        if last_7_feeds.first().unwrap().created_at.date()
            != last_7_feeds.last().unwrap().created_at.date()
                - Duration::days(7 - 1)
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
                // All six orderings of one gain, one loss, one no-change.
                (Ordering::Greater, Ordering::Less, Ordering::Equal)
                    | (Ordering::Less, Ordering::Greater, Ordering::Equal)
                    | (Ordering::Equal, Ordering::Greater, Ordering::Less)
                    | (Ordering::Equal, Ordering::Less, Ordering::Greater)
                    | (Ordering::Greater, Ordering::Equal, Ordering::Less)
                    | (Ordering::Less, Ordering::Equal, Ordering::Greater)
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

        // Dates, not timestamps: `created_at` and `now` come from different
        // clocks and never match exactly.
        let correct_range = last.created_at.date()
            - Duration::days((n - 1) as i64)
            == first.created_at.date();
        let ends_today = last.created_at.date() == now.date();

        correct_range && ends_today
    };

    // 4. "7 п'ятниць на тиждень" — кожен день тижня з приростом
    let seven_fridays = || {
        let last_7_days = &grow_log[grow_log.len().saturating_sub(7)..];
        let mut gained_days = [false; 7];

        if last_7_days.len() != 7 {
            return false;
        }

        // Unreachable, but a panic here would kill the `/grow` reply.
        let [first, .., last] = &last_7_days else { return false };

        let is_calendar_week =
            first.created_at.weekday().num_days_from_monday() == 0
                && first.created_at.date()
                    == last.created_at.date() - Duration::days(6);

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
    push_if(&mut new, first_loss, Ach::FirstLoss, achieved);
    push_if(&mut new, kama_sutra, Ach::KamaSutra, achieved);
    push_if(&mut new, monster_grow, Ach::MonsterGrow, achieved);
    push_if(&mut new, electric_grandpa, Ach::ElectricGrandpa, achieved);
    push_if(&mut new, year_weight, Ach::YearWeight, achieved);
    push_if(&mut new, hundred_club, Ach::HundredClub, achieved);
    push_if(&mut new, five_meters_of_fat, Ach::FiveMetersOfFat, achieved);
    push_if(&mut new, ton_of_pig, Ach::TonOfPig, achieved);
    push_if(&mut new, jackpot, Ach::Jackpot, achieved);
    push_if(&mut new, demon_pig, Ach::DemonPig, achieved);
    push_if(&mut new, adult_pig, Ach::AdultPig, achieved);
    push_if(&mut new, planet_pig, Ach::PlanetPig, achieved);
    push_if(&mut new, rollercoaster, Ach::Rollercoaster, achieved);
    push_if(&mut new, feeder_of_the_year, Ach::FeederOfTheYear, achieved);
    push_if(&mut new, schrodinger_pig, Ach::SchrodingerPig, achieved);
    push_if(
        &mut new,
        employee_of_the_month,
        Ach::EmployeeOfTheMonth,
        achieved,
    );
    push_if(&mut new, seven_fridays, Ach::SevenFridays, achieved);
    push_if(&mut new, pendulum, Ach::Pendulum, achieved);
    push_if(&mut new, groundhog_day, Ach::GroundhogDay, achieved);
    push_if(&mut new, no_change_three_days, Ach::NoChangeThreeDays, achieved);
    push_if(&mut new, weekly_dedication, Ach::WeeklyDedication, achieved);
    push_if(&mut new, fortnight, Ach::Fortnight, achieved);
    push_if(&mut new, hungry_streak, Ach::HungryStreak, achieved);
    push_if(&mut new, infinity_war, Ach::InfinityWar, achieved);
    push_if(&mut new, eternal_genin, Ach::EternalGenin, achieved);
    push_if(&mut new, new_year_pig, Ach::NewYearPig, achieved);
    push_if(&mut new, zero_hour, Ach::ZeroHour, achieved);
    push_if(&mut new, agent_007, Ach::Agent007, achieved);
    push_if(&mut new, new_hope, Ach::NewHope, achieved);
    push_if(&mut new, love_pig, Ach::LovePig, achieved);
    push_if(&mut new, halloween_pig, Ach::HalloweenPig, achieved);
    push_if(&mut new, peremoga_bude, Ach::PeremogaBude, achieved);
    new
}

/// Whether [`evaluate_social_achievements`] still has anything to award.
pub fn needs_chat_count(achieved: &[Ach]) -> bool {
    !achieved.contains(&Ach::PigInTwoChats)
        || !achieved.contains(&Ach::PigEverywhere)
        || !achieved.contains(&Ach::Hruklid19)
}

/// Cross-chat achievements, from a count of qualifying chats.
pub fn evaluate_social_achievements(
    chat_count: i64,
    achieved: &[Ach],
) -> Vec<Ach> {
    let mut new = vec![];

    if !achieved.contains(&Ach::PigInTwoChats) && chat_count >= 2 {
        new.push(Ach::PigInTwoChats);
    }
    if !achieved.contains(&Ach::PigEverywhere) && chat_count >= 5 {
        new.push(Ach::PigEverywhere);
    }
    if !achieved.contains(&Ach::Hruklid19) && chat_count >= 10 {
        new.push(Ach::Hruklid19);
    }

    new
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{daily_grow_log, datetime, game};

    const NOW: fn() -> NaiveDateTime = || datetime(2026, 7, 28, 15, 30);

    fn pig(mass: i32) -> PigSnapshot {
        PigSnapshot::from(&game(mass))
    }

    /// Runs the evaluator with nothing unlocked yet.
    fn eval(
        mass: i32,
        log: &[GrowLog],
        now: NaiveDateTime,
    ) -> Vec<Ach> {
        evaluate_achievements(pig(mass), log, &[], now)
    }

    fn has(mass: i32, log: &[GrowLog], now: NaiveDateTime, ach: Ach) -> bool {
        eval(mass, log, now).contains(&ach)
    }

    /// `n` consecutive daily feeds, all +1, ending on `now`.
    fn streak(now: NaiveDateTime, n: usize) -> Vec<GrowLog> {
        daily_grow_log(now, 1, &vec![1; n])
    }


    #[test]
    fn an_empty_history_unlocks_nothing_history_based() {
        let now = datetime(2026, 7, 15, 12, 0);
        let unlocked = eval(5, &[], now);

        for ach in [
            Ach::FirstLoss,
            Ach::Rollercoaster,
            Ach::MonsterGrow,
            Ach::FeederOfTheYear,
            Ach::SchrodingerPig,
            Ach::GroundhogDay,
            Ach::HungryStreak,
            Ach::InfinityWar,
            Ach::EternalGenin,
            Ach::NewYearPig,
        ] {
            assert!(!unlocked.contains(&ach), "{ach:?} fired on empty history");
        }
    }

    #[test]
    fn already_unlocked_achievements_are_not_repeated() {
        let now = NOW();
        let log = daily_grow_log(now, 1, &[1, -1]);

        assert!(eval(100, &log, now).contains(&Ach::HundredClub));

        let again =
            evaluate_achievements(pig(100), &log, &[Ach::HundredClub], now);
        assert!(!again.contains(&Ach::HundredClub));
    }

    #[test]
    fn every_ach_code_round_trips_through_from_i16() {
        use strum::VariantArray;

        for ach in Ach::VARIANTS {
            let code = *ach as i16;
            assert_eq!(
                Ach::from_i16(code),
                Some(*ach),
                "{ach:?} (code {code})"
            );
        }
    }

    #[test]
    fn achievement_codes_are_unique() {
        use strum::VariantArray;

        let mut codes: Vec<i16> =
            Ach::VARIANTS.iter().map(|a| *a as i16).collect();
        codes.sort_unstable();
        let before = codes.len();
        codes.dedup();

        assert_eq!(before, codes.len(), "duplicate achievement code");
    }

    #[test]
    fn the_achievement_count_matches_what_the_docs_claim() {
        // The denominator `/achievements` prints, so the docs must track it.
        assert_eq!(Ach::COUNT, 37);

        // Per-category breakdown.
        let by_category = 4 + 9 + 10 + 5 + 6 + 3;
        assert_eq!(Ach::COUNT, by_category);
    }


    #[test]
    fn first_loss_fires_on_a_negative_last_feed() {
        let now = NOW();

        assert!(has(9, &daily_grow_log(now, 10, &[1, -2]), now, Ach::FirstLoss));
        assert!(!has(
            12,
            &daily_grow_log(now, 10, &[-2, 4]),
            now,
            Ach::FirstLoss
        ));
        assert!(!has(10, &daily_grow_log(now, 10, &[0]), now, Ach::FirstLoss));
    }

    #[test]
    fn kama_sutra_needs_exactly_sixty_nine_kg() {
        let now = NOW();
        assert!(has(69, &[], now, Ach::KamaSutra));
        assert!(!has(68, &[], now, Ach::KamaSutra));
        assert!(!has(70, &[], now, Ach::KamaSutra));
    }

    #[test]
    fn monster_grow_needs_the_maximum_roll() {
        let now = NOW();
        assert!(has(30, &daily_grow_log(now, 10, &[20]), now, Ach::MonsterGrow));
        assert!(!has(
            29,
            &daily_grow_log(now, 10, &[19]),
            now,
            Ach::MonsterGrow
        ));
    }

    #[test]
    fn rollercoaster_needs_a_gain_a_loss_and_a_no_change_in_seven_feeds() {
        let now = NOW();

        let mixed = daily_grow_log(now, 50, &[1, -1, 0, 1, 1, 1, 1]);
        assert!(has(54, &mixed, now, Ach::Rollercoaster));
        let no_zero = daily_grow_log(now, 50, &[1, -1, 1, 1, 1, 1, 1]);
        assert!(!has(55, &no_zero, now, Ach::Rollercoaster));
        let short = daily_grow_log(now, 50, &[1, -1, 0]);
        assert!(!has(50, &short, now, Ach::Rollercoaster));
    }

    #[test]
    fn rollercoaster_tolerates_feeds_at_different_times_of_day() {
        // The window is checked on calendar dates. It used to compare full
        // timestamps, which meant real users — who feed whenever they like —
        // could never unlock it.
        let now = NOW();

        let mut log = daily_grow_log(now, 50, &[1, -1, 0, 1, 1, 1, 1]);
        assert!(has(54, &log, now, Ach::Rollercoaster));
        log[0].created_at += Duration::minutes(1);
        log[2].created_at -= Duration::hours(6);
        log[5].created_at += Duration::hours(7);
        assert!(has(54, &log, now, Ach::Rollercoaster));
    }

    #[test]
    fn rollercoaster_still_needs_seven_consecutive_days() {
        let now = NOW();

        let mut log = daily_grow_log(now, 50, &[1, -1, 0, 1, 1, 1, 1]);
        log[0].created_at -= Duration::days(1);

        assert!(!has(54, &log, now, Ach::Rollercoaster));
    }


    #[test]
    fn numeric_thresholds_match_the_spec() {
        let now = datetime(2026, 6, 10, 12, 0);
        assert!(has(1488, &[], now, Ach::ElectricGrandpa));
        assert!(!has(1487, &[], now, Ach::ElectricGrandpa));
        assert!(has(777, &[], now, Ach::Jackpot));
        assert!(!has(778, &[], now, Ach::Jackpot));
        assert!(has(666, &[], now, Ach::DemonPig));
        assert!(has(18, &[], now, Ach::AdultPig));

        // The year the feed happened in, not a constant.
        assert!(has(2026, &[], now, Ach::YearWeight));
        assert!(!has(2026, &[], datetime(2027, 1, 1, 12, 0), Ach::YearWeight));
        assert!(has(2027, &[], datetime(2027, 1, 1, 12, 0), Ach::YearWeight));
        for (mass, ach, fires) in [
            (99, Ach::HundredClub, false),
            (100, Ach::HundredClub, true),
            (5_000, Ach::HundredClub, true),
            (499, Ach::FiveMetersOfFat, false),
            (500, Ach::FiveMetersOfFat, true),
            (999, Ach::TonOfPig, false),
            (1_000, Ach::TonOfPig, true),
            (4_999, Ach::PlanetPig, false),
            (5_000, Ach::PlanetPig, true),
        ] {
            assert_eq!(has(mass, &[], now, ach), fires, "{mass} {ach:?}");
        }
    }


    #[test]
    fn feeder_of_the_year_needs_five_maximum_rolls() {
        let now = NOW();

        assert!(has(
            110,
            &daily_grow_log(now, 10, &[20, 20, 20, 20, 20]),
            now,
            Ach::FeederOfTheYear
        ));
        assert!(!has(
            109,
            &daily_grow_log(now, 10, &[20, 20, 20, 20, 19]),
            now,
            Ach::FeederOfTheYear
        ));
        assert!(has(
            111,
            &daily_grow_log(now, 10, &[1, 20, 20, 20, 20, 20]),
            now,
            Ach::FeederOfTheYear
        ));
    }

    #[test]
    fn schrodinger_pig_matches_all_six_orderings() {
        // Including the two with the flat day in the middle, which the
        // original `matches!` arms left out.
        let now = NOW();

        for deltas in [
            [5, -5, 0],
            [-5, 5, 0],
            [0, 5, -5],
            [0, -5, 5],
            [5, 0, -5],
            [-5, 0, 5],
        ] {
            let log = daily_grow_log(now, 100, &deltas);
            assert!(
                has(100, &log, now, Ach::SchrodingerPig),
                "{deltas:?} should fire"
            );
        }
    }

    #[test]
    fn schrodinger_pig_needs_one_of_each_kind_of_day() {
        let now = NOW();
        for deltas in [[5, -5, 5], [5, 5, -5], [0, 0, 5], [1, 1, 1], [0, 0, 0]]
        {
            let log = daily_grow_log(now, 100, &deltas);
            assert!(
                !has(100, &log, now, Ach::SchrodingerPig),
                "{deltas:?} should not fire"
            );
        }
    }

    #[test]
    fn pendulum_needs_a_swing_of_twenty_in_both_directions() {
        let now = NOW();

        assert!(has(
            10,
            &daily_grow_log(now, 10, &[20, -20]),
            now,
            Ach::Pendulum
        ));
        assert!(has(
            10,
            &daily_grow_log(now, 10, &[-20, 20]),
            now,
            Ach::Pendulum
        ));
        assert!(!has(
            11,
            &daily_grow_log(now, 10, &[20, -19]),
            now,
            Ach::Pendulum
        ));
    }

    #[test]
    fn groundhog_day_needs_three_straight_losses() {
        let now = NOW();

        assert!(has(
            97,
            &daily_grow_log(now, 100, &[-1, -1, -1]),
            now,
            Ach::GroundhogDay
        ));
        assert!(!has(
            98,
            &daily_grow_log(now, 100, &[-1, -1, 0]),
            now,
            Ach::GroundhogDay
        ));
    }

    #[test]
    fn no_change_three_days_needs_three_flat_feeds() {
        let now = NOW();

        assert!(has(
            100,
            &daily_grow_log(now, 100, &[0, 0, 0]),
            now,
            Ach::NoChangeThreeDays
        ));
        assert!(!has(
            101,
            &daily_grow_log(now, 100, &[0, 0, 1]),
            now,
            Ach::NoChangeThreeDays
        ));
    }

    #[test]
    fn hungry_streak_needs_five_straight_gains() {
        let now = NOW();

        assert!(has(105, &streak(now, 5), now, Ach::HungryStreak));
        assert!(!has(
            104,
            &daily_grow_log(now, 100, &[1, 1, 1, 1, 0]),
            now,
            Ach::HungryStreak
        ));
    }

    #[test]
    fn weekly_dedication_and_fortnight_need_a_run_ending_today() {
        let now = NOW();

        assert!(has(8, &streak(now, 7), now, Ach::WeeklyDedication));
        assert!(!has(7, &streak(now, 6), now, Ach::WeeklyDedication));

        assert!(has(15, &streak(now, 14), now, Ach::Fortnight));
        assert!(!has(14, &streak(now, 13), now, Ach::Fortnight));
    }

    #[test]
    fn weekly_dedication_compares_dates_not_timestamps() {
        // Unlike `rollercoaster`, this one uses `.date()`, so feeds at
        // different times of day still count.
        let now = NOW();
        let mut log = streak(now, 7);
        log[3].created_at += Duration::hours(5);

        assert!(has(8, &log, now, Ach::WeeklyDedication));
    }

    #[test]
    fn weekly_dedication_needs_the_run_to_end_today() {
        let now = NOW();
        let yesterday = now - Duration::days(1);

        assert!(!has(8, &streak(yesterday, 7), now, Ach::WeeklyDedication));
    }

    #[test]
    fn a_gap_in_the_run_breaks_weekly_dedication() {
        let now = NOW();
        let mut log = streak(now, 7);
        // Push the oldest feed a day further back: still 7 rows, but the
        // span is now 8 days.
        log[0].created_at -= Duration::days(1);

        assert!(!has(8, &log, now, Ach::WeeklyDedication));
    }

    #[test]
    fn employee_of_the_month_needs_thirty_consecutive_days_ending_today() {
        let now = NOW();

        assert!(has(31, &streak(now, 30), now, Ach::EmployeeOfTheMonth));
        assert!(!has(30, &streak(now, 29), now, Ach::EmployeeOfTheMonth));
        let yesterday = now - Duration::days(1);
        assert!(!has(31, &streak(yesterday, 30), now, Ach::EmployeeOfTheMonth));
    }

    #[test]
    fn employee_of_the_month_tolerates_clock_skew() {
        // `created_at` is written with `get_datetime()` while `now` is the
        // message time; comparing the two for exact equality meant this
        // could never fire. Dates are compared instead.
        let now = NOW();
        let mut log = streak(now, 30);

        let last = log.len() - 1;
        log[last].created_at += Duration::seconds(1);
        assert!(has(31, &log, now, Ach::EmployeeOfTheMonth));

        log[last].created_at -= Duration::hours(9);
        log[0].created_at += Duration::hours(3);
        assert!(has(31, &log, now, Ach::EmployeeOfTheMonth));
    }

    #[test]
    fn employee_of_the_month_needs_the_run_to_be_unbroken() {
        let now = NOW();
        let mut log = streak(now, 30);
        log[0].created_at -= Duration::days(1);

        assert!(!has(31, &log, now, Ach::EmployeeOfTheMonth));
    }

    #[test]
    fn seven_fridays_needs_a_monday_to_sunday_week_of_gains() {
        // 2026-07-26 is a Sunday, so the week runs Mon 2026-07-20..Sun 26.
        let sunday = datetime(2026, 7, 26, 12, 0);
        assert_eq!(sunday.weekday().num_days_from_monday(), 6);

        let log = daily_grow_log(sunday, 1, &[1; 7]);
        assert!(has(8, &log, sunday, Ach::SevenFridays));

        // A single flat day breaks it.
        let mut with_gap = log.clone();
        with_gap[3].weight_change = 0;
        assert!(!has(8, &with_gap, sunday, Ach::SevenFridays));
    }

    #[test]
    fn seven_fridays_tolerates_feeds_at_different_times_of_day() {
        let sunday = datetime(2026, 7, 26, 12, 0);
        let mut log = daily_grow_log(sunday, 1, &[1; 7]);

        log[0].created_at += Duration::hours(5);
        log[6].created_at -= Duration::hours(8);

        assert!(has(8, &log, sunday, Ach::SevenFridays));
    }

    #[test]
    fn seven_fridays_does_not_fire_on_a_week_not_starting_monday() {
        // Same seven consecutive days, but ending on a Wednesday.
        let wednesday = datetime(2026, 7, 29, 12, 0);
        assert_eq!(wednesday.weekday().num_days_from_monday(), 2);

        let log = daily_grow_log(wednesday, 1, &[1; 7]);
        assert!(!has(8, &log, wednesday, Ach::SevenFridays));
    }

    #[test]
    fn seven_fridays_survives_a_short_history_without_panicking() {
        // This branch used to be a `todo!()`.
        let now = NOW();
        for n in 0..7 {
            let log = streak(now, n);
            assert!(!has(1 + n as i32, &log, now, Ach::SevenFridays));
        }
    }


    #[test]
    fn infinity_war_needs_a_crash_to_exactly_one_kg() {
        let now = NOW();

        let log = vec![crate::test_support::grow_log(now, -20, 1)];
        assert!(has(1, &log, now, Ach::InfinityWar));

        let smaller_loss = vec![
            crate::test_support::grow_log(now, -19, 1),];
        assert!(!has(1, &smaller_loss, now, Ach::InfinityWar));

        let landed_on_two = vec![
            crate::test_support::grow_log(now, -20, 2),];
        assert!(!has(2, &landed_on_two, now, Ach::InfinityWar));
    }

    #[test]
    fn eternal_genin_needs_seven_feeds_at_ten_kg_or_less() {
        let now = NOW();

        let tiny = daily_grow_log(now, 3, &[1, 1, 1, 1, 1, 1, 1]);
        assert!(tiny.last().unwrap().current_weight <= 10);
        assert!(has(10, &tiny, now, Ach::EternalGenin));

        let outgrew = daily_grow_log(now, 5, &[1, 1, 1, 1, 1, 1, 1]);
        assert!(outgrew.last().unwrap().current_weight > 10);
        assert!(!has(12, &outgrew, now, Ach::EternalGenin));
    }

    #[test]
    fn new_year_pig_needs_feeds_on_new_years_eve_and_new_years_day() {
        let jan_1 = datetime(2027, 1, 1, 10, 0);
        let log = daily_grow_log(jan_1, 100, &[1, 1]);

        assert_eq!(log[0].created_at.date(), datetime(2026, 12, 31, 10, 0).date());
        assert!(has(102, &log, jan_1, Ach::NewYearPig));

        let ordinary = daily_grow_log(NOW(), 100, &[1, 1]);
        assert!(!has(102, &ordinary, NOW(), Ach::NewYearPig));
    }


    #[test]
    fn date_triggered_achievements_key_off_the_feed_time() {
        let cases = [
            (datetime(2026, 6, 3, 0, 0), Ach::ZeroHour, true),
            (datetime(2026, 6, 3, 0, 1), Ach::ZeroHour, false),
            (datetime(2026, 6, 3, 1, 0), Ach::ZeroHour, false),
            (datetime(2026, 7, 7, 7, 30), Ach::Agent007, true),
            (datetime(2026, 7, 7, 8, 0), Ach::Agent007, false),
            (datetime(2026, 8, 7, 7, 0), Ach::Agent007, false),
            (datetime(2026, 3, 1, 13, 0), Ach::NewHope, true),
            (datetime(2026, 3, 2, 13, 0), Ach::NewHope, false),
            (datetime(2026, 2, 14, 13, 0), Ach::LovePig, true),
            (datetime(2026, 2, 15, 13, 0), Ach::LovePig, false),
            (datetime(2026, 10, 31, 13, 0), Ach::HalloweenPig, true),
            (datetime(2026, 10, 30, 13, 0), Ach::HalloweenPig, false),
            (datetime(2026, 5, 15, 13, 0), Ach::PeremogaBude, true),
            (datetime(2026, 5, 16, 13, 0), Ach::PeremogaBude, false),
        ];

        for (now, ach, fires) in cases {
            assert_eq!(has(50, &[], now, ach), fires, "{now} {ach:?}");
        }
    }

    #[test]
    fn zero_hour_and_new_hope_can_fire_together() {
        let midnight_on_the_first = datetime(2026, 9, 1, 0, 0);
        let unlocked = eval(50, &[], midnight_on_the_first);

        assert!(unlocked.contains(&Ach::ZeroHour));
        assert!(unlocked.contains(&Ach::NewHope));
    }


    #[test]
    fn social_achievements_unlock_at_their_thresholds() {
        let cases = [
            (0, vec![]),
            (1, vec![]),
            (2, vec![Ach::PigInTwoChats]),
            (4, vec![Ach::PigInTwoChats]),
            (5, vec![Ach::PigInTwoChats, Ach::PigEverywhere]),
            (9, vec![Ach::PigInTwoChats, Ach::PigEverywhere]),
            (
                10,
                vec![
                    Ach::PigInTwoChats,
                    Ach::PigEverywhere,
                    Ach::Hruklid19,
                ],
            ),
        ];

        for (count, expected) in cases {
            assert_eq!(
                evaluate_social_achievements(count, &[]),
                expected,
                "chat_count {count}"
            );
        }
    }

    #[test]
    fn social_achievements_skip_what_is_already_unlocked() {
        assert_eq!(
            evaluate_social_achievements(10, &[Ach::PigInTwoChats]),
            vec![Ach::PigEverywhere, Ach::Hruklid19]
        );
    }

    #[test]
    fn the_chat_count_query_is_skipped_once_all_three_are_unlocked() {
        assert!(needs_chat_count(&[]));
        assert!(needs_chat_count(&[Ach::PigInTwoChats, Ach::PigEverywhere]));
        assert!(!needs_chat_count(&[
            Ach::PigInTwoChats,
            Ach::PigEverywhere,
            Ach::Hruklid19,
        ]));
    }

    #[test]
    fn social_achievements_are_never_produced_by_the_history_evaluator() {
        let now = NOW();
        let unlocked = eval(10_000, &streak(now, 30), now);

        for ach in [Ach::PigInTwoChats, Ach::PigEverywhere, Ach::Hruklid19] {
            assert!(!unlocked.contains(&ach), "{ach:?}");
        }
    }

    #[test]
    fn day_pig_and_pigolator_are_awarded_elsewhere() {
        // Both are unlocked by their own DB shims, never by the evaluator.
        let now = NOW();
        let unlocked = eval(69, &streak(now, 30), now);

        assert!(!unlocked.contains(&Ach::PigOfTheDay));
        assert!(!unlocked.contains(&Ach::Pigolator));
    }
}
