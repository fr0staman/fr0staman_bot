//! Neither the button layout nor the `callback_data` payload is checked by
//! the compiler.

use std::str::FromStr;

use crate::{
    enums::{CbActions, Top10Variant},
    keyboards,
    lang::LocaleTag,
    test_support::init_all,
    utils::helpers::decode_callback_data,
};
use teloxide::types::{
    InlineKeyboardButton, InlineKeyboardButtonKind, InlineKeyboardMarkup,
    UserId,
};

const USER: UserId = UserId(123_456_789);

fn setup() -> LocaleTag {
    init_all()
}

fn rows(markup: &InlineKeyboardMarkup) -> &Vec<Vec<InlineKeyboardButton>> {
    &markup.inline_keyboard
}

fn only_button(markup: &InlineKeyboardMarkup) -> &InlineKeyboardButton {
    let rows = rows(markup);
    assert_eq!(rows.len(), 1, "expected a single row");
    assert_eq!(rows[0].len(), 1, "expected a single button");
    &rows[0][0]
}

fn callback_payload(button: &InlineKeyboardButton) -> &str {
    match &button.kind {
        InlineKeyboardButtonKind::CallbackData(data) => data,
        other => panic!("expected a callback button, got {other:?}"),
    }
}

/// Returns the payload tail.
fn assert_action(
    button: &InlineKeyboardButton,
    action: CbActions,
) -> String {
    let raw = callback_payload(button);
    let decoded = decode_callback_data(raw)
        .unwrap_or_else(|| panic!("{raw} did not decode"));

    assert_eq!(CbActions::from_str(decoded.0).unwrap(), action);
    assert_eq!(decoded.1, USER);

    decoded.2.to_owned()
}


#[test]
fn single_button_keyboards_carry_a_decodable_payload() {
    let ltag = setup();

    let cases: Vec<(InlineKeyboardMarkup, CbActions, &str)> = vec![
        (keyboards::keyboard_day_pig(ltag, USER), CbActions::FindHryak, ""),
        (
            keyboards::keyboard_add_inline_top10(ltag, USER),
            CbActions::AddChat,
            "",
        ),
        (keyboards::keyboard_start_duel(ltag, USER), CbActions::StartDuel, ""),
        (keyboards::keyboard_reset_vote(ltag, USER), CbActions::ResetVote, ""),
        (
            keyboards::keyboard_change_flag(ltag, USER, "ua"),
            CbActions::ChangeFlag,
            "ua",
        ),
        (
            keyboards::keyboard_change_lang(ltag, USER, "uk"),
            CbActions::ChangeLang,
            "uk",
        ),
        (
            keyboards::keyboard_new_name(ltag, USER, "Hrundel".to_owned()),
            CbActions::GiveName,
            "Hrundel",
        ),
    ];

    for (markup, action, expected_payload) in cases {
        let payload = assert_action(only_button(&markup), action);
        assert_eq!(payload, expected_payload, "{action:?}");
    }
}

#[test]
fn the_top10_button_encodes_the_variant_it_switches_to() {
    let ltag = setup();

    for name in ["global", "chat", "win", "p_global", "p_win"] {
        let to = Top10Variant::from_str(name).unwrap();
        let markup = keyboards::keyboard_in_top10(ltag, USER, to);

        let payload = assert_action(only_button(&markup), CbActions::Top10);
        assert_eq!(payload, name);
    }
}

#[test]
fn the_moderation_keyboards_offer_accept_and_reject() {
    setup();

    let voice = keyboards::keyboard_voice_check(USER);
    let voice_rows = rows(&voice);
    assert_eq!(voice_rows.len(), 1);
    assert_eq!(voice_rows[0].len(), 2);
    assert_eq!(voice_rows[0][0].text, "✅");
    assert_eq!(voice_rows[0][1].text, "❌");
    assert_action(&voice_rows[0][0], CbActions::AllowVoice);
    assert_action(&voice_rows[0][1], CbActions::DisallowVoice);

    let gif = keyboards::keyboard_gif_check(USER);
    let gif_rows = rows(&gif);
    assert_eq!(gif_rows[0].len(), 2);
    assert_eq!(assert_action(&gif_rows[0][0], CbActions::GifDecision), "+");
    assert_eq!(assert_action(&gif_rows[0][1], CbActions::GifDecision), "-");
}

#[test]
fn the_lang_keyboard_switches_its_label_for_the_clear_option() {
    let ltag = setup();

    let set = keyboards::keyboard_change_lang(ltag, USER, "uk");
    let clear = keyboards::keyboard_change_lang(ltag, USER, "-");

    assert_ne!(only_button(&set).text, only_button(&clear).text);
    assert_eq!(assert_action(only_button(&clear), CbActions::ChangeLang), "-");
}


#[test]
fn the_first_page_has_no_left_arrow() {
    let ltag = setup();
    let markup = keyboards::keyboard_top(ltag, 1, USER, false);

    let row = &rows(&markup)[0];
    assert_eq!(row.len(), 1, "only the right arrow");
    assert_eq!(assert_action(&row[0], CbActions::TopRight), "2");
}

#[test]
fn a_middle_page_has_both_arrows() {
    let ltag = setup();
    let markup = keyboards::keyboard_top(ltag, 3, USER, false);

    let row = &rows(&markup)[0];
    assert_eq!(row.len(), 2);
    assert_eq!(assert_action(&row[0], CbActions::TopLeft), "2");
    assert_eq!(assert_action(&row[1], CbActions::TopRight), "4");
}

