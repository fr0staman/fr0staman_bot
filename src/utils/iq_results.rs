use teloxide::types::{
    ChatType, InlineQueryResultArticle, InlineQueryResultCachedGif,
    InlineQueryResultVoice, InputMessageContent, InputMessageContentText,
    LinkPreviewOptions, UserId,
};
use teloxide::utils::html::{bold, italic};
use url::Url;

use crate::config::consts::INLINE_NAME_SET_LIMIT;
use crate::config::env::{BOT_CONFIG, bot_me};
use crate::db::models::{InlineUser, User};
use crate::enums::{CbActions, Image, InlineResults, Top10Variant};
use crate::keyboards;
use crate::lang::{InnerLang, LocaleTag, lng};
use crate::traits::SimpleDisableWebPagePreview;
use crate::utils::flag::Flags;
use crate::utils::formulas;
use crate::utils::helpers::{
    self, get_photostock, truncate,
};

pub fn get_start_duel(
    ltag: LocaleTag,
    id_user: UserId,
    info: &InlineUser,
) -> InlineQueryResultArticle {
    let winrate = _get_duel_winrate(ltag, info.win, info.rout);

    let title = lng("DuelInlineCaption", ltag);
    let text = lng("InlineDuelStartMessage", ltag).args(&[
        ("name", &info.name),
        ("winrate", &winrate),
        ("weight", &info.weight.to_string()),
    ]);

    let content = InputMessageContentText::new(text)
        .link_preview_options(LinkPreviewOptions::disable(true));

    let desc = lng("DuelInlineDesc", ltag)
        .args(&[("chat_name", &BOT_CONFIG.chat_link)]);

    InlineQueryResultArticle::new(
        InlineResults::GetStartDuel.to_string_with_args(),
        title,
        InputMessageContent::Text(content),
    )
    .description(desc)
    .thumbnail_url(get_photostock(Image::Fight))
    .reply_markup(keyboards::keyboard_start_duel(ltag, id_user))
}

pub fn get_top10_info(
    ltag: LocaleTag,
    id_user: UserId,
    text: String,
    chat_type: Top10Variant,
) -> InlineQueryResultArticle {
    let title = lng("Top10Caption", ltag);
    let message_text = InputMessageContentText::new(text)
        .link_preview_options(LinkPreviewOptions::disable(true));

    InlineQueryResultArticle::new(
        InlineResults::GetTop10Info.to_string_with_args(),
        title,
        InputMessageContent::Text(message_text),
    )
    .description(lng("InlineTop10Desc", ltag))
    .thumbnail_url(get_photostock(Image::Top))
    .reply_markup(keyboards::keyboard_in_top10(ltag, id_user, chat_type))
}

pub fn get_hryak_info(
    ltag: LocaleTag,
    id_user: UserId,
    info: &(InlineUser, User),
    remove_markup: bool,
) -> InlineQueryResultArticle {
    let append = if info.1.supported {
        lng("InlineSupportedDeveloping", ltag)
    } else {
        String::new()
    };

    let converted_mass = info.0.weight.to_string();

    let caption = lng("InlineStatsCaption", ltag);
    let message = lng("HandPigWeightMessage", ltag).args(&[
        ("weight", converted_mass.as_str()),
        ("emoji", formulas::get_pig_emoji(info.0.weight)),
        ("append", append.as_str()),
    ]);

    let desc =
        lng("InlineStatsDesc", ltag).args(&[("weight", &converted_mass)]);
    let query_result = InlineQueryResultArticle::new(
        InlineResults::GetHryakInfo.to_string_with_args(),
        caption,
        InputMessageContent::Text(
            InputMessageContentText::new(message)
                .link_preview_options(LinkPreviewOptions::disable(true)),
        ),
    )
    .description(desc)
    .thumbnail_url(get_photostock(Image::TakeWeight));

    if remove_markup {
        query_result
    } else {
        query_result
            .reply_markup(keyboards::keyboard_add_inline_top10(ltag, id_user))
    }
}

