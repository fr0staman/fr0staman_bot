use teloxide::utils::html::bold;

use crate::{
    consts::TOP_LIMIT,
    enums::Top10Variant,
    lang::{lng, InnerLang, LocaleTag},
    models::{Game, InlineUser},
};

use super::{flag::Flags, helpers};

pub fn generate_top10_text(
    ltag: LocaleTag,
    top10_info: Vec<InlineUser>,
    chat_type: Top10Variant,
) -> String {
    let summarized = chat_type.summarize();
    let is_win = matches!(summarized, Top10Variant::Win);
    let chat_type = summarized.as_ref();

    let text = lng(&format!("InlineTop10Header_{}", chat_type), ltag);

    let header = bold(&text);
    let key = format!("InlineTop10Line_{}", chat_type);

    let mut result = String::with_capacity(512) + &header + "\n";

    for (index, item) in top10_info.iter().enumerate() {
        let value = if is_win { item.win as i32 } else { item.weight };

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

pub fn generate_chat_top50_text(
    ltag: LocaleTag,
    top50_info: Vec<Game>,
    offset_multiplier: i64,
) -> String {
    let text = lng("GameTop50Header", ltag);
    let header = bold(&text);

    let mut result = String::with_capacity(512) + &header;

    for (index, item) in top50_info.iter().enumerate() {
        let value = item.mass;
        let index = (index as i64) + (offset_multiplier * TOP_LIMIT);

        let line = lng("GameTop50Line", ltag).args(&[
            ("number", (index + 1).to_string()),
            ("name", helpers::escape_links(&item.name)),
            ("value", value.to_string()),
        ]);

        result += &("\n".to_owned() + &line);
    }

    result
}
