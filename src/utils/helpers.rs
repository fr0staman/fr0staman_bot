use diesel::{debug_query, pg::Pg, query_builder::DebugQuery};
use futures::TryStreamExt;
use std::iter;
use teloxide::{
    net::Download,
    types::{ChatKind, PublicChatKind, UserId},
    utils::html,
};
use unicode_width::UnicodeWidthChar;

use std::hash::Hasher;
use url::Url;

use crate::{
    config::consts::{
        HAND_PIG_ADDITION_ON_SUBSCRIBED, HAND_PIG_ADDITION_ON_SUPPORTED,
    },
    config::env::BOT_CONFIG,
    db::models::User,
    enums::{CbActions, Image},
    types::{MyBot, ParsedCallbackData},
};

const SEPARATOR: char = ':';

/// Telegram rejects `callback_data` over this many bytes.
pub const CALLBACK_DATA_LIMIT: usize = 64;

/// `action:user_id:payload`, with the payload truncated to fit the limit.
pub fn encode_callback_data<U>(
    action: CbActions,
    id_user: UserId,
    second: U,
) -> String
where
    U: Into<String>,
{
    let mut encoded = String::with_capacity(CALLBACK_DATA_LIMIT);

    encoded.push_str(action.into_str());
    encoded.push(SEPARATOR);
    encoded.push_str(&id_user.to_string());
    encoded.push(SEPARATOR);

    let payload = second.into();
    let room = CALLBACK_DATA_LIMIT.saturating_sub(encoded.len());
    encoded.push_str(truncate_bytes(&payload, room));

    encoded
}

/// Payload bytes available after the action and user id. Callers shorten
/// user text with this before rendering, so the preview matches what is
/// stored.
pub fn callback_payload_room(action: CbActions, id_user: UserId) -> usize {
    let prefix = action.into_str().len() + 1 + id_user.to_string().len() + 1;

    CALLBACK_DATA_LIMIT.saturating_sub(prefix)
}

/// Longest prefix of `s` within `max_bytes` that is still valid UTF-8.
pub fn truncate_bytes(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }

    let mut cut = max_bytes;
    while cut > 0 && !s.is_char_boundary(cut) {
        cut -= 1;
    }

    &s[..cut]
}

pub fn decode_callback_data(data: &str) -> Option<ParsedCallbackData<'_>> {
    let splitted: Vec<&str> = data.splitn(3, SEPARATOR).collect();

    if splitted.len() < 3 {
        return None;
    }

    let action = splitted[0];
    let Ok(id_user) = splitted[1].parse::<u64>() else {
        return None;
    };

    let payload = splitted[2];

    Some((action, UserId(id_user), payload))
}

/// `None` rather than a fallback: every call site used to `.unwrap_or(1)`,
/// merging unrelated chats into one inline group.
pub fn parse_chat_instance(raw: &str) -> Option<i64> {
    match raw.parse::<i64>() {
        Ok(parsed) => Some(parsed),
        Err(_) => {
            log::warn!("Unparseable chat_instance [{raw}], ignoring");
            None
        },
    }
}

pub fn get_photostock(append: Image) -> Url {
    let url = &BOT_CONFIG.photostock_url;

    url.join(append.into_str()).unwrap_or_else(|_| url.clone())
}

// Truncate to some width with emoji check, because it can be 2 bytes or even 4 bytes
pub fn truncate(s: &str, width: usize) -> (&str, usize) {
    let (bidx, new_width) = s
        .char_indices()
        .map(|(bidx, c)| (bidx, c.width().unwrap_or(0)))
        .chain(iter::once((s.len(), 0)))
        .scan(0, |w, (bidx, cw)| {
            let curr_w = *w;
            *w += cw;
            Some((bidx, curr_w))
        })
        .take_while(|&(_, w)| w <= width)
        .last()
        .unwrap_or((0, 0));
    (s.get(..bidx).unwrap(), new_width)
}

