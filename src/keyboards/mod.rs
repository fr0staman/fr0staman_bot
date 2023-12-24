use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, UserId};

use crate::{
    config::BOT_CONFIG,
    enums::{CbActions, Top10Variant},
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

    let keyboard = [[InlineKeyboardButton::callback(
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

    let button = InlineKeyboardButton::callback(
        lng("InlineDayPigButton", ltag),
        coded_data,
    );
    let keyboard = [[button]];

    InlineKeyboardMarkup::new(keyboard)
}

pub fn keyboard_day_pig_to_inline(ltag: LocaleTag) -> InlineKeyboardMarkup {
    let button = InlineKeyboardButton::switch_inline_query(
        lng("InlineDayPigButton", ltag),
        lng("InlineMenuButtonDayPigSwitch", ltag),
    );

    let keyboard = [[button]];

    InlineKeyboardMarkup::new(keyboard)
}

pub fn keyboard_add_inline_top10(
    ltag: LocaleTag,
    id_user: UserId,
) -> InlineKeyboardMarkup {
    let coded_data = encode_callback_data(CbActions::AddChat, id_user, "");

    let keyboard = [[InlineKeyboardButton::callback(
        lng("InlineAddTop10ChatButton", ltag),
        coded_data,
    )]];

    InlineKeyboardMarkup::new(keyboard)
}

pub fn keyboard_in_top10(
    ltag: LocaleTag,
    id_user: UserId,
    to: Top10Variant,
) -> InlineKeyboardMarkup {
    let coded_data =
        encode_callback_data(CbActions::Top10, id_user, to.as_ref());

    let summarized = to.summarize();
    let key = summarized.as_ref();

    let text = lng(&format!("InlineTop10ButtonIn_{}", &key), ltag);
    let button = InlineKeyboardButton::callback(text, coded_data);

    let keyboard = [[button]];

    InlineKeyboardMarkup::new(keyboard)
}

pub fn keyboard_start_duel(
    ltag: LocaleTag,
    id_user: UserId,
) -> InlineKeyboardMarkup {
    let coded_data = encode_callback_data(CbActions::StartDuel, id_user, "");

    let text = lng("InlineDuelStartButton", ltag);
    let button = InlineKeyboardButton::callback(text, coded_data);
    let keyboard = [[button]];

    InlineKeyboardMarkup::new(keyboard)
}

pub fn keyboard_startgroup(ltag: LocaleTag) -> InlineKeyboardMarkup {
    let button = _button_startgroup(ltag);
    let keyboard = [[button]];

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

    let keyboard = [row];

    InlineKeyboardMarkup::new(keyboard)
}

pub fn keyboard_voice_check(id_user: UserId) -> InlineKeyboardMarkup {
    let success_data = encode_callback_data(CbActions::AllowVoice, id_user, "");
    let success_button = InlineKeyboardButton::callback("✅", success_data);
    let denied_data =
        encode_callback_data(CbActions::DisallowVoice, id_user, "");
    let denied_button = InlineKeyboardButton::callback("❌", denied_data);

    let keyboard = [[success_button, denied_button]];

    InlineKeyboardMarkup::new(keyboard)
}

pub fn keyboard_gif_check(id_user: UserId) -> InlineKeyboardMarkup {
    let success_data =
        encode_callback_data(CbActions::GifDecision, id_user, "+");
    let success_button = InlineKeyboardButton::callback("✅", success_data);

    let denied_data =
        encode_callback_data(CbActions::GifDecision, id_user, "-");
    let denied_button = InlineKeyboardButton::callback("❌", denied_data);

    let keyboard = [[success_button, denied_button]];

    InlineKeyboardMarkup::new(keyboard)
}

pub fn keyboard_change_flag(
    ltag: LocaleTag,
    id_user: UserId,
    flag_code: &str,
) -> InlineKeyboardMarkup {
    let data = encode_callback_data(CbActions::ChangeFlag, id_user, flag_code);

    let text = lng("InlineLangChangeButton", ltag);
    let keyboard = [[InlineKeyboardButton::callback(text, data)]];

    InlineKeyboardMarkup::new(keyboard)
}

pub fn keyboard_change_lang(
    ltag: LocaleTag,
    id_user: UserId,
    flag_code: &str,
) -> InlineKeyboardMarkup {
    let data = encode_callback_data(CbActions::ChangeLang, id_user, flag_code);

    let button_key = if flag_code == "-" {
        "InlineLangDeleteButton"
    } else {
        "InlineLangChangeButton"
    };
    let text = lng(button_key, ltag);
    let keyboard = [[InlineKeyboardButton::callback(text, data)]];

    InlineKeyboardMarkup::new(keyboard)
}

macro_rules! make_switch_buttons {
    ($ltag:expr, [$($input:expr),* $(,)?]) => {
        [$({
            let lng_key = concat!("InlineMenuButton", $input);
            let lng_switch_key = concat!("InlineMenuButton", $input, "Switch");
            let text = $crate::lang::lng(lng_key, $ltag);
            let switch_query = $crate::lang::lng(lng_switch_key, $ltag);
            teloxide::types::InlineKeyboardButton::switch_inline_query_current_chat(text, switch_query)
        }),*]
    };
}

pub fn keyboard_more_info(ltag: LocaleTag) -> InlineKeyboardMarkup {
    let [chg_name_button, chg_flag_button, chg_lang_button, pig_day_button, oc_button, hru_button, gifs_button] = make_switch_buttons!(
        ltag,
        [
            "ChangeHandPigName",
            "ChangeFlag",
            "ChangeLang",
            "DayPig",
            "OC",
            "HearHruks",
            "PigGifs"
        ]
    );

    let startgroup_button = _button_startgroup(ltag);

    let keyboard = [
        vec![chg_name_button, chg_flag_button],
        vec![chg_lang_button, pig_day_button],
        vec![hru_button, oc_button],
        vec![gifs_button],
        vec![startgroup_button],
    ];

    InlineKeyboardMarkup::new(keyboard)
}

pub fn keyboard_empty() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::default()
}

pub fn keyboard_link_to_chat(ltag: LocaleTag) -> InlineKeyboardMarkup {
    let text = lng("InlineDuelChatButton", ltag);
    let chat_url =
        format!("https://t.me/{}", &BOT_CONFIG.chat_link).parse().unwrap();
    let button = InlineKeyboardButton::url(text, chat_url);

    InlineKeyboardMarkup::new([[button]])
}

fn _button_startgroup(ltag: LocaleTag) -> InlineKeyboardButton {
    let url = BOT_CONFIG.me.tme_url();
    let url = url.join("?startgroup=inline").unwrap();
    let text = lng("BotAddToGroup", ltag);
    InlineKeyboardButton::url(text, url)
}
