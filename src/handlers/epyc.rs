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

    let user_info = DB.other.get_user(from.id.0).await?;
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
        crate::myerr!("Error {:?} in command: !{}", err, cmd.to_string());
    } else {
        log::info!("Handled command !{}", cmd.to_string());
    }

    Ok(())
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

    let mut splitted = arg.split_whitespace();
    let Some(option) = splitted.next() else { return Ok(()) };
    let setting = splitted.next();

    let function = match option {
        "привітання" | "приветствие" | "greetings" => {
            _epyc_greetings_setting(bot, m, ltag, setting).boxed()
        },
        "топ" | "top" => _epyc_top_setting(bot, m, ltag, setting).boxed(),
        "мова" | "язык" | "lang" => {
            _epyc_chat_lang_setting(bot, m, ltag, setting).boxed()
        },
        _ => _epyc_function_not_exist(bot, m, ltag).boxed(),
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

    let key = match setting {
        "-" => (1, "GreetingsDisabled"),
        "+" => (0, "GreetingsEnabled"),
        _ => {
            _epyc_invalid_arg(bot, m, ltag, "greeting").await?;
            return Ok(());
        },
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
