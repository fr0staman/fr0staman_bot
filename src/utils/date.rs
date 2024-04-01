use chrono::{prelude::*, Duration};

const FIXED_HOUR: u32 = 12;
const FIXED_MINUTE: u32 = 36;
const FIXED_OFFSET_IN_SECONDS: i32 = -3 * 3600;
const FIXED_OFFSET: FixedOffset =
    match FixedOffset::east_opt(FIXED_OFFSET_IN_SECONDS) {
        Some(offset) => offset,
        None => panic!("Wrong fixed offset!"),
    };

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