pub fn get_more_info(ltag: LocaleTag) -> InlineQueryResultArticle {
    let caption = lng("InlineMoreInfoCaption", ltag);
    let message = lng("InlineMoreInfoMessage", ltag);

    let desc = lng("InlineMoreInfoDesc", ltag);

    InlineQueryResultArticle::new(
        InlineResults::GetMoreInfo.to_string_with_args(),
        caption,
        InputMessageContent::Text(
            InputMessageContentText::new(message)
                .link_preview_options(LinkPreviewOptions::disable(true)),
        ),
    )
    .description(desc)
    .thumbnail_url(get_photostock(Image::MoreInfo))
    .reply_markup(keyboards::keyboard_more_info(ltag))
}

pub fn name_hryak_info(
    ltag: LocaleTag,
    name: String,
) -> InlineQueryResultArticle {
    let caption = lng("HandPigNameGoCaption", ltag);
    let message = lng("HandPigNameGoMessage", ltag)
        .args(&[("name", &*name), ("bot_name", bot_me().username())]);

    InlineQueryResultArticle::new(
        InlineResults::NameHryakInfo.to_string_with_args(),
        caption,
        InputMessageContent::Text(
            InputMessageContentText::new(message)
                .link_preview_options(LinkPreviewOptions::disable(true)),
        ),
    )
    .description(lng("HandPigNameGoDesc", ltag))
    .thumbnail_url(get_photostock(Image::NameTyping))
}

pub fn rename_hryak_info(
    ltag: LocaleTag,
    id_user: UserId,
    old_name: String,
    new_name: &str,
) -> InlineQueryResultArticle {
    // Display width and callback-payload bytes are separate budgets; both
    // apply here so the confirmed name matches the stored one.
    let (cutted_name, _) = truncate(new_name, INLINE_NAME_SET_LIMIT);
    let room = helpers::callback_payload_room(CbActions::GiveName, id_user);
    let cutted_name = helpers::truncate_bytes(cutted_name, room).to_string();

    let message = lng("HandPigNameChangeMessage", ltag)
        .args(&[("past_name", &old_name), ("future_name", &cutted_name)]);

    let desc = lng("HandPigNameChangeDesc", ltag);
    InlineQueryResultArticle::new(
        InlineResults::RenameHryakInfo.to_string_with_args(),
        &cutted_name,
        InputMessageContent::Text(
            InputMessageContentText::new(message)
                .link_preview_options(LinkPreviewOptions::disable(true)),
        ),
    )
    .description(desc)
    .reply_markup(keyboards::keyboard_new_name(ltag, id_user, cutted_name))
    .thumbnail_url(get_photostock(Image::NameSuccess))
}

pub fn day_pig_info(
    ltag: LocaleTag,
    id_user: UserId,
    chat_type: Option<ChatType>,
) -> InlineQueryResultArticle {
    let caption = lng("InlineDayPigCaption", ltag);
    let message = lng("InlineDayPigMessage", ltag);
    let desc = lng("InlineDayPigDesc", ltag);

    let is_public_chat = chat_type
        .is_some_and(|v| !matches!(v, ChatType::Private | ChatType::Channel));

    let markup = if is_public_chat {
        keyboards::keyboard_day_pig(ltag, id_user)
    } else {
        keyboards::keyboard_day_pig_to_inline(ltag)
    };
    InlineQueryResultArticle::new(
        InlineResults::DayPigInfo.to_string_with_args(),
        caption,
        InputMessageContent::Text(
            InputMessageContentText::new(message)
                .link_preview_options(LinkPreviewOptions::disable(true)),
        ),
    )
    .description(desc)
    .thumbnail_url(get_photostock(Image::DayPig))
    .reply_markup(markup)
}

pub fn flag_info(ltag: LocaleTag, flag: &str) -> InlineQueryResultArticle {
    let caption = lng("HandPigFlagGoCaption", ltag).args(&[("flag", flag)]);
    let desc = lng("HandPigFlagGoDesc", ltag);
    let message = lng("HandPigFlagGoMessage", ltag)
        .args(&[("flag", flag), ("bot_name", bot_me().username())]);

    InlineQueryResultArticle::new(
        InlineResults::FlagInfo.to_string_with_args(),
        caption,
        InputMessageContent::Text(
            InputMessageContentText::new(message)
                .link_preview_options(LinkPreviewOptions::disable(true)),
        ),
    )
    .description(desc)
}

