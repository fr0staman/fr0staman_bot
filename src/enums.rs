#![allow(non_camel_case_types)]

use strum::{AsRefStr, Display, EnumString};
use teloxide::macros::BotCommands;

// Descriptions of BotCommands — check locales /<command>_desc
#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "lowercase")]
pub enum MyCommands {
    /// System
    Start,
    Help,
    Id,

    Pidor,

    Print(String),
    P(String),

    /// Game commands
    Grow,
    Name(String),
    My,
    Top,
    Game,
    Lang,
}

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "lowercase")]
pub enum EpycCommands {
    // Command center, check for multilang desc
    #[command(rename = "епік", prefix = "!")]
    EpycUA(String),
    #[command(rename = "эпик", prefix = "!")]
    EpycRU(String),
    #[command(rename = "epyc", prefix = "!")]
    EpycEN(String),
}

// Descriptions of BotCommands — check locales /<command>_desc
#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "lowercase")]
pub enum AdminCommands {
    Promote(String),
}

#[derive(Display, AsRefStr, EnumString)]
pub enum InlineCommands {
    #[strum(
        serialize = "ім'я",
        serialize = "імя",
        serialize = "имя",
        serialize = "name"
    )]
    Name,
    #[strum(serialize = "хрю", serialize = "hru", serialize = "grunt")]
    Hru,
    #[strum(serialize = "прапор", serialize = "флаг", serialize = "flag")]
    Flag,
    #[strum(serialize = "гіф", serialize = "гиф", serialize = "gif")]
    Gif,
}

#[derive(Display, AsRefStr, EnumString)]
pub enum InlineKeywords {
    #[strum(
        serialize = "ім'я",
        serialize = "імя",
        serialize = "имя",
        serialize = "name"
    )]
    Name,
    #[strum(
        serialize = "хряк",
        serialize = "свиня",
        serialize = "свинья",
        serialize = "pig"
    )]
    DayPig,
    #[strum(serialize = "ос", serialize = "oc")]
    OC,
    #[strum(serialize = "хрю", serialize = "hru", serialize = "grunt")]
    Hru,
    #[strum(serialize = "прапор", serialize = "флаг", serialize = "flag")]
    Flag,
    #[strum(serialize = "мова", serialize = "язык", serialize = "lang")]
    Lang,
    #[strum(serialize = "гіф", serialize = "гиф", serialize = "gif")]
    Gif,
}

#[derive(AsRefStr, EnumString, Display)]
#[strum(serialize_all = "snake_case")]
pub enum CbActions {
    GiveName,
    FindHryak,
    AddChat,
    Top10,
    StartDuel,
    TopLeft,
    TopRight,
    // Naming is not really good, but legacy is legacy
    AllowVoice,
    DisallowVoice,
    ChangeFlag,
    ChangeLang,
    SubCheck,
    SubGift,
    GifDecision,
}

#[derive(AsRefStr, EnumString, Display)]
#[strum(serialize_all = "snake_case")]
pub enum Top10Variant {
    Global,
    Chat,
    Win,
    PGlobal,
    PWin,
}

impl Top10Variant {
    pub fn summarize(self) -> Self {
        match self {
            Self::PGlobal => Self::Global,
            Self::PWin => Self::Win,
            _ => self,
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(AsRefStr)]
pub enum Image {
    #[strum(serialize = "1_fight_200x200.jpg")]
    Fight,
    #[strum(serialize = "2_top_200x200.jpg")]
    Top,
    #[strum(serialize = "3_take_weight_200x200.jpg")]
    TakeWeight,
    #[strum(serialize = "4_name_typing_200x200.jpg")]
    NameTyping,
    #[strum(serialize = "5_name_success_200x200.jpg")]
    NameSuccess,
    #[strum(serialize = "6_pigoftheday_200x200.jpg")]
    DayPig,
    #[strum(serialize = "8_1_cpu_200x200.jpg")]
    OCCPU,
    #[strum(serialize = "8_2_ram_200x200.jpg")]
    OCRAM,
    #[strum(serialize = "8_3_gpu_200x200.jpg")]
    OCGPU,
    #[strum(serialize = "9_error_200x200.jpg")]
    Error,
    #[strum(serialize = "10_more_info_200x200.jpg")]
    MoreInfo,
}

#[derive(AsRefStr, EnumString, Display)]
#[strum(serialize_all = "snake_case")]
pub enum PigGrowthStatus {
    Lost,
    Maintained,
    Gained,
}
