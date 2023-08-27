use crate::db::DB;
use crate::enums::EpycCommands;
use crate::lang::{get_tag_opt, lng, tag, InnerLang, LocaleTag};
use crate::traits::MaybeMessageSetter;
use crate::{MyBot, MyResult};

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
    let ltag = tag(get_tag_opt(m.from()));

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
        log::error!("Error {:?} in command: !{}", err, cmd.to_string());
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
    let Some(from) = m.from() else { return Ok(())};

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
        "приветствие" | "привітання" | "greetings" => {
            _epyc_greetings_setting(bot, m, ltag, setting).boxed()
        },
        "топ" | "top" => _epyc_top_setting(bot, m, ltag, setting).boxed(),
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
    if setting.is_none() {
        let text = lng("OptionExistIncorrectParam", ltag);
        bot.send_message(m.chat.id, text).maybe_thread_id(&m).await?;
        return Ok(());
    }

    let key = match setting.unwrap() {
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