// Purpose: for logging
pub fn get_chat_kind(kind: &ChatKind) -> &str {
    match kind {
        ChatKind::Public(kind) => match kind.kind {
            PublicChatKind::Channel(_) => "channel",
            PublicChatKind::Group => "group",
            PublicChatKind::Supergroup(_) => "supergroup",
        },
        ChatKind::Private(_) => "private",
    }
}

pub fn get_hash<T>(value: T) -> u64
where
    T: std::hash::Hash,
{
    let mut hasher = ahash::AHasher::default();
    value.hash(&mut hasher);
    hasher.finish()
}

pub fn escape(s: &str) -> String {
    escape_links(&html::escape(s).replace('@', ""))
}

const LINK_MARKERS: [&str; 4] = ["t.me", "telegram.me", "http://", "https://"];

/// Repeats until stable: one pass lets a nested marker reassemble itself,
/// e.g. `htthttps://ps://x` collapsing back into `https://x`.
pub fn escape_links(s: &str) -> String {
    let mut out = s.to_owned();

    loop {
        let mut pass = out.clone();
        for marker in LINK_MARKERS {
            pass = pass.replace(marker, "");
        }

        if pass == out {
            return out;
        }
        out = pass;
    }
}

// Plural rule for languages, may some inaccurate - but it works!
pub fn plural(n: i64) -> i64 {
    // `%` keeps the sign of the dividend, so negatives used to fall through
    // to "many". `unsigned_abs` over `abs` so `i64::MIN` cannot overflow.
    let magnitude = n.unsigned_abs();
    let last_digit = magnitude % 10;
    let last_two = magnitude % 100;

    if last_digit == 1 && last_two != 11 {
        0
    } else if (2..=4).contains(&last_digit) && !(10..20).contains(&last_two) {
        1
    } else {
        2
    }
}

#[allow(unused)]
pub fn db_debug<T>(query: &T) -> DebugQuery<'_, T, Pg> {
    debug_query::<Pg, _>(query)
}

pub fn mass_addition_on_status(user: &User) -> i32 {
    if user.supported {
        HAND_PIG_ADDITION_ON_SUPPORTED
    } else if user.subscribed {
        HAND_PIG_ADDITION_ON_SUBSCRIBED
    } else {
        0
    }
}

