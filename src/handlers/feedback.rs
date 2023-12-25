use futures::FutureExt;
use teloxide::prelude::*;
use teloxide::types::ChosenInlineResult;

use crate::consts::INLINE_NAME_SET_LIMIT;
use crate::db::DB;
use crate::enums::InlineResults;
use crate::lang::{get_tag, lng, tag, tag_one_or, InnerLang, LocaleTag};
use crate::types::MyBot;
use crate::utils::flag::Flags;
use crate::utils::helpers;
use crate::{MyError, MyResult};

pub async fn filter_inline_feedback_commands(
    bot: MyBot,
    q: ChosenInlineResult,
) -> MyResult<()> {
    let Some(result) = InlineResults::from_str_with_args(&q.result_id) else {
        log::error!(
            "Undefined chosen inline: [{}] user: [{}]",
            q.result_id,
            q.from.id
        );
        return Ok(());
    };

    let user = DB.other.get_user(q.from.id.0).await?;
    let ltag =
        tag_one_or(user.and_then(|u| u.lang).as_deref(), get_tag(&q.from));

    let temp_bot = bot.clone();

    let function = match &result {
        InlineResults::RenameHryakInfo => {
            chosen_rename_hryak(bot, &q, ltag).boxed()
        },
        InlineResults::FlagChangeInfo(v) => {
            chosen_change_flag(bot, &q, ltag, v).boxed()
        },
        InlineResults::LangChangeInfo(v) => {
            chosen_change_lang(bot, &q, ltag, v).boxed()
        },
        _ => {
            chosen_unhandled(bot, &q).await?;
            return Ok(());
        },
    };

    let response = function.await;

    if let Err(err) = response {
        handle_error(temp_bot, &q, err).await;
    } else {
        handle_good(temp_bot, &q).await;
    }

    Ok(())
}

async fn chosen_rename_hryak(
    bot: MyBot,
    q: &ChosenInlineResult,
    ltag: LocaleTag,
) -> MyResult<()> {
    let Some(im_id) = &q.inline_message_id else { return Ok(()) };

    let name_from_query = q.query.split_once(' ').map_or("pig", |v| v.1);
    let (new_name, _) =
        helpers::truncate(name_from_query, INLINE_NAME_SET_LIMIT);

    DB.hand_pig.update_hrundel_name(q.from.id.0, new_name).await?;

    let text = lng("HandPigNameNowIs", ltag).args(&[("new_name", new_name)]);
    bot.edit_message_text_inline(im_id, text)
        .disable_web_page_preview(true)
        .await?;
    Ok(())
}

async fn chosen_change_flag(
    bot: MyBot,
    q: &ChosenInlineResult,
    ltag: LocaleTag,
    code: &str,
) -> MyResult<()> {
    let Some(im_id) = &q.inline_message_id else { return Ok(()) };

    let probably_flag = Flags::from_code(code).unwrap_or(Flags::Us);

    DB.hand_pig
        .update_hrundel_flag(q.from.id.0, probably_flag.to_code())
        .await?;

    let text = lng("HandPigFlagChangeResponse", ltag)
        .args(&[("flag", probably_flag.to_emoji())]);
    bot.edit_message_text_inline(im_id, &text).await?;

    Ok(())
}

async fn chosen_change_lang(
    bot: MyBot,
    q: &ChosenInlineResult,
    ltag: LocaleTag,
    code: &str,
) -> MyResult<()> {
    let Some(im_id) = &q.inline_message_id else { return Ok(()) };
    let (text, res) = if code == "-" {
        DB.other.change_user_lang(q.from.id.0, None).await?;

        let text = lng("InlineLangDeleteResponse", ltag);
        (text, None)
    } else {
        let probably_flag = Flags::from_code(code).unwrap_or(Flags::Us);

        let new_ltag = tag(code);
        let text = lng("InlineLangChangeResponse", new_ltag)
            .args(&[("code", code), ("flag", probably_flag.to_emoji())]);
        (text, Some(code))
    };

    DB.other.change_user_lang(q.from.id.0, res).await?;
    bot.edit_message_text_inline(im_id, &text).await?;

    Ok(())
}
async fn chosen_unhandled(_bot: MyBot, q: &ChosenInlineResult) -> MyResult<()> {
    log::info!(
        "Unhandled chosen inline: [{}] user: [{}]",
        q.result_id,
        q.from.id
    );
    Ok(())
}

async fn handle_error(_bot: MyBot, q: &ChosenInlineResult, err: MyError) {
    log::info!(
        "Error in inline feedback: [{}] {:?} user: [{}]",
        q.result_id,
        err,
        q.from.id
    )
}

async fn handle_good(_bot: MyBot, q: &ChosenInlineResult) {
    log::info!(
        "Handled inline feedback: [{}] user: [{}]",
        q.result_id,
        q.from.id
    );
}
