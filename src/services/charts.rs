use std::collections::BTreeSet;

use ahash::AHashMap;
use charts_rs::*;
use chrono::{Duration, NaiveDateTime, NaiveTime};
use unicode_width::UnicodeWidthChar;

use crate::{
    config::consts::CHARTS_PIXELS_WIDTH,
    db::models::{Game, GrowLog},
    lang::{InnerLang, LocaleTag, lng},
    services::save_image::svg_to_png,
    utils::date::get_datetime,
};

pub async fn generate_charts(
    data: Vec<(Game, Vec<GrowLog>)>,
    chat_name: String,
    ltag: LocaleTag,
) -> Option<Vec<u8>> {
    let title = lng("TopChartsTitle", ltag).args(&[("chat_name", &chat_name)]);
    let (send, recv) = tokio::sync::oneshot::channel();

    rayon::spawn(move || {
        let encoded = generate_charts_inner(data, title, 14);
        let _ = send.send(encoded);
    });

    recv.await.ok()?
}

pub async fn generate_my_chart(
    game: Game,
    logs: Vec<GrowLog>,
    ltag: LocaleTag,
) -> Option<Vec<u8>> {
    let title = lng("MyPigChartTitle", ltag).args(&[("name", &game.name)]);
    let data = vec![(game, logs)];
    let (send, recv) = tokio::sync::oneshot::channel();

    rayon::spawn(move || {
        let encoded = generate_charts_inner(data, title, 14);
        let _ = send.send(encoded);
    });

    recv.await.ok()?
}

fn generate_charts_inner(
    data: Vec<(Game, Vec<GrowLog>)>,
    title: String,
    days: i64,
) -> Option<Vec<u8>> {
    let data = normalize_data(data, days, get_datetime());

    let mut all_dates = BTreeSet::new();

    for (_, logs) in &data {
        for log in logs {
            all_dates.insert(log.created_at.date());
        }
    }

    let dates: Vec<_> = all_dates.into_iter().collect();

    let min_value = data
        .iter()
        .flat_map(|(_, logs)| logs.iter())
        .map(|log| log.current_weight)
        .min()
        // force lib to show chart from lowest weight
        // without -0.001 in some cases not working
        // maybe due to float precision issues
        .map_or(0.0, |f| (f as f32) - 0.001);

    let chart_data: Vec<_> = data
        .into_iter()
        .map(|(pig, logs)| {
            let mut map = std::collections::HashMap::new();
            for gl in logs {
                map.insert(gl.created_at.date(), gl.current_weight as f32);
            }

            let first_date = map.keys().min().cloned().unwrap();
            let start_index =
                dates.iter().position(|d| d == &first_date).unwrap();

            let aligned_data: Vec<_> = dates[start_index..]
                .iter()
                .map(|d| map.get(d).copied())
                .collect();

            Series {
                name: pig.name,
                data: aligned_data,
                start_index,
                label_show: true,
                ..Default::default()
            }
        })
        .collect();

    let dates = dates.into_iter().map(|d| d.to_string()).collect();

    let mut line_chart =
        LineChart::new_with_theme(chart_data, dates, THEME_GRAFANA);

    line_chart.legend_show = Some(true);
    line_chart.margin =
        Box { top: 20.0, bottom: 10.0, left: 10.0, right: 10.0 };
    line_chart.legend_margin = Some(Box {
        top: line_chart.title_height,
        bottom: 20.0,
        ..Default::default()
    });
    line_chart.title_text = title;
    line_chart.font_family = "Roboto".to_string();

    line_chart.y_axis_configs[0].axis_min = Some(min_value);

    let svg = line_chart.svg().ok()?;

    svg_to_png(&svg, CHARTS_PIXELS_WIDTH).ok()
}

