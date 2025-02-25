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

enum Actions<'a> {
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

pub async fn handle_message(bot: MyBot, m: Message) -> MyResult<()> {
    crate::metrics::MESSAGE_COUNTER.inc();
    let text_lower = m.text().unwrap().to_lowercase();
    let text_str = text_lower.as_str();

    let maybe_action = match text_str {
        "хорни" | "horny" => {
            Actions::MaybeRText(italic("go to horny jail."))
        },
        "пацєтко" => Actions::MaybeRText(bold("пацєтко сє вродило")),
        "@fr0staman_bot" => Actions::Photo(PHOTO),
        _ => Actions::None,
    };

    if let Actions::None = maybe_action {
    } else {
        _maybe_send_message(bot, m, maybe_action).await?;
        return Ok(());
    }

    let probably_action = if m.reply_to_message().is_some() {
        match text_str {
            "бдсм" | "bdsm" => Actions::RSticker(STICKER),
            "хрюкни" | "grunt" => Actions::RVoice(GRUNT),
            _ => Actions::None,
        }
    } else {
        Actions::None
    };

    _maybe_send_message(bot, m, probably_action).await?;

    Ok(())
}

async fn _maybe_send_message(
    bot: MyBot,
    m: Message,
    action: Actions<'_>,
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
            bot.send_sticker(m.chat.id, InputFile::file_id(file_id))
                .reply_parameters(ReplyParameters::new(
                    m.reply_to_message().unwrap().id,
                ))
                .maybe_thread_id(&m)
                .await?;
            log::info!("Handled message with sticker: chat [{}]", m.chat.id);
            crate::metrics::MESSAGE_HANDLED_COUNTER.inc();
        },
        Actions::RVoice(file_id) => {
            bot.send_voice(m.chat.id, InputFile::file_id(file_id))
                .reply_parameters(ReplyParameters::new(
                    m.reply_to_message().unwrap().id,
                ))
                .maybe_thread_id(&m)
                .await?;
            log::info!("Handled message with voice: chat [{}]", m.chat.id);
            crate::metrics::MESSAGE_HANDLED_COUNTER.inc();
        },
        Actions::Photo(file_id) => {
            bot.send_photo(m.chat.id, InputFile::file_id(file_id))
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