pub async fn get_file_from_stream(
    bot: &MyBot,
    file: &teloxide::types::File,
) -> Option<bytes::Bytes> {
    bot.download_file_stream(&file.path)
        .try_collect()
        .await
        .map(bytes::BytesMut::freeze)
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::user as test_user;
    use std::str::FromStr;


    #[test]
    fn callback_data_round_trips() {
        let uid = UserId(123_456_789);
        let encoded = encode_callback_data(CbActions::StartDuel, uid, "abc");

        assert_eq!(encoded, "start_duel:123456789:abc");
        assert_eq!(
            decode_callback_data(&encoded),
            Some(("start_duel", uid, "abc"))
        );
    }

    #[test]
    fn callback_data_round_trips_for_every_action() {
        use strum::VariantArray;

        let uid = UserId(42);

        for action in CbActions::VARIANTS {
            let encoded = encode_callback_data(*action, uid, "payload");
            let decoded = decode_callback_data(&encoded)
                .unwrap_or_else(|| panic!("{action:?} failed to decode"));

            assert_eq!(decoded.0, action.into_str());
            assert_eq!(decoded.1, uid);
            assert_eq!(decoded.2, "payload");
            assert_eq!(CbActions::from_str(decoded.0).unwrap(), *action);
        }
    }

    #[test]
    fn an_empty_payload_still_decodes() {
        let uid = UserId(7);
        let encoded = encode_callback_data(CbActions::Top10, uid, "");

        assert_eq!(encoded, "top10:7:");
        assert_eq!(decode_callback_data(&encoded), Some(("top10", uid, "")));
    }

    #[test]
    fn a_payload_may_contain_the_separator() {
        let uid = UserId(7);
        let encoded = encode_callback_data(CbActions::ChangeLang, uid, "a:b:c");

        assert_eq!(
            decode_callback_data(&encoded),
            Some(("change_lang", uid, "a:b:c"))
        );
    }

    #[test]
    fn malformed_callback_data_is_rejected() {
        for bad in ["", "a", "a:b", "a:notanumber:c", "a:-1:c", ":::"] {
            let decoded = decode_callback_data(bad);
            if bad == ":::" {
                // Three empty fields parse structurally but the id does not.
                assert_eq!(decoded, None, "{bad:?}");
            } else {
                assert_eq!(decoded, None, "{bad:?}");
            }
        }
    }

    #[test]
    fn a_long_pig_name_is_truncated_to_fit_the_callback_limit() {
        // A 20-char Cyrillic name is 40 bytes, which used to push the whole
        // payload past Telegram's 64-byte limit.
        let uid = UserId(u64::MAX);
        let name = "и".repeat(crate::config::consts::INLINE_NAME_SET_LIMIT);

        let encoded = encode_callback_data(CbActions::GiveName, uid, name);

        assert!(
            encoded.len() <= CALLBACK_DATA_LIMIT,
            "{} bytes: {encoded}",
            encoded.len()
        );
        let decoded = decode_callback_data(&encoded).expect("did not decode");
        assert_eq!(decoded.0, "give_name");
        assert_eq!(decoded.1, uid);
        assert!(!decoded.2.is_empty());
    }

    #[test]
    fn truncation_never_splits_a_multibyte_character() {
        // The cut lands on a UTF-8 boundary, so the payload stays valid and
        // the name is not corrupted mid-character.
        for uid in [UserId(1), UserId(123_456_789), UserId(u64::MAX)] {
            for len in 1..40 {
                let name = "🐷".repeat(len);
                let encoded =
                    encode_callback_data(CbActions::GiveName, uid, name);

                assert!(encoded.len() <= CALLBACK_DATA_LIMIT);
                let payload = decode_callback_data(&encoded).unwrap().2;
                assert!(payload.chars().all(|c| c == '🐷'), "{payload:?}");
            }
        }
    }

    #[test]
    fn every_action_and_user_id_produces_a_payload_within_the_limit() {
        use strum::VariantArray;

        let long = "x".repeat(200);

        for action in CbActions::VARIANTS {
            for uid in [UserId(1), UserId(u64::MAX)] {
                let encoded =
                    encode_callback_data(*action, uid, long.as_str());
                assert!(
                    encoded.len() <= CALLBACK_DATA_LIMIT,
                    "{action:?}/{uid}: {} bytes",
                    encoded.len()
                );
            }
        }
    }

    #[test]
    fn a_short_name_is_left_untouched() {
        let uid = UserId(123_456_789);
        let encoded =
            encode_callback_data(CbActions::GiveName, uid, "Pig McPigface");

        assert!(encoded.len() <= CALLBACK_DATA_LIMIT);
        assert_eq!(decode_callback_data(&encoded).unwrap().2, "Pig McPigface");
    }

    #[test]
    fn the_payload_room_matches_what_encoding_actually_leaves() {
        // `rename_hryak_info` shortens the displayed name with this budget,
        // so it has to agree with the encoder or the preview and the stored
        // name would diverge.
        for uid in [UserId(1), UserId(999), UserId(u64::MAX)] {
            let room = callback_payload_room(CbActions::GiveName, uid);
            let name = "a".repeat(room);

            let encoded = encode_callback_data(CbActions::GiveName, uid, name);

            assert_eq!(encoded.len(), CALLBACK_DATA_LIMIT);
            assert_eq!(decode_callback_data(&encoded).unwrap().2.len(), room);
        }
    }

    #[test]
    fn truncate_bytes_cuts_on_a_character_boundary() {
        assert_eq!(truncate_bytes("abcdef", 3), "abc");
        assert_eq!(truncate_bytes("abc", 10), "abc");
        assert_eq!(truncate_bytes("", 5), "");
        assert_eq!(truncate_bytes("иии", 4), "ии");
        assert_eq!(truncate_bytes("иии", 3), "и");
        assert_eq!(truncate_bytes("иии", 1), "");
        assert_eq!(truncate_bytes("🐷🐷", 7), "🐷");
        assert_eq!(truncate_bytes("🐷🐷", 8), "🐷🐷");
    }


    #[test]
    fn a_numeric_chat_instance_parses() {
        assert_eq!(parse_chat_instance("7777777"), Some(7_777_777));
        assert_eq!(parse_chat_instance("-1234567890123"), Some(-1_234_567_890_123));
        assert_eq!(parse_chat_instance("0"), Some(0));
        assert_eq!(parse_chat_instance("1"), Some(1));
    }

    #[test]
    fn an_unparseable_chat_instance_is_rejected_not_bucketed() {
        // These used to `.unwrap_or(1)`, quietly merging every unparseable
        // instance bot-wide into inline group 1.
        for junk in [
            "",
            "not-a-number",
            "abc",
            " 1",
            "1 ",
            "1.5",
            "99999999999999999999999",
            "-99999999999999999999999",
        ] {
            assert_eq!(parse_chat_instance(junk), None, "{junk:?}");
        }
    }


    #[test]
    fn truncate_leaves_short_strings_alone() {
        assert_eq!(truncate("abc", 10), ("abc", 3));
        assert_eq!(truncate("", 10), ("", 0));
        assert_eq!(truncate("abc", 3), ("abc", 3));
    }

    #[test]
    fn truncate_cuts_on_display_width() {
        assert_eq!(truncate("abcdef", 3), ("abc", 3));
        // Cyrillic is one column per char.
        assert_eq!(truncate("привіт", 3), ("при", 3));
    }

    #[test]
    fn truncate_never_splits_a_wide_character() {
        // CJK is two columns, so an odd budget leaves one column unused.
        assert_eq!(truncate("漢字漢字", 4), ("漢字", 4));
        assert_eq!(truncate("漢字漢字", 3), ("漢", 2));
        assert_eq!(truncate("漢字", 1), ("", 0));
    }

    #[test]
    fn truncate_handles_emoji() {
        let (cut, width) = truncate("🐷🐷🐷", 4);
        assert_eq!(cut, "🐷🐷");
        assert_eq!(width, 4);
        assert!(cut.is_char_boundary(cut.len()));
    }

    #[test]
    fn truncate_with_a_zero_budget_yields_nothing() {
        assert_eq!(truncate("abc", 0), ("", 0));
    }

    #[test]
    fn truncate_always_returns_valid_utf8_within_the_budget() {
        let samples =
            ["", "a", "漢字", "🐷x🐷", "приве́т", "a\u{200d}b", "🇺🇦"];

        for s in samples {
            for width in 0..12 {
                let (cut, w) = truncate(s, width);
                assert!(s.starts_with(cut), "{s:?}/{width}");
                assert!(w <= width, "{s:?}/{width} -> {w}");
            }
        }
    }


    #[test]
    fn escape_strips_html_mentions_and_links() {
        assert_eq!(escape("<b>hi</b>"), "&lt;b&gt;hi&lt;/b&gt;");
        assert_eq!(escape("@someone"), "someone");
        assert_eq!(escape("https://t.me/chat"), "/chat");
    }

    #[test]
    fn escape_links_removes_each_marker() {
        assert_eq!(escape_links("t.me/x"), "/x");
        assert_eq!(escape_links("telegram.me/x"), "/x");
        assert_eq!(escape_links("http://x"), "x");
        assert_eq!(escape_links("https://x"), "x");
        assert_eq!(escape_links("nothing here"), "nothing here");
    }

    #[test]
    fn escape_links_strips_nested_markers() {
        // A single `replace` pass let these reassemble into working links.
        assert_eq!(escape_links("htthttps://ps://x"), "x");
        assert_eq!(escape_links("tt.me.me/x"), "/x");

        // Deeper nesting does not always collapse to nothing, but it always
        // reaches a fixpoint with no usable marker left in it.
        let messy = escape_links("htthttthttps://ps://ps://x");
        for marker in LINK_MARKERS {
            assert!(!messy.contains(marker), "{messy:?} kept {marker}");
        }
    }

    #[test]
    fn escape_links_terminates_and_removes_every_marker() {
        let samples = [
            "",
            "clean text",
            "t.me",
            "tt.me.me",
            "https://https://",
            "htthttps://ps://t.mtt.me.mee",
            &"t.me".repeat(50),
            &"htt".repeat(30),
        ];

        for sample in samples {
            let out = escape_links(sample);
            for marker in LINK_MARKERS {
                assert!(
                    !out.contains(marker),
                    "{sample:?} -> {out:?} still contains {marker}"
                );
            }
        }
    }

    #[test]
    fn escape_links_is_idempotent() {
        for sample in ["htthttps://ps://x", "tt.me.me/x", "plain"] {
            let once = escape_links(sample);
            assert_eq!(escape_links(&once), once, "{sample:?}");
        }
    }


    #[test]
    fn plural_follows_the_slavic_rule() {
        let ones = [1, 21, 31, 101, 121];
        let few = [2, 3, 4, 22, 23, 24, 102];
        let many = [0, 5, 6, 9, 10, 11, 12, 13, 14, 15, 19, 20, 25, 100, 111];

        for n in ones {
            assert_eq!(plural(n), 0, "n = {n}");
        }
        for n in few {
            assert_eq!(plural(n), 1, "n = {n}");
        }
        for n in many {
            assert_eq!(plural(n), 2, "n = {n}");
        }
    }

    #[test]
    fn plural_only_ever_returns_zero_one_or_two() {
        for n in 0..1_000i64 {
            assert!((0..=2).contains(&plural(n)), "n = {n}");
        }
    }

    #[test]
    fn plural_treats_negatives_by_magnitude() {
        // Rust's `%` keeps the sign of the dividend, so every negative input
        // used to fall through to the "many" bucket.
        for n in 1..500i64 {
            assert_eq!(plural(-n), plural(n), "n = {n}");
        }

        assert_eq!(plural(-1), 0);
        assert_eq!(plural(-2), 1);
        assert_eq!(plural(-11), 2);
        assert_eq!(plural(-21), 0);
    }

    #[test]
    fn plural_handles_the_extremes_without_overflowing() {
        // `unsigned_abs`, not `abs`: `i64::MIN.abs()` would panic.
        for n in [i64::MIN, i64::MIN + 1, i64::MAX, 0] {
            assert!((0..=2).contains(&plural(n)), "n = {n}");
        }
    }

    #[test]
    fn plural_is_unchanged_for_the_values_callers_actually_pass() {
        // `get_timediff` yields 0..=24 hours and 0..60 minutes/seconds.
        let expected: Vec<i64> = (0..=60)
            .map(|n: i64| {
                if n % 10 == 1 && n % 100 != 11 {
                    0
                } else if (2..=4).contains(&(n % 10))
                    && (n % 100 < 10 || n % 100 >= 20)
                {
                    1
                } else {
                    2
                }
            })
            .collect();

        for (n, want) in expected.iter().enumerate() {
            assert_eq!(plural(n as i64), *want, "n = {n}");
        }
    }


    #[test]
    fn hashing_is_stable_within_a_run() {
        assert_eq!(get_hash("abc"), get_hash("abc"));
        assert_ne!(get_hash("abc"), get_hash("abd"));
    }

    #[test]
    fn status_bonus_prefers_supporter_over_subscriber() {
        let mut user = test_user(1, 1);
        assert_eq!(mass_addition_on_status(&user), 0);

        user.subscribed = true;
        assert_eq!(
            mass_addition_on_status(&user),
            crate::config::consts::HAND_PIG_ADDITION_ON_SUBSCRIBED
        );

        user.supported = true;
        assert_eq!(
            mass_addition_on_status(&user),
            crate::config::consts::HAND_PIG_ADDITION_ON_SUPPORTED
        );

        user.subscribed = false;
        assert_eq!(
            mass_addition_on_status(&user),
            crate::config::consts::HAND_PIG_ADDITION_ON_SUPPORTED
        );
    }
}