fn normalize_data(
    mut data: Vec<(Game, Vec<GrowLog>)>,
    days: i64,
    today: NaiveDateTime,
) -> Vec<(Game, Vec<GrowLog>)> {
    let mut result = Vec::new();

    let start_date = today - Duration::days(days - 1);

    let user_dates: Vec<_> =
        (0..days).map(|i| (start_date + Duration::days(i)).date()).collect();

    for (game, _) in data.iter_mut() {
        let mut chars: Vec<_> = game.name.chars().collect();

        for c in &mut chars {
            if UnicodeWidthChar::width(*c).unwrap_or(1) > 1 {
                *c = ' ';
            }
        }

        game.name = chars.into_iter().collect();
    }

    for (game, mut grow_logs) in data {
        grow_logs.sort_unstable_by_key(|l| l.created_at);

        let mut logs_by_date = AHashMap::with_capacity(grow_logs.len());
        for log in grow_logs {
            logs_by_date.insert(log.created_at.date(), log);
        }

        // Pig had history before the window if its earliest in-window log is
        // not its very first ever log (current_weight == weight_change + 1
        // only holds for the first-ever grow starting from mass 1).
        let first_in_window = logs_by_date
            .keys()
            .min()
            .and_then(|d| logs_by_date.get(d));
        let had_prior_history = first_in_window
            .map(|l| l.current_weight != l.weight_change + 1)
            .unwrap_or(false);

        let mut current_weight = first_in_window
            .map(|l| l.current_weight)
            .unwrap_or(game.mass);

        let mut normalized = Vec::with_capacity(user_dates.len());

        for day in &user_dates {
            if let Some(log) = logs_by_date.get(day) {
                current_weight = log.current_weight;
                normalized.push(log.clone());
            } else if logs_by_date.is_empty()
                || !normalized.is_empty()
                || had_prior_history
            {
                normalized.push(GrowLog {
                    game_id: game.id,
                    created_at: day.and_time(NaiveTime::MIN),
                    weight_change: 0,
                    current_weight,
                });
            }
            // else: pig started within the window but hasn't grown yet on this day — skip
        }

        result.push((game, normalized));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{datetime, game, grow_log};

    const TODAY: fn() -> NaiveDateTime = || datetime(2026, 7, 28, 12, 0);

    fn normalized(logs: Vec<GrowLog>) -> Vec<GrowLog> {
        normalize_data(vec![(game(100), logs)], 14, TODAY())
            .pop()
            .unwrap()
            .1
    }

    #[test]
    fn a_pig_with_no_history_gets_a_flat_line_across_the_window() {
        // `logs_by_date.is_empty()` takes the fill branch for every day.
        let out = normalized(vec![]);

        assert_eq!(out.len(), 14);
        assert!(out.iter().all(|l| l.current_weight == 100));
        assert!(out.iter().all(|l| l.weight_change == 0));
    }

    #[test]
    fn gaps_are_filled_forward_with_the_last_known_weight() {
        let logs = vec![
            grow_log(TODAY() - Duration::days(13), 5, 50),
            grow_log(TODAY(), 5, 55),
        ];
        let out = normalized(logs);

        assert_eq!(out.len(), 14);
        assert_eq!(out.first().unwrap().current_weight, 50);
        // Everything between the two real feeds holds at 50.
        assert!(out[1..13].iter().all(|l| l.current_weight == 50));
        assert_eq!(out.last().unwrap().current_weight, 55);
    }

    #[test]
    fn a_pig_born_inside_the_window_has_no_points_before_its_first_feed() {
        // The first-ever feed is recognisable as `current_weight ==
        // weight_change + 1` (it started from CHAT_PIG_START_MASS).
        let birth = TODAY() - Duration::days(3);
        let logs = vec![grow_log(birth, 4, 5), grow_log(TODAY(), 1, 6)];

        let out = normalized(logs);

        assert_eq!(out.len(), 4, "expected only the days from birth onward");
        assert_eq!(out.first().unwrap().current_weight, 5);
        assert_eq!(out.last().unwrap().current_weight, 6);
    }

    #[test]
    fn a_pig_that_existed_before_the_window_is_back_filled() {
        // Not a first-ever feed, so the days before it are drawn flat.
        let first = TODAY() - Duration::days(3);
        let logs = vec![grow_log(first, 4, 80), grow_log(TODAY(), 1, 81)];

        let out = normalized(logs);

        assert_eq!(out.len(), 14);
        assert!(out[..11].iter().all(|l| l.current_weight == 80));
    }

    #[test]
    fn logs_older_than_the_window_are_dropped() {
        let logs = vec![
            grow_log(TODAY() - Duration::days(40), 1, 10),
            grow_log(TODAY(), 1, 90),
        ];

        let out = normalized(logs);

        assert_eq!(out.len(), 14);
        assert!(out.iter().all(|l| l.created_at.date() >= (TODAY() - Duration::days(13)).date()));
    }

    #[test]
    fn unsorted_input_is_handled() {
        let a = grow_log(TODAY() - Duration::days(2), 1, 70);
        let b = grow_log(TODAY(), 1, 71);

        let sorted = normalized(vec![a.clone(), b.clone()]);
        let unsorted = normalized(vec![b, a]);

        assert_eq!(
            sorted.iter().map(|l| l.current_weight).collect::<Vec<_>>(),
            unsorted.iter().map(|l| l.current_weight).collect::<Vec<_>>()
        );
    }

    #[test]
    fn only_the_last_feed_of_a_day_survives() {
        // `logs_by_date` is keyed by date, so a second feed the same day wins.
        let logs = vec![
            grow_log(TODAY(), 1, 60),
            grow_log(TODAY() + Duration::hours(1), 1, 61),
        ];

        let out = normalized(logs);
        assert_eq!(out.last().unwrap().current_weight, 61);
    }

    #[test]
    fn wide_characters_in_a_pig_name_are_blanked_for_the_legend() {
        let mut pig = game(10);
        pig.name = "漢字Pig🐷".to_owned();

        let out = normalize_data(vec![(pig, vec![])], 14, TODAY());

        assert_eq!(out[0].0.name, "  Pig ");
    }

    #[test]
    fn an_all_wide_name_becomes_blank_rather_than_empty() {
        let mut pig = game(10);
        pig.name = "漢字漢字".to_owned();

        let out = normalize_data(vec![(pig, vec![])], 14, TODAY());

        assert_eq!(out[0].0.name, "    ");
    }

    #[test]
    fn several_pigs_are_normalised_independently() {
        // First-ever feed: `current_weight == weight_change + 1`.
        let a = (game(10), vec![grow_log(TODAY(), 9, 10)]);
        let mut second = game(20);
        second.id = 2;
        let b = (second, vec![]);

        let out = normalize_data(vec![a, b], 14, TODAY());

        assert_eq!(out.len(), 2);
        assert_eq!(out[0].1.len(), 1, "born today: one point");
        assert_eq!(out[1].1.len(), 14, "no history: flat line");
    }

    #[test]
    fn the_first_ever_feed_is_detected_by_its_weight_arithmetic() {
        // The heuristic is arithmetic, not a `created_at` lookup: any pig
        // satisfying `current_weight == weight_change + 1` reads as newborn.
        let day = TODAY() - Duration::days(5);

        let newborn = normalized(vec![grow_log(day, 7, 8)]);
        assert_eq!(newborn.len(), 6, "treated as born 5 days ago");

        let veteran = normalized(vec![grow_log(day, 7, 9)]);
        assert_eq!(veteran.len(), 14, "treated as pre-existing");
    }
}