pub fn flag_empty_info(ltag: LocaleTag) -> InlineQueryResultArticle {
    let caption = lng("HandPigNoFlagChangeCaption", ltag);
    let desc = lng("HandPigNoFlagChangeDesc", ltag);
    let message = lng("HandPigNoFlagChangeMessage", ltag);

    InlineQueryResultArticle::new(
        InlineResults::FlagEmptyInfo.to_string_with_args(),
        caption,
        InputMessageContent::Text(
            InputMessageContentText::new(message)
                .link_preview_options(LinkPreviewOptions::disable(true)),
        ),
    )
    .description(desc)
}

pub fn flag_change_info(
    ltag: LocaleTag,
    id_user: UserId,
    old_flag: Flags,
    new_flag: Flags,
) -> InlineQueryResultArticle {
    let old_flag_emoji = old_flag.to_emoji();
    let new_flag_emoji = new_flag.to_emoji();
    let new_flag_code = new_flag.to_code();

    let caption =
        lng("HandPigFlagChangeCaption", ltag).args(&[("flag", new_flag_emoji)]);
    let desc =
        lng("HandPigFlagChangeDesc", ltag).args(&[("code", new_flag_code)]);

    let message = lng("HandPigFlagChangeMessage", ltag)
        .args(&[("old_flag", old_flag_emoji), ("new_flag", new_flag_emoji)]);

    let markup = keyboards::keyboard_change_flag(ltag, id_user, new_flag_code);

    InlineQueryResultArticle::new(
        InlineResults::FlagChangeInfo(new_flag_code.to_string())
            .to_string_with_args(),
        caption,
        InputMessageContent::Text(
            InputMessageContentText::new(message)
                .link_preview_options(LinkPreviewOptions::disable(true)),
        ),
    )
    .description(desc)
    .reply_markup(markup)
}

pub fn lang_info(
    ltag: LocaleTag,
    id_user: UserId,
    flag: &str,
    code: &str,
) -> InlineQueryResultArticle {
    let caption = lng("InlineLangGoCaption", ltag)
        .args(&[("flag", flag), ("code", code)]);
    let desc = lng("InlineLangGoDesc", ltag);
    let message = lng("InlineLangGoMessage", ltag).args(&[
        ("flag", flag),
        ("code", code),
        ("bot_name", bot_me().username()),
    ]);

    let markup = keyboards::keyboard_change_lang(ltag, id_user, "-");
    InlineQueryResultArticle::new(
        InlineResults::LangInfo.to_string_with_args(),
        caption,
        InputMessageContent::Text(
            InputMessageContentText::new(message)
                .link_preview_options(LinkPreviewOptions::disable(true)),
        ),
    )
    .description(desc)
    .reply_markup(markup)
}

pub fn lang_empty_info(ltag: LocaleTag) -> InlineQueryResultArticle {
    let caption = lng("InlineLangNoChangeCaption", ltag);
    let desc = lng("InlineLangNoChangeDesc", ltag);
    let message = lng("InlineLangNoChangeMessage", ltag);

    InlineQueryResultArticle::new(
        InlineResults::LangEmptyInfo.to_string_with_args(),
        caption,
        InputMessageContent::Text(
            InputMessageContentText::new(message)
                .link_preview_options(LinkPreviewOptions::disable(true)),
        ),
    )
    .description(desc)
}