#[test]
fn the_last_page_has_no_right_arrow() {
    let ltag = setup();
    let markup = keyboards::keyboard_top(ltag, 3, USER, true);

    let row = &rows(&markup)[0];
    assert_eq!(row.len(), 1);
    assert_eq!(assert_action(&row[0], CbActions::TopLeft), "2");
}

#[test]
fn a_single_page_top_has_no_arrows_at_all() {
    let ltag = setup();
    let markup = keyboards::keyboard_top(ltag, 1, USER, true);

    assert!(rows(&markup)[0].is_empty());
}


#[test]
fn the_more_info_keyboard_has_its_documented_layout() {
    let ltag = setup();
    let markup = keyboards::keyboard_more_info(ltag);
    let rows = rows(&markup);

    let sizes: Vec<usize> = rows.iter().map(Vec::len).collect();
    assert_eq!(sizes, vec![2, 2, 2, 1, 1]);
    assert!(matches!(
        rows[4][0].kind,
        InlineKeyboardButtonKind::Url(_)
    ));
}

#[test]
fn url_keyboards_build_a_valid_link() {
    let ltag = setup();

    for markup in [
        keyboards::keyboard_startgroup(ltag),
        keyboards::keyboard_link_to_chat(ltag),
    ] {
        match &only_button(&markup).kind {
            InlineKeyboardButtonKind::Url(url) => {
                assert_eq!(url.scheme(), "https");
                assert_eq!(url.host_str(), Some("t.me"));
            },
            other => panic!("expected a URL button, got {other:?}"),
        }
    }
}

#[test]
fn the_startgroup_link_carries_the_deep_link_parameter() {
    let ltag = setup();
    let markup = keyboards::keyboard_startgroup(ltag);

    match &only_button(&markup).kind {
        InlineKeyboardButtonKind::Url(url) => {
            assert_eq!(url.query(), Some("startgroup=inline"));
        },
        other => panic!("{other:?}"),
    }
}

#[test]
fn no_button_label_is_an_unresolved_locale_key() {
    let ltag = setup();

    let all = [
        keyboards::keyboard_day_pig(ltag, USER),
        keyboards::keyboard_day_pig_to_inline(ltag),
        keyboards::keyboard_day_pig_to_inline_current_chat(ltag),
        keyboards::keyboard_add_inline_top10(ltag, USER),
        keyboards::keyboard_start_duel(ltag, USER),
        keyboards::keyboard_startgroup(ltag),
        keyboards::keyboard_top(ltag, 3, USER, false),
        keyboards::keyboard_change_flag(ltag, USER, "ua"),
        keyboards::keyboard_change_lang(ltag, USER, "uk"),
        keyboards::keyboard_more_info(ltag),
        keyboards::keyboard_reset_vote(ltag, USER),
        keyboards::keyboard_link_to_chat(ltag),
        keyboards::keyboard_new_name(ltag, USER, "Pig".to_owned()),
        keyboards::keyboard_in_top10(ltag, USER, Top10Variant::Global),
    ];

    for markup in &all {
        for row in rows(markup) {
            for button in row {
                assert!(
                    !button.text.starts_with("lang:"),
                    "unresolved label: {}",
                    button.text
                );
                assert!(!button.text.is_empty());
            }
        }
    }
}

#[test]
fn the_empty_keyboard_has_no_buttons() {
    assert!(keyboards::keyboard_empty().inline_keyboard.is_empty());
}

#[test]
fn switch_inline_query_buttons_carry_a_query() {
    let ltag = setup();

    for markup in [
        keyboards::keyboard_day_pig_to_inline(ltag),
        keyboards::keyboard_day_pig_to_inline_current_chat(ltag),
    ] {
        let kind = &only_button(&markup).kind;
        let query = match kind {
            InlineKeyboardButtonKind::SwitchInlineQuery(q) => q,
            InlineKeyboardButtonKind::SwitchInlineQueryCurrentChat(q) => q,
            other => panic!("expected a switch button, got {other:?}"),
        };
        assert!(!query.starts_with("lang:"), "{query}");
    }
}

#[test]
fn every_keyboard_payload_stays_within_telegrams_limit() {
    // Telegram rejects `callback_data` over 64 bytes. `keyboard_new_name` is
    // the one that can exceed it, so it is checked separately in the
    // `helpers` unit tests; everything else must fit.
    let ltag = setup();

    let all = [
        keyboards::keyboard_day_pig(ltag, USER),
        keyboards::keyboard_add_inline_top10(ltag, USER),
        keyboards::keyboard_start_duel(ltag, USER),
        keyboards::keyboard_top(ltag, 999_999, USER, false),
        keyboards::keyboard_change_flag(ltag, USER, "ua"),
        keyboards::keyboard_change_lang(ltag, USER, "uk"),
        keyboards::keyboard_reset_vote(ltag, USER),
        keyboards::keyboard_voice_check(USER),
        keyboards::keyboard_gif_check(USER),
        keyboards::keyboard_in_top10(ltag, USER, Top10Variant::PGlobal),
    ];

    for markup in &all {
        for row in rows(markup) {
            for button in row {
                if let InlineKeyboardButtonKind::CallbackData(data) =
                    &button.kind
                {
                    assert!(
                        data.len() <= 64,
                        "{data} is {} bytes",
                        data.len()
                    );
                }
            }
        }
    }
}
