use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, UserId};
use url::Url;

use crate::{
    enums::CbActions,
    lang::{lng, LocaleTag},
    utils::helpers::encode_callback_data,
};

pub fn keyboard_new_name(
    ltag: LocaleTag,
    id_user: UserId,
    new_name: String,
) -> InlineKeyboardMarkup {
    let coded_data =
        encode_callback_data(CbActions::GiveName, id_user, new_name);

    let keyboard = vec![vec![InlineKeyboardButton::callback(
        lng("HandPigNameChangeButton", ltag),
        coded_data,
    )]];

    InlineKeyboardMarkup::new(keyboard)
}

pub fn keyboard_day_pig(
    ltag: LocaleTag,
    id_user: UserId,
) -> InlineKeyboardMarkup {
    let coded_data = encode_callback_data(CbActions::FindHryak, id_user, "");

    let keyboard = vec![vec![InlineKeyboardButton::callback(
        lng("InlineDayPigButton", ltag),
        coded_data,
    )]];

    InlineKeyboardMarkup::new(keyboard)
}

pub fn keyboard_add_inline_top10(
    ltag: LocaleTag,
    id_user: UserId,
) -> InlineKeyboardMarkup {
    let coded_data = encode_callback_data(CbActions::AddChat, id_user, "");

    let keyboard = vec![vec![InlineKeyboardButton::callback(
        lng("InlineAddTop10ChatButton", ltag),
        coded_data,
    )]];

    InlineKeyboardMarkup::new(keyboard)
}

pub fn keyboard_in_top10(
    ltag: LocaleTag,
    id_user: UserId,
    to: &str,
) -> InlineKeyboardMarkup {
    let coded_data = encode_callback_data(CbActions::Top10, id_user, to);

    let key = to.replace("p_", "");

    let keyboard = vec![vec![InlineKeyboardButton::callback(
        lng(&format!("InlineTop10ButtonIn_{}", &key), ltag),
        coded_data,
    )]];

    InlineKeyboardMarkup::new(keyboard)
}

pub fn keyboard_start_duel(
    ltag: LocaleTag,
    id_user: UserId,
) -> InlineKeyboardMarkup {
    let coded_data = encode_callback_data(CbActions::StartDuel, id_user, "");

    let keyboard = vec![vec![InlineKeyboardButton::callback(
        lng("InlineDuelStartButton", ltag),
        coded_data,
    )]];

    InlineKeyboardMarkup::new(keyboard)
}

pub fn keyboard_startgroup(ltag: LocaleTag, url: Url) -> InlineKeyboardMarkup {
    let text = lng("BotAddToGroup", ltag);
    let url = url.join("?startgroup=chat").unwrap();
    let keyboard = vec![vec![InlineKeyboardButton::url(text, url)]];

    InlineKeyboardMarkup::new(keyboard)
}

pub fn keyboard_top50(
    ltag: LocaleTag,
    offset: i64,
    id_user: UserId,
    is_end: bool,
) -> InlineKeyboardMarkup {
    let mut row: Vec<InlineKeyboardButton> = vec![];

    let left_offset = offset - 1;
    if left_offset != 0 {
        let button_left = lng("GameTop50Button_left", ltag);

        let button_left_data = encode_callback_data(
            CbActions::TopLeft,
            id_user,
            left_offset.to_string(),
        );

        row.push(InlineKeyboardButton::callback(button_left, button_left_data));
    }

    if !is_end {
        let button_right = lng("GameTop50Button_right", ltag);

        let right_offset = offset + 1;
        let button_right_data = encode_callback_data(
            CbActions::TopRight,
            id_user,
            right_offset.to_string(),
        );

        row.push(InlineKeyboardButton::callback(
            button_right,
            button_right_data,
        ));
    }

    let keyboard = vec![row];

    InlineKeyboardMarkup::new(keyboard)
}

pub fn keyboard_voice_check(id_user: UserId) -> InlineKeyboardMarkup {
    let success_data = encode_callback_data(CbActions::AllowVoice, id_user, "");
    let success_button = InlineKeyboardButton::callback("✅", success_data);
    let denied_data =
        encode_callback_data(CbActions::DisallowVoice, id_user, "");
    let denied_button = InlineKeyboardButton::callback("❌", denied_data);

    let keyboard = vec![vec![success_button, denied_button]];

    InlineKeyboardMarkup::new(keyboard)
}

pub fn keyboard_change_flag(
    ltag: LocaleTag,
    id_user: UserId,
    flag_code: &str,
) -> InlineKeyboardMarkup {
    let data = encode_callback_data(CbActions::ChangeFlag, id_user, flag_code);

    let text = lng("HandPigFlagChangeButton", ltag);
    let keyboard = vec![vec![InlineKeyboardButton::callback(text, data)]];

    InlineKeyboardMarkup::new(keyboard)
}

pub fn keyboard_empty() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::default()
}
