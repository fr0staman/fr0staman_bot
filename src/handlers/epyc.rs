use crate::db::DB;
use crate::db::models::UpdateGroups;
use crate::enums::EpycCommands;
use crate::lang::{
    InnerLang, LocaleTag, get_langs, get_tag_opt, lng, tag, tag_one_two_or,
};
use crate::traits::MaybeMessageSetter;
use crate::types::{MyBot, MyResult};

use futures::FutureExt;
use teloxide::prelude::*;
use teloxide::types::ChatKind;

impl std::fmt::Display for EpycCommands {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            EpycCommands::EpycUA(args) => write!(f, "епік {}", args),
            EpycCommands::EpycRU(args) => write!(f, "эпик {}", args),
            EpycCommands::EpycEN(args) => write!(f, "epyc {}", args),
        }
    }
}

pub async fn filter_commands(
    bot: MyBot,
    m: Message,
    cmd: EpycCommands,
) -> MyResult<()> {
    let Some(from) = &m.from else { return Ok(()) };

    let user_info = DB.other.get_user(from.id.0 as i64).await?;
    let chat_info = DB.other.get_chat(m.chat.id.0).await?;

    let ltag = tag_one_two_or(
        user_info.and_then(|c| c.lang).as_deref(),
        chat_info.and_then(|c| c.lang).as_deref(),
        get_tag_opt(m.from.as_ref()),
    );

    if let ChatKind::Private(_) = m.chat.kind {
        let text = lng("EPYCCenterOnlyForChats", ltag);
        bot.send_message(m.chat.id, text).maybe_thread_id(&m).await?;
        return Ok(());
    }

    let function = match &cmd {
        EpycCommands::EpycUA(arg)
        | EpycCommands::EpycRU(arg)
        | EpycCommands::EpycEN(arg) => command_epyc(bot, m, ltag, arg).boxed(),
    };

    let response = function.await;

    if let Err(err) = response {
        crate::myerr!("Error {:?} in command: !{cmd}", err);
    } else {
        log::info!("Handled command !{cmd}");
    }

    Ok(())
}

#[derive(Clone, Copy)]
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub enum EpycSetting {
    Greetings,
    Top,
    Lang,
}

/// `None` for an unrecognised sub-command — the caller answers "function not
/// found".
pub fn parse_epyc_arg(arg: &str) -> (Option<EpycSetting>, Option<&str>) {
    let mut splitted = arg.split_whitespace();
    let Some(option) = splitted.next() else { return (None, None) };
    let setting = splitted.next();

    let parsed = match option {
        "привітання" | "приветствие" | "greetings" => {
            Some(EpycSetting::Greetings)
        },
        "топ" | "top" => Some(EpycSetting::Top),
        "мова" | "язык" | "lang" => Some(EpycSetting::Lang),
        _ => None,
    };

    (parsed, setting)
}

/// The `groups.settings` value and the locale key confirming it.
pub fn parse_greetings_setting(setting: &str) -> Option<(i16, &'static str)> {
    match setting {
        "-" => Some((1, "GreetingsDisabled")),
        "+" => Some((0, "GreetingsEnabled")),
        _ => None,
    }
}

// Command center
async fn command_epyc(
    bot: MyBot,
    m: Message,
    ltag: LocaleTag,
    arg: &str,
) -> MyResult<()> {
    let Some(from) = &m.from else { return Ok(()) };

    let member = bot.get_chat_member(m.chat.id, from.id).await?;

    if !member.can_restrict_members() {
        let text = lng("YoureNotAdmin", ltag);
        bot.send_message(m.chat.id, text).maybe_thread_id(&m).await?;
        return Ok(());
    }

    if arg.is_empty() {
        let text = lng("EPYC", ltag);
        bot.send_message(m.chat.id, text).maybe_thread_id(&m).await?;
        return Ok(());
    }

    // Whitespace-only argument: stay silent, as before.
    if arg.split_whitespace().next().is_none() {
        return Ok(());
    }

    let (option, setting) = parse_epyc_arg(arg);

    let function = match option {
        Some(EpycSetting::Greetings) => {
            _epyc_greetings_setting(bot, m, ltag, setting).boxed()
        },
        Some(EpycSetting::Top) => {
            _epyc_top_setting(bot, m, ltag, setting).boxed()
        },
        Some(EpycSetting::Lang) => {
            _epyc_chat_lang_setting(bot, m, ltag, setting).boxed()
        },
        None => _epyc_function_not_exist(bot, m, ltag).boxed(),
    };

    function.await?;

    Ok(())
}

async fn _epyc_function_not_exist(
    bot: MyBot,
    m: Message,
    ltag: LocaleTag,
) -> MyResult<()> {
    let text = lng("FunctionNotExist", ltag);
    bot.send_message(m.chat.id, text).maybe_thread_id(&m).await?;

    Ok(())
}

async fn _epyc_invalid_arg(
    bot: MyBot,
    m: Message,
    ltag: LocaleTag,
    #[allow(unused)] example: &str,
) -> MyResult<()> {
    let text = lng("OptionExistIncorrectParam", ltag);
    bot.send_message(m.chat.id, text).maybe_thread_id(&m).await?;

    Ok(())
}

