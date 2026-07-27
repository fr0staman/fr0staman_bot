use std::collections::BTreeSet;

use ahash::AHashMap;
use charts_rs::*;
use chrono::{Duration, NaiveTime};
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
    let data = normalize_data(data, days);

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
) -> Vec<(Game, Vec<GrowLog>)> {
    let mut result = Vec::new();

    let today = get_datetime();
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
