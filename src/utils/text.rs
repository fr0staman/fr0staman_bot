use teloxide::utils::html::bold;

use crate::{
    config::consts::{TOP_LIMIT, TOP_LIMIT_WITH_CHARTS},
    db::models::{Game, InlineUser},
    enums::Top10Variant,
    lang::{InnerLang, LocaleTag, lng},
};

use super::{flag::Flags, helpers};

pub fn generate_top10_text(
    ltag: LocaleTag,
    top10_info: Vec<InlineUser>,
    chat_type: Top10Variant,
) -> String {
    let summarized = chat_type.summarize();
    let is_win = matches!(summarized, Top10Variant::Win);
    let chat_type = summarized.into_str();

    let text = lng(&format!("InlineTop10Header_{}", chat_type), ltag);

    let header = bold(&text);
    let key = format!("InlineTop10Line_{}", chat_type);

    let mut result = String::with_capacity(512) + &header + "\n";

    for (index, item) in top10_info.iter().enumerate() {
        let value = if is_win { item.win } else { item.weight };

        let code = Flags::from_code(&item.flag).unwrap_or(Flags::Us);
        let flag = code.to_emoji();

        let line = lng(&key, ltag).args(&[
            ("number", (index + 1).to_string()),
            ("flag", flag.to_string()),
            ("name", helpers::escape_links(&item.name)),
            ("value", value.to_string()),
        ]);

        result += &("\n".to_owned() + &line);
    }

    result
}

pub fn generate_chat_top_text(
    ltag: LocaleTag,
    top_info: Vec<Game>,
    offset_multiplier: i64,
    with_chart: bool,
) -> String {
    let top_limit = if with_chart { TOP_LIMIT_WITH_CHARTS } else { TOP_LIMIT };

    let text = lng("GameTopHeader", ltag).args(&[("limit", top_limit)]);
    let header = bold(&text);

    let mut result = String::with_capacity(512) + &header;

    for (index, item) in top_info.iter().enumerate() {
        let value = item.mass;

        let index = (index as i64) + (offset_multiplier * top_limit);

        let line = lng("GameTopLine", ltag).args(&[
            ("number", (index + 1).to_string()),
            ("name", helpers::escape_links(&item.name)),
            ("value", value.to_string()),
        ]);

        result += &("\n".to_owned() + &line);
    }

    result
}