pub fn lang_change_info(
    ltag: LocaleTag,
    id_user: UserId,
    old_lang_code: Option<&str>,
    new_lang_code: &str,
) -> InlineQueryResultArticle {
    let old_lang_emoji = old_lang_code
        .map_or("-", |v| Flags::from_code(v).unwrap_or(Flags::Us).to_emoji());
    let new_lang_emoji =
        Flags::from_code(new_lang_code).unwrap_or(Flags::Us).to_emoji();

    let langed_key = format!("lang_{new_lang_code}");

    let langed_name = lng(&langed_key, ltag);
    let caption = lng("InlineLangChangeCaption", ltag)
        .args(&[("flag", new_lang_emoji), ("lang", &langed_name)]);
    let desc_key = if old_lang_emoji == new_lang_emoji {
        "InlineLangChangeDescAlready"
    } else {
        "InlineLangChangeDesc"
    };
    let desc = lng(desc_key, ltag).args(&[("code", new_lang_code)]);

    let message = lng("InlineLangChangeMessage", ltag).args(&[
        ("old_code", old_lang_code.unwrap_or("-")),
        ("old_flag", old_lang_emoji),
        ("new_code", new_lang_code),
        ("new_flag", new_lang_emoji),
    ]);

    let markup = keyboards::keyboard_change_lang(ltag, id_user, new_lang_code);

    InlineQueryResultArticle::new(
        InlineResults::LangChangeInfo(new_lang_code.to_owned())
            .to_string_with_args(),
        caption,
        InputMessageContent::Text(
            InputMessageContentText::new(message)
                .link_preview_options(LinkPreviewOptions::disable(true)),
        ),
    )
    .description(desc)
    .reply_markup(markup)
}

pub fn cpu_oc_info(ltag: LocaleTag, mass: f32) -> InlineQueryResultArticle {
    let caption = lng("InlineOcCPUCaption", ltag);

    let message = lng("InlineOcCPUMessage", ltag).args(&[
        ("cpu_clock", mass.to_string().as_str()),
        ("cpu_emoji", formulas::get_oc_cpu_emoji(mass)),
    ]);

    InlineQueryResultArticle::new(
        InlineResults::CpuOcInfo.to_string_with_args(),
        caption,
        InputMessageContent::Text(
            InputMessageContentText::new(message)
                .link_preview_options(LinkPreviewOptions::disable(true)),
        ),
    )
    .thumbnail_url(get_photostock(Image::OCCPU))
}

pub fn ram_oc_info(ltag: LocaleTag, mass: u32) -> InlineQueryResultArticle {
    let caption = lng("InlineOcRAMCaption", ltag);
    let message = lng("InlineOcRAMMessage", ltag).args(&[
        ("ram_clock", mass.to_string().as_str()),
        ("ram_emoji", formulas::get_oc_ram_emoji(mass)),
    ]);

    InlineQueryResultArticle::new(
        InlineResults::RamOcInfo.to_string_with_args(),
        caption,
        InputMessageContent::Text(
            InputMessageContentText::new(message)
                .link_preview_options(LinkPreviewOptions::disable(true)),
        ),
    )
    .thumbnail_url(get_photostock(Image::OCRAM))
}

pub fn gpu_oc_info(ltag: LocaleTag, mass: f32) -> InlineQueryResultArticle {
    let caption = lng("InlineOcGPUCaption", ltag);
    let message = lng("InlineOcGPUMessage", ltag).args(&[
        ("gpu_hashrate", mass.to_string().as_str()),
        ("gpu_emoji", formulas::get_oc_gpu_emoji(mass)),
    ]);

    InlineQueryResultArticle::new(
        InlineResults::GpuOcInfo.to_string_with_args(),
        caption,
        InputMessageContent::Text(
            InputMessageContentText::new(message)
                .link_preview_options(LinkPreviewOptions::disable(true)),
        ),
    )
    .thumbnail_url(get_photostock(Image::OCGPU))
}

pub fn hru_voice_info(
    id: i16,
    voice_url: Url,
    caption: String,
) -> InlineQueryResultVoice {
    InlineQueryResultVoice::new(
        InlineResults::HruVoice(id).to_string_with_args(),
        voice_url,
        caption,
    )
}

pub fn gif_pig_info(id: i16, file_id: String) -> InlineQueryResultCachedGif {
    InlineQueryResultCachedGif::new(
        InlineResults::PigGif(id).to_string_with_args(),
        teloxide::types::FileId(file_id),
    )
}

