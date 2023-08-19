use crate::{
    MaybeMessageSetter, MaybePhotoSetter, MaybeStickerSetter, MaybeVoiceSetter,
    MyBot, MyResult,
};
use teloxide::types::Message;
use teloxide::{prelude::*, types::InputFile, utils::html::italic};

enum Actions<'a> {
    Text(String),
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
    let text_lower = m.text().unwrap().to_lowercase();
    let text_str = text_lower.as_str();

    let maybe_action = match text_str {
        "хорни" | "horny" => Actions::Text(italic("go to horny jail.")),
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
                .reply_to_message_id(m.id)
                .maybe_thread_id(&m)
                .await?;
            log::info!("Handled message: chat [{}]", m.chat.id);
        },
        Actions::RSticker(file_id) => {
            bot.send_sticker(m.chat.id, InputFile::file_id(file_id))
                .reply_to_message_id(m.reply_to_message().unwrap().id.0)
                .maybe_thread_id(&m)
                .await?;
            log::info!("Handled message with sticker: chat [{}]", m.chat.id);
        },
        Actions::RVoice(file_id) => {
            bot.send_voice(m.chat.id, InputFile::file_id(file_id))
                .reply_to_message_id(m.reply_to_message().unwrap().id)
                .maybe_thread_id(&m)
                .await?;
            log::info!("Handled message with voice: chat [{}]", m.chat.id);
        },
        Actions::Photo(file_id) => {
            bot.send_photo(m.chat.id, InputFile::file_id(file_id))
                .reply_to_message_id(m.id)
                .maybe_thread_id(&m)
                .await?;
            log::info!("Handled message with image: chat [{}]", m.chat.id);
        },
        Actions::None => {
            log::info!("Unhandled message: chat [{}]", m.chat.id)
        },
    };

    Ok(())
}
