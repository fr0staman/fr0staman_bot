use std::cmp::Ordering;

use chrono::Datelike;
use rand::RngExt;

use crate::enums::PigGrowthStatus;

use super::date::{get_datetime, get_fixed_timestamp};

const STRANGE_DELIMITER: f64 = 5527.0;
const ANOTHER_STRANGE_DELIMITER: f64 = 1009.0;
const SECOND_STRANGE_DELIMITER: f64 = 4049.0;

pub fn calculate_hryak_size(user_id: i64) -> i32 {
    calculate_hryak_size_at(get_datetime(), user_id)
}

/// Deterministic given `(datetime, user_id)`, though the preserved seconds
/// make it wobble slightly within a day by design.
pub fn calculate_hryak_size_at(
    datetime: chrono::NaiveDateTime,
    user_id: i64,
) -> i32 {
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
    calculate_chat_pig_grow_with(&mut rand::rng(), current_kg)
}

/// RNG passed in so it can be seeded in tests.
pub fn calculate_chat_pig_grow_with<R: RngExt>(
    rng: &mut R,
    current_kg: i32,
) -> (i32, PigGrowthStatus) {
    let chance = rng.random_range(-8..=20);

    match chance.cmp(&0) {
        Ordering::Greater => (chance, PigGrowthStatus::Gained),
        Ordering::Less => {
            // `<=`, not `<`: at exactly 20 the old bound let -20 through
            // and landed the pig on 0.
            let min = if current_kg <= 20 { current_kg - 1 } else { 20 };
            if min < 1 {
                // Try another.
                return calculate_chat_pig_grow_with(rng, current_kg);
            }
            let chance = rng.random_range(-min..0);
            (chance, PigGrowthStatus::Lost)
        },
        Ordering::Equal => {
            if current_kg == 0 {
                // Try another.
                return calculate_chat_pig_grow_with(rng, current_kg);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::datetime;
    use rand::{SeedableRng, rngs::StdRng};


    #[test]
    fn hryak_size_is_deterministic_for_a_given_day_and_user() {
        let now = datetime(2026, 7, 28, 9, 15);

        let first = calculate_hryak_size_at(now, 123_456);
        let second = calculate_hryak_size_at(now, 123_456);

        assert_eq!(first, second);
    }

    #[test]
    fn hryak_size_golden_values() {
        // Locks the formula: any change to the constants or the category
        // ladder moves these.
        let now = datetime(2026, 7, 28, 9, 15);

        let cases = [
            (1_i64, 633),
            (100, 653),
            (777_777, 1116),
            (123_456_789, 477),
            (5_000_000_000, 416),
        ];

        for (user_id, expected) in cases {
            assert_eq!(
                calculate_hryak_size_at(now, user_id),
                expected,
                "user_id {user_id}"
            );
        }
    }

    #[test]
    fn hryak_size_is_never_zero_or_negative() {
        let now = datetime(2026, 3, 9, 0, 0);

        for user_id in (1_i64..2_000_000).step_by(9_973) {
            let size = calculate_hryak_size_at(now, user_id);
            assert!(size >= 1, "user_id {user_id} produced {size}");
        }
    }

    #[test]
    fn hryak_size_stays_positive_across_every_calendar_day() {
        // `modulo_by_size` is derived from day/month, so a month where it
        // went non-positive would poison the whole day.
        for month in 1..=12u32 {
            for day in 1..=28u32 {
                let now = datetime(2026, month, day, 12, 0);
                let size = calculate_hryak_size_at(now, 42);
                assert!(size >= 1, "{day}.{month} produced {size}");
            }
        }
    }

    #[test]
    fn hryak_size_drifts_within_a_day_because_seconds_survive() {
        // `get_fixed_timestamp` pins the hour and minute to 12:36 but leaves
        // the seconds untouched, so the daily seed keeps a slight wobble.
        // Intentional.
        let base = datetime(2026, 7, 28, 9, 15);
        let plus_30s = base + chrono::Duration::seconds(30);

        assert_eq!(get_fixed_timestamp(base) + 30, get_fixed_timestamp(plus_30s));
    }


    #[test]
    fn cpu_clock_stays_in_range() {
        for size in (-5_000..5_000).step_by(37) {
            for uid in [0_i64, 1, 999, -999, i32::MAX as i64] {
                let clock = calculate_cpu_clock(size, uid);
                assert!(
                    (1.9..=6.0).contains(&clock),
                    "size {size} uid {uid} -> {clock}"
                );
            }
        }
    }

    #[test]
    fn ram_clock_stays_in_range_and_snaps_to_the_step() {
        for size in (-5_000..5_000).step_by(37) {
            let clock = calculate_ram_clock(size, 7);
            assert!((1333..=5867).contains(&clock), "size {size} -> {clock}");
        }
    }

    #[test]
    fn gpu_hashrate_stays_in_range() {
        for size in (-5_000..5_000).step_by(37) {
            let rate = calculate_gpu_hashrate(size, 7);
            assert!((0.0..128.0).contains(&rate), "size {size} -> {rate}");
        }
    }

    #[test]
    fn oc_values_are_deterministic() {
        assert_eq!(calculate_cpu_clock(100, 500), calculate_cpu_clock(100, 500));
        assert_eq!(calculate_ram_clock(100, 500), calculate_ram_clock(100, 500));
        assert_eq!(
            calculate_gpu_hashrate(100, 500),
            calculate_gpu_hashrate(100, 500)
        );
        assert_eq!(calculate_cpu_clock(100, 500), calculate_cpu_clock(500, 100));
    }


    #[test]
    fn grow_roll_is_reproducible_from_a_seed() {
        let mut a = StdRng::seed_from_u64(42);
        let mut b = StdRng::seed_from_u64(42);

        assert_eq!(
            calculate_chat_pig_grow_with(&mut a, 100),
            calculate_chat_pig_grow_with(&mut b, 100)
        );
    }

    #[test]
    fn grow_roll_status_matches_the_sign_of_the_offset() {
        for seed in 0..500u64 {
            let mut rng = StdRng::seed_from_u64(seed);
            let (offset, status) = calculate_chat_pig_grow_with(&mut rng, 100);

            let expected = match offset {
                o if o > 0 => PigGrowthStatus::Gained,
                o if o < 0 => PigGrowthStatus::Lost,
                _ => PigGrowthStatus::Maintained,
            };
            assert_eq!(status, expected, "seed {seed} offset {offset}");
        }
    }

    #[test]
    fn grow_roll_gain_never_exceeds_twenty() {
        for seed in 0..2_000u64 {
            let mut rng = StdRng::seed_from_u64(seed);
            let (offset, _) = calculate_chat_pig_grow_with(&mut rng, 100);
            assert!((-20..=20).contains(&offset), "seed {seed} -> {offset}");
        }
    }

    #[test]
    fn grow_roll_never_drops_a_pig_below_one_kg() {
        // The loss clamp is the only thing keeping a light pig alive, and it
        // must hold at every mass — including exactly 20, where the bound
        // switches from `mass - 1` to a flat 20.
        for mass in [1, 2, 3, 5, 10, 19, 20, 21, 22, 100, 5_000] {
            for seed in 0..3_000u64 {
                let mut rng = StdRng::seed_from_u64(seed);
                let (offset, _) = calculate_chat_pig_grow_with(&mut rng, mass);
                assert!(
                    mass + offset >= 1,
                    "mass {mass} seed {seed} offset {offset} -> {}",
                    mass + offset
                );
            }
        }
    }

    #[test]
    fn grow_roll_holds_the_invariant_at_every_small_mass() {
        for mass in 1..60 {
            for seed in 0..1_000u64 {
                let mut rng = StdRng::seed_from_u64(seed);
                let (offset, _) = calculate_chat_pig_grow_with(&mut rng, mass);
                assert!(
                    mass + offset >= 1,
                    "mass {mass} seed {seed} offset {offset}"
                );
            }
        }
    }

    #[test]
    fn the_loss_ceiling_is_capped_by_the_pigs_own_mass_up_to_twenty() {
        // Below and at 20 kg the worst case is `-(mass - 1)`; above it the
        // flat -20 applies.
        for (mass, worst_case) in
            [(2, -1), (5, -4), (20, -19), (21, -20), (100, -20)]
        {
            let mut seen_worst = false;
            for seed in 0..20_000u64 {
                let mut rng = StdRng::seed_from_u64(seed);
                let (offset, _) = calculate_chat_pig_grow_with(&mut rng, mass);
                assert!(
                    offset >= worst_case,
                    "mass {mass} seed {seed} lost {offset}, past {worst_case}"
                );
                if offset == worst_case {
                    seen_worst = true;
                }
            }
            assert!(seen_worst, "mass {mass} never hit its {worst_case} bound");
        }
    }


    #[test]
    fn pig_emoji_boundaries() {
        let cases = [
            (i32::MIN, "🦴"),
            (-1, "🦴"),
            (0, "🦴"),
            (1, "🍽"),
            (2, "🦴"),
            (9, "🦴"),
            (10, "🍖"),
            (17, "🍖"),
            (18, "🔞"),
            (19, "🍖"),
            (20, "🐷"),
            (99, "🐷"),
            (100, "🐽"),
            (299, "🐽"),
            (300, "🐖"),
            (499, "🐖"),
            (500, "🐖💨"),
            (665, "🐖💨"),
            (666, "👹"),
            (667, "🐖💨"),
            (776, "🐖💨"),
            (777, "🎰"),
            (778, "🐖💨"),
            (799, "🐖💨"),
            (800, "🚷"),
            (999, "🚷"),
            (1000, "☣️"),
            (1487, "☣️"),
            (1488, "⚡⚡"),
            (1489, "☣️"),
            (1999, "☣️"),
            (2000, "☢️"),
            (3000, "💥"),
            (4000, "🌋"),
            (5000, "🌍"),
            (6000, "🌠"),
            (7000, "💫"),
            (8000, "☄"),
            (9999, "☄"),
            (10000, "🪐"),
            (i32::MAX, "🪐"),
        ];

        for (mass, expected) in cases {
            assert_eq!(get_pig_emoji(mass), expected, "mass {mass}");
        }
    }

    #[test]
    fn oc_emoji_boundaries() {
        let cpu = [
            (0.0_f32, "🧊"),
            (3.9, "🧊"),
            (4.0, "♨"),
            (4.3, "♨"),
            (4.4, "🧨"),
            (4.6, "🧨"),
            (4.7, "💣"),
            (4.9, "💣"),
            (5.0, "💥"),
            (5.4, "💥"),
            (5.5, "🌋"),
            (6.0, "🌋"),
        ];
        for (clock, expected) in cpu {
            assert_eq!(get_oc_cpu_emoji(clock), expected, "cpu {clock}");
        }

        let ram = [
            (0_u32, "🧊"),
            (3599, "🧊"),
            (3600, "♨"),
            (3999, "♨"),
            (4000, "🧨"),
            (4599, "🧨"),
            (4600, "💣"),
            (4999, "💣"),
            (5000, "💥"),
            (5299, "💥"),
            (5300, "🌋"),
        ];
        for (clock, expected) in ram {
            assert_eq!(get_oc_ram_emoji(clock), expected, "ram {clock}");
        }

        let gpu = [
            (0.0_f32, "🐢"),
            (19.9, "🐢"),
            (20.0, "🤸"),
            (40.0, "🧗"),
            (60.0, "⛹"),
            (80.0, "🚛"),
            (100.0, "🚜"),
            (110.0, "🚝"),
            (120.0, "🔥"),
            (128.0, "🔥"),
        ];
        for (rate, expected) in gpu {
            assert_eq!(get_oc_gpu_emoji(rate), expected, "gpu {rate}");
        }
    }
}
