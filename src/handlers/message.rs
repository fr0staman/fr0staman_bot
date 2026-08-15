use crate::{
    traits::{
        MaybeMessageSetter, MaybePhotoSetter, MaybeStickerSetter,
        MaybeVoiceSetter,
    },
    types::{MyBot, MyResult},
};

use teloxide::{
    prelude::*,
    types::{InputFile, Message, ReplyParameters},
    utils::html::{bold, italic},
};

#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub enum Actions<'a> {
    #[allow(dead_code)]
    Text(String),
    MaybeRText(String),
    Photo(&'a str),
    RSticker(&'a str),
    RVoice(&'a str),
    None,
}

const PHOTO: &str = "AgACAgIAAxkBAAEkwLhk0ol5jagMJ58v6y1PuRh4pG2OAwACG8wxG-7GkUodLEo13wzrWAEAAwIAA3gAAzAE";
const STICKER: &str = "CAACAgIAAx0CWjbDqQACJqVhBXDPsHT3uscpSlWcQTQxhjgetgACdAEAAntOKhC7YDsAAWimi98gBA";
const GRUNT: &str =
    "AwACAgIAAxkBAAIfv2Eep89pMun_Qq3u-o_UdS997Bx9AAIsEgACErX4SBRdIvQwnUdhIAQ";

/// `text` is expected already lowercased.
pub fn match_keyword(text: &str) -> Actions<'static> {
    match text {
        "хорни" | "horny" => Actions::MaybeRText(italic("go to horny jail.")),
        "пацєтко" => Actions::MaybeRText(bold("пацєтко сє вродило")),
        "@fr0staman_bot" => Actions::Photo(PHOTO),
        _ => Actions::None,
    }
}

/// Keywords that only mean something as a reply.
pub fn match_reply_keyword(text: &str) -> Actions<'static> {
    match text {
        "бдсм" | "bdsm" => Actions::RSticker(STICKER),
        "хрюкни" | "grunt" => Actions::RVoice(GRUNT),
        _ => Actions::None,
    }
}

pub async fn handle_message(bot: MyBot, m: Message) -> MyResult<()> {
    crate::metrics::MESSAGE_COUNTER.inc();
    // Safe: the dispatch tree only routes here when `text()` is `Some`.
    let text_lower = m.text().unwrap_or_default().to_lowercase();
    let text_str = text_lower.as_str();

    let maybe_action = match_keyword(text_str);

    if let Actions::None = maybe_action {
    } else {
        _maybe_send_message(bot, m, maybe_action).await?;
        return Ok(());
    }

    let probably_action = if m.reply_to_message().is_some() {
        match_reply_keyword(text_str)
    } else {
        Actions::None
    };

    _maybe_send_message(bot, m, probably_action).await?;

    Ok(())
}

async fn _maybe_send_message(
    bot: MyBot,
    m: Message,
    action: Actions<'static>,
) -> MyResult<()> {
    match action {
        Actions::Text(text) => {
            bot.send_message(m.chat.id, text)
                .reply_parameters(ReplyParameters::new(m.id))
                .maybe_thread_id(&m)
                .await?;
            log::info!("Handled message: chat [{}]", m.chat.id);
            crate::metrics::MESSAGE_HANDLED_COUNTER.inc();
        },
        Actions::MaybeRText(text) => {
            bot.send_message(m.chat.id, text)
                .reply_parameters(ReplyParameters::new(
                    m.reply_to_message().map_or(m.id, |v| v.id),
                ))
                .maybe_thread_id(&m)
                .await?;

            log::info!("Handled message: chat [{}]", m.chat.id);
            crate::metrics::MESSAGE_HANDLED_COUNTER.inc();
        },
        Actions::RSticker(file_id) => {
            bot.send_sticker(m.chat.id, InputFile::file_id(file_id.into()))
                .reply_parameters(ReplyParameters::new(
                    m.reply_to_message().unwrap().id,
                ))
                .maybe_thread_id(&m)
                .await?;
            log::info!("Handled message with sticker: chat [{}]", m.chat.id);
            crate::metrics::MESSAGE_HANDLED_COUNTER.inc();
        },
        Actions::RVoice(file_id) => {
            bot.send_voice(m.chat.id, InputFile::file_id(file_id.into()))
                .reply_parameters(ReplyParameters::new(
                    m.reply_to_message().unwrap().id,
                ))
                .maybe_thread_id(&m)
                .await?;
            log::info!("Handled message with voice: chat [{}]", m.chat.id);
            crate::metrics::MESSAGE_HANDLED_COUNTER.inc();
        },
        Actions::Photo(file_id) => {
            bot.send_photo(m.chat.id, InputFile::file_id(file_id.into()))
                .reply_parameters(ReplyParameters::new(m.id))
                .maybe_thread_id(&m)
                .await?;
            log::info!("Handled message with image: chat [{}]", m.chat.id);
            crate::metrics::MESSAGE_HANDLED_COUNTER.inc();
        },
        Actions::None => {
            log::info!("Unhandled message: chat [{}]", m.chat.id)
        },
    };

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standalone_keywords_are_recognised() {
        assert!(matches!(match_keyword("хорни"), Actions::MaybeRText(_)));
        assert!(matches!(match_keyword("horny"), Actions::MaybeRText(_)));
        assert!(matches!(match_keyword("пацєтко"), Actions::MaybeRText(_)));
        assert_eq!(match_keyword("@fr0staman_bot"), Actions::Photo(PHOTO));
    }

    #[test]
    fn reply_only_keywords_are_recognised() {
        assert_eq!(match_reply_keyword("бдсм"), Actions::RSticker(STICKER));
        assert_eq!(match_reply_keyword("bdsm"), Actions::RSticker(STICKER));
        assert_eq!(match_reply_keyword("хрюкни"), Actions::RVoice(GRUNT));
        assert_eq!(match_reply_keyword("grunt"), Actions::RVoice(GRUNT));
    }

    #[test]
    fn the_two_keyword_sets_do_not_overlap() {
        for word in ["хорни", "horny", "пацєтко", "@fr0staman_bot"] {
            assert_eq!(match_reply_keyword(word), Actions::None, "{word}");
        }
        for word in ["бдсм", "bdsm", "хрюкни", "grunt"] {
            assert_eq!(match_keyword(word), Actions::None, "{word}");
        }
    }

    #[test]
    fn anything_else_matches_nothing() {
        for word in ["", "hello", "хорни ", " horny", "хорниии"] {
            assert_eq!(match_keyword(word), Actions::None, "{word:?}");
            assert_eq!(match_reply_keyword(word), Actions::None, "{word:?}");
        }
    }

    #[test]
    fn matching_is_case_sensitive_because_callers_lowercase_first() {
        // `handle_message` lowercases before calling in.
        assert_eq!(match_keyword("HORNY"), Actions::None);
        assert!(matches!(
            match_keyword(&"HORNY".to_lowercase()),
            Actions::MaybeRText(_)
        ));
    }
}
