use std::cmp::Ordering;

use chrono::Datelike;
use rand::RngExt;

use crate::enums::PigGrowthStatus;

use super::date::{get_datetime, get_fixed_timestamp};

const STRANGE_DELIMITER: f64 = 5527.0;
const ANOTHER_STRANGE_DELIMITER: f64 = 1009.0;
const SECOND_STRANGE_DELIMITER: f64 = 4049.0;

pub fn calculate_hryak_size(user_id: i64) -> i32 {
    let datetime = get_datetime();
    let day = f64::from(datetime.day());
    let month = f64::from(datetime.month());
    let timestamp = get_fixed_timestamp(datetime) as f64;
    let uid = user_id as f64;

    let calculated_category =
        timestamp / STRANGE_DELIMITER * day / month + uid / (day * month);
    let kf = calculated_category.rem_euclid(25.0);

    let category = match kf {
        21.0.. => 7.0,
        12.0.. => 5.0,
        6.0.. => 3.0,
        0.3.. => 2.0,
        0.05.. => 1.0,
        0.0.. => 0.39,
        _ => 0.0,
    };

    let modulo_by_size =
        SECOND_STRANGE_DELIMITER + 10.0 * (day + (month - 8.0) * 30.0);
    let size = (timestamp / day * month / ANOTHER_STRANGE_DELIMITER + uid)
        .rem_euclid(modulo_by_size)
        / category;

    let casted_size = size as i32;

    if casted_size == 0 { 1 } else { casted_size }
}

pub fn calculate_cpu_clock(hryak_size: i32, user_id: i64) -> f32 {
    const MIN_CLOCK: i64 = 19;
    const MAX_TOP_ON_MIN_CLOCK: i64 = 42;

    ((hryak_size as i64 + user_id).rem_euclid(MAX_TOP_ON_MIN_CLOCK) + MIN_CLOCK)
        as f32
        / 10.0
}

pub fn calculate_ram_clock(hryak_size: i32, user_id: i64) -> u32 {
    const STEP: f32 = 266.67;
    const MIN_CLOCK: i64 = 1333;
    const MAX_TOP_ON_MIN_CLOCK: i64 = 4533;

    let ram_clock = ((hryak_size as i64 + user_id)
        .rem_euclid(MAX_TOP_ON_MIN_CLOCK)
        + MIN_CLOCK) as u32;

    ram_clock + (STEP - (ram_clock as f32).rem_euclid(STEP)) as u32
}

pub fn calculate_gpu_hashrate(hryak_size: i32, user_id: i64) -> f32 {
    const MAX_HASHRATE: i64 = 12800;

    ((hryak_size as i64 + user_id).rem_euclid(MAX_HASHRATE)) as f32 / 100.0
}

pub fn calculate_chat_pig_grow(current_kg: i32) -> (i32, PigGrowthStatus) {
    let chance = rand::rng().random_range(-8..=20);

    match chance.cmp(&0) {
        Ordering::Greater => (chance, PigGrowthStatus::Gained),
        Ordering::Less => {
            let min = if current_kg < 20 { current_kg - 1 } else { 20 };
            if min < 1 {
                // Try another.
                return calculate_chat_pig_grow(current_kg);
            }
            let chance = rand::rng().random_range(-min..0);
            (chance, PigGrowthStatus::Lost)
        },
        Ordering::Equal => {
            if current_kg == 0 {
                // Try another.
                return calculate_chat_pig_grow(current_kg);
            }
            (chance, PigGrowthStatus::Maintained)
        },
    }
}

pub fn get_pig_emoji<'a>(hryak_size: i32) -> &'a str {
    match hryak_size {
        10000.. => "🪐",
        8000.. => "☄",
        7000.. => "💫",
        6000.. => "🌠",
        5000.. => "🌍",
        4000.. => "🌋",
        3000.. => "💥",
        2000.. => "☢️",
        1488 => "⚡⚡",
        1000.. => "☣️",
        800.. => "🚷",
        777 => "🎰",
        666 => "👹",
        500.. => "🐖💨",
        300.. => "🐖",
        100.. => "🐽",
        20.. => "🐷",
        18 => "🔞",
        10.. => "🍖",
        1 => "🍽",
        _ => "🦴",
    }
}

pub fn get_oc_cpu_emoji<'a>(cpu_clock: f32) -> &'a str {
    match cpu_clock {
        5.5.. => "🌋",
        5.0.. => "💥",
        4.7.. => "💣",
        4.4.. => "🧨",
        4.0.. => "♨",
        _ => "🧊",
    }
}

pub fn get_oc_ram_emoji<'a>(ram_clock: u32) -> &'a str {
    match ram_clock {
        5300.. => "🌋",
        5000.. => "💥",
        4600.. => "💣",
        4000.. => "🧨",
        3600.. => "♨",
        _ => "🧊",
    }
}

pub fn get_oc_gpu_emoji<'a>(hashrate: f32) -> &'a str {
    match hashrate {
        120.0.. => "🔥",
        110.0.. => "🚝",
        100.0.. => "🚜",
        80.0.. => "🚛",
        60.0.. => "⛹",
        40.0.. => "🧗",
        20.0.. => "🤸",
        _ => "🐢",
    }
}