pub fn handle_error_info(ltag: LocaleTag) -> InlineQueryResultArticle {
    let caption = lng("Error", ltag);
    let message = lng("InlineTechDesc", ltag);
    let desc = lng("InlineTechCaption", ltag);

    InlineQueryResultArticle::new(
        InlineResults::ErrorInfo.to_string_with_args(),
        caption,
        InputMessageContent::Text(
            InputMessageContentText::new(message)
                .link_preview_options(LinkPreviewOptions::disable(true)),
        ),
    )
    .description(desc)
    .thumbnail_url(get_photostock(Image::Error))
}

pub fn handle_error_parse(ltag: LocaleTag) -> InlineQueryResultArticle {
    let caption = lng("ErrorParseInlineNumberCaption", ltag);
    let desc = lng("ErrorParseInlineNumberDesc", ltag);

    let message = format!("{}\n\n{}", &caption, &desc);

    InlineQueryResultArticle::new(
        InlineResults::ErrorParse.to_string_with_args(),
        caption,
        InputMessageContent::Text(
            InputMessageContentText::new(message)
                .link_preview_options(LinkPreviewOptions::disable(true)),
        ),
    )
    .description(desc)
}

pub fn handle_no_results(ltag: LocaleTag) -> InlineQueryResultArticle {
    let caption = lng("ErrorNoResultsCaption", ltag);
    let desc = lng("ErrorNoResultsDesc", ltag);

    let message = format!("{}\n\n{}", &caption, &desc);

    InlineQueryResultArticle::new(
        InlineResults::NoResults.to_string_with_args(),
        caption,
        InputMessageContent::Text(
            InputMessageContentText::new(message)
                .link_preview_options(LinkPreviewOptions::disable(true)),
        ),
    )
    .description(desc)
}

/// `None` only for a pig that has never fought.
pub fn winrate_percent(win: i32, rout: i32) -> Option<i32> {
    let total = win.saturating_add(rout);

    if total <= 0 || win < 0 {
        return None;
    }

    Some(((win as f32 / total as f32) * 100.0) as i32)
}

fn _get_duel_winrate(ltag: LocaleTag, win: i32, rout: i32) -> String {
    let Some(percent_winrate) = winrate_percent(win, rout) else {
        return italic(&lng("InlineDuelNotEnoughBattles", ltag));
    };

    bold(&format!("{} %", percent_winrate))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn winrate_is_the_share_of_wins() {
        assert_eq!(winrate_percent(1, 1), Some(50));
        assert_eq!(winrate_percent(3, 1), Some(75));
        assert_eq!(winrate_percent(1, 3), Some(25));
        assert_eq!(winrate_percent(2, 1), Some(66));
    }

    #[test]
    fn an_unbeaten_record_reports_one_hundred_percent() {
        // The guard used to be `rout == 0 || win == 0`, so a perfect record
        // showed "not enough battles" instead of 100 %.
        assert_eq!(winrate_percent(10, 0), Some(100));
        assert_eq!(winrate_percent(1, 0), Some(100));
    }

    #[test]
    fn a_winless_record_reports_zero_percent() {
        assert_eq!(winrate_percent(0, 10), Some(0));
        assert_eq!(winrate_percent(0, 1), Some(0));
    }

    #[test]
    fn only_a_pig_that_has_never_fought_reports_nothing() {
        assert_eq!(winrate_percent(0, 0), None);
    }

    #[test]
    fn the_percentage_never_leaves_zero_to_one_hundred() {
        for win in 0..80i32 {
            for rout in 0..80i32 {
                if win == 0 && rout == 0 {
                    continue;
                }
                let pct = winrate_percent(win, rout).unwrap();
                assert!((0..=100).contains(&pct), "{win}-{rout} -> {pct}");
            }
        }
    }

    #[test]
    fn the_percentage_is_truncated_not_rounded() {
        // 2/3 = 66.66… -> 66
        assert_eq!(winrate_percent(2, 1), Some(66));
        // 1/3 = 33.33… -> 33
        assert_eq!(winrate_percent(1, 2), Some(33));
    }


}
