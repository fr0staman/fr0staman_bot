use chrono::{Duration, prelude::*};

const GMT: i32 = 3;
const FIXED_HOUR: u32 = 12;
const FIXED_MINUTE: u32 = 36;
const FIXED_OFFSET_IN_SECONDS: i32 = -GMT * 3600;
const FIXED_OFFSET: FixedOffset =
    FixedOffset::east_opt(FIXED_OFFSET_IN_SECONDS)
        .expect("Wrong fixed offset!");

pub fn get_datetime() -> NaiveDateTime {
    let datetime = Utc::now().naive_utc();

    let date = FIXED_OFFSET.from_local_datetime(&datetime).unwrap();

    date.naive_utc()
}

pub fn get_date() -> NaiveDate {
    get_datetime().date()
}

pub fn get_fixed_timestamp(expected_datetime: NaiveDateTime) -> i64 {
    expected_datetime
        .with_hour(FIXED_HOUR)
        .unwrap()
        .with_minute(FIXED_MINUTE)
        .unwrap()
        .and_utc()
        .timestamp()
}

pub fn get_timediff(cur_datetime: NaiveDateTime) -> (i64, i64, i64) {
    let next_day = cur_datetime + Duration::days(1);
    let next_datetime = next_day.date().and_hms_opt(0, 0, 0).unwrap();

    let duration =
        next_datetime.round_subsecs(0).signed_duration_since(cur_datetime);

    let hours = duration.num_hours();
    let minutes = duration.num_minutes() % 60;
    let seconds = duration.num_seconds() % 60;
    (hours, minutes, seconds)
}

pub fn get_datetime_from_message_date(
    datetime: DateTime<Utc>,
) -> NaiveDateTime {
    FIXED_OFFSET.from_local_datetime(&datetime.naive_utc()).unwrap().naive_utc()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dt(y: i32, m: u32, d: u32, h: u32, min: u32, s: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(y, m, d)
            .unwrap()
            .and_hms_opt(h, min, s)
            .unwrap()
    }

    #[test]
    fn the_fixed_offset_is_effectively_plus_three_hours() {
        // Built from a negative constant but applied via
        // `from_local_datetime(..).naive_utc()`, which inverts it to UTC+3.
        let utc = dt(2026, 7, 28, 9, 0, 0);
        let shifted = get_datetime_from_message_date(utc.and_utc());

        assert_eq!(shifted, dt(2026, 7, 28, 12, 0, 0));
    }

    #[test]
    fn the_offset_carries_across_a_date_boundary() {
        let utc = dt(2026, 7, 28, 22, 30, 0);
        let shifted = get_datetime_from_message_date(utc.and_utc());

        assert_eq!(shifted, dt(2026, 7, 29, 1, 30, 0));
    }


    #[test]
    fn fixed_timestamp_pins_hour_and_minute() {
        let a = get_fixed_timestamp(dt(2026, 7, 28, 0, 0, 0));
        let b = get_fixed_timestamp(dt(2026, 7, 28, 23, 59, 0));

        assert_eq!(a, b);
        assert_eq!(a, dt(2026, 7, 28, FIXED_HOUR, FIXED_MINUTE, 0)
            .and_utc()
            .timestamp());
    }

    #[test]
    fn fixed_timestamp_keeps_the_seconds() {
        // By design: only the hour and minute are pinned, so
        // the seed feeding `calculate_hryak_size` keeps a slight wobble
        // across the day rather than being perfectly constant.
        let base = get_fixed_timestamp(dt(2026, 7, 28, 3, 3, 0));
        let later = get_fixed_timestamp(dt(2026, 7, 28, 3, 3, 41));

        assert_eq!(later - base, 41);
    }

    #[test]
    fn fixed_timestamp_differs_between_days() {
        let a = get_fixed_timestamp(dt(2026, 7, 28, 5, 0, 0));
        let b = get_fixed_timestamp(dt(2026, 7, 29, 5, 0, 0));

        assert_eq!(b - a, 86_400);
    }


    #[test]
    fn timediff_counts_down_to_midnight() {
        let cases = [
            (dt(2026, 7, 28, 0, 0, 0), (24, 0, 0)),
            (dt(2026, 7, 28, 23, 59, 59), (0, 0, 1)),
            (dt(2026, 7, 28, 23, 0, 0), (1, 0, 0)),
            (dt(2026, 7, 28, 12, 30, 30), (11, 29, 30)),
            (dt(2026, 7, 28, 22, 15, 45), (1, 44, 15)),
        ];

        for (now, expected) in cases {
            assert_eq!(get_timediff(now), expected, "now {now}");
        }
    }

    #[test]
    fn timediff_components_are_always_in_range() {
        let mut now = dt(2026, 7, 28, 0, 0, 0);

        for _ in 0..(24 * 60) {
            let (h, m, s) = get_timediff(now);
            assert!((0..=24).contains(&h), "{now} -> {h}");
            assert!((0..60).contains(&m), "{now} -> {m}");
            assert!((0..60).contains(&s), "{now} -> {s}");
            now += Duration::minutes(1);
        }
    }

    #[test]
    fn timediff_crosses_a_month_boundary() {
        assert_eq!(get_timediff(dt(2026, 7, 31, 23, 30, 0)), (0, 30, 0));
        // A leap day, for good measure.
        assert_eq!(get_timediff(dt(2024, 2, 28, 22, 0, 0)), (2, 0, 0));
    }
}