async fn _epyc_top_setting(
    bot: MyBot,
    m: Message,
    ltag: LocaleTag,
    setting: Option<&str>,
) -> MyResult<()> {
    if setting.is_none() {
        let text = lng("OptionExistIncorrectParam", ltag);
        bot.send_message(m.chat.id, text).maybe_thread_id(&m).await?;
        return Ok(());
    }

    let key = match setting.unwrap().parse::<i32>() {
        Ok(value) => (value, "ChangedTop10Visiblity"),
        Err(_) => {
            _epyc_invalid_arg(bot, m, ltag, "top10").await?;
            return Ok(());
        },
    };

    DB.other.set_top10_setting(m.chat.id.0, key.0).await?;
    let text = lng(key.1, ltag).args(&[("setting", &key.0.to_string())]);
    bot.send_message(m.chat.id, text).maybe_thread_id(&m).await?;

    Ok(())
}

async fn _epyc_greetings_setting(
    bot: MyBot,
    m: Message,
    ltag: LocaleTag,
    setting: Option<&str>,
) -> MyResult<()> {
    let Some(setting) = setting else {
        let text = lng("OptionExistIncorrectParam", ltag);
        bot.send_message(m.chat.id, text).maybe_thread_id(&m).await?;
        return Ok(());
    };

    let Some(key) = parse_greetings_setting(setting) else {
        _epyc_invalid_arg(bot, m, ltag, "greeting").await?;
        return Ok(());
    };

    DB.other.set_chat_settings(m.chat.id.0, key.0).await?;
    bot.send_message(m.chat.id, lng(key.1, ltag)).maybe_thread_id(&m).await?;

    Ok(())
}

async fn _epyc_chat_lang_setting(
    bot: MyBot,
    m: Message,
    mut ltag: LocaleTag,
    setting: Option<&str>,
) -> MyResult<()> {
    let Some(setting) = setting else {
        let text = lng("OptionExistIncorrectParam", ltag);
        bot.send_message(m.chat.id, text).maybe_thread_id(&m).await?;
        return Ok(());
    };

    let Some(chat_info) = DB.other.get_chat(m.chat.id.0).await? else {
        return Ok(());
    };

    let langs = get_langs();

    let value = if langs.iter().any(|lang| lang == setting) {
        ltag = tag(setting);
        ("EPYCCommandLangSetSuccessMessage", Some(setting.to_string()))
    } else if setting == "-" {
        ("EPYCCommandLangDeleteSuccessMessage", None)
    } else {
        _epyc_invalid_arg(bot, m, ltag, "top10").await?;
        return Ok(());
    };

    let update_chat_info =
        UpdateGroups { lang: value.1, ..chat_info.to_update() };

    DB.other.update_chat(m.chat.id.0, update_chat_info).await?;
    let text = lng(value.0, ltag).args(&[("chat_lang", setting)]);
    bot.send_message(m.chat.id, text).maybe_thread_id(&m).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_sub_command_is_recognised_in_every_language() {
        let cases = [
            ("привітання", EpycSetting::Greetings),
            ("приветствие", EpycSetting::Greetings),
            ("greetings", EpycSetting::Greetings),
            ("топ", EpycSetting::Top),
            ("top", EpycSetting::Top),
            ("мова", EpycSetting::Lang),
            ("язык", EpycSetting::Lang),
            ("lang", EpycSetting::Lang),
        ];

        for (word, expected) in cases {
            assert_eq!(parse_epyc_arg(word).0, Some(expected), "{word}");
        }
    }

    #[test]
    fn the_parameter_is_the_second_token() {
        assert_eq!(
            parse_epyc_arg("greetings +"),
            (Some(EpycSetting::Greetings), Some("+"))
        );
        assert_eq!(parse_epyc_arg("top 10"), (Some(EpycSetting::Top), Some("10")));
        assert_eq!(parse_epyc_arg("lang uk"), (Some(EpycSetting::Lang), Some("uk")));
    }

    #[test]
    fn extra_whitespace_is_ignored() {
        assert_eq!(
            parse_epyc_arg("   top    10   "),
            (Some(EpycSetting::Top), Some("10"))
        );
    }

    #[test]
    fn trailing_tokens_beyond_the_parameter_are_dropped() {
        assert_eq!(
            parse_epyc_arg("top 10 20 30"),
            (Some(EpycSetting::Top), Some("10"))
        );
    }

    #[test]
    fn a_missing_parameter_is_reported_as_none() {
        assert_eq!(parse_epyc_arg("top"), (Some(EpycSetting::Top), None));
    }

    #[test]
    fn an_unknown_sub_command_is_rejected() {
        for word in ["nope", "TOP", "Топ", "прив", ""] {
            assert_eq!(parse_epyc_arg(word).0, None, "{word:?}");
        }
    }

    #[test]
    fn greetings_maps_plus_and_minus_to_the_settings_column() {
        // `groups.settings`: 0 = greetings on, 1 = off.
        assert_eq!(parse_greetings_setting("+"), Some((0, "GreetingsEnabled")));
        assert_eq!(parse_greetings_setting("-"), Some((1, "GreetingsDisabled")));
    }

    #[test]
    fn an_invalid_greetings_parameter_is_rejected() {
        for bad in ["", "on", "off", "1", "0", "++"] {
            assert_eq!(parse_greetings_setting(bad), None, "{bad:?}");
        }
    }
}
