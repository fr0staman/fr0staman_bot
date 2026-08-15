//! A typo in a Cyrillic or Azerbaijani alias is invisible in review and
//! silently breaks that language's whole command.

use std::str::FromStr;

use crate::enums::{
    AdminCommands, EpycCommands, InlineCommands, InlineKeywords, MyCommands,
};
use teloxide::utils::command::BotCommands;

const BOT: &str = "fr0staman_bot";


#[test]
fn every_slash_command_parses() {
    let commands = [
        "/start",
        "/help",
        "/id",
        "/pidor",
        "/grow",
        "/my",
        "/top",
        "/daypig",
        "/daypigs",
        "/game",
        "/lang",
        "/louder",
        "/achievements",
        "/resetpigs",
    ];

    for command in commands {
        MyCommands::parse(command, BOT)
            .unwrap_or_else(|e| panic!("{command} did not parse: {e:?}"));
    }
}

#[test]
fn the_at_bot_suffix_is_accepted() {
    // Group members type `/grow@fr0staman_bot` when several bots are present.
    for command in ["/grow", "/grow@fr0staman_bot"] {
        assert!(
            matches!(MyCommands::parse(command, BOT), Ok(MyCommands::Grow)),
            "{command}"
        );
    }
}

#[test]
fn a_command_addressed_to_another_bot_is_rejected() {
    assert!(MyCommands::parse("/grow@some_other_bot", BOT).is_err());
}

#[test]
fn print_and_its_alias_carry_the_same_argument() {
    let long = MyCommands::parse("/print hello world", BOT).unwrap();
    let short = MyCommands::parse("/p hello world", BOT).unwrap();

    match (long, short) {
        (MyCommands::Print(a), MyCommands::P(b)) => {
            assert_eq!(a, "hello world");
            assert_eq!(a, b);
        },
        other => panic!("unexpected parse: {other:?}"),
    }
}

#[test]
fn argument_taking_commands_accept_an_empty_argument() {
    for command in ["/name", "/print", "/p"] {
        MyCommands::parse(command, BOT)
            .unwrap_or_else(|e| panic!("{command}: {e:?}"));
    }
}

#[test]
fn name_keeps_its_whole_argument() {
    match MyCommands::parse("/name Сер Хрюндель III", BOT).unwrap() {
        MyCommands::Name(arg) => assert_eq!(arg, "Сер Хрюндель III"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn unknown_commands_and_plain_text_are_rejected() {
    for text in ["/nope", "grow", "", "hello", "//grow"] {
        assert!(MyCommands::parse(text, BOT).is_err(), "{text:?}");
    }
}

#[test]
fn the_ignored_commands_list_only_names_real_commands() {
    use crate::config::consts::IGNORED_COMMANDS;

    let known: Vec<String> = MyCommands::bot_commands()
        .iter()
        .map(|c| c.command.clone())
        .collect();

    for ignored in IGNORED_COMMANDS {
        assert!(
            known.iter().any(|c| c == ignored),
            "{ignored} is not a real command"
        );
    }
}

// !epyc

#[test]
fn epyc_parses_in_all_three_languages() {
    let cases = [("!епік", "епік"), ("!эпик", "эпик"), ("!epyc", "epyc")];

    for (prefix, _) in cases {
        let parsed = EpycCommands::parse(&format!("{prefix} top 10"), BOT)
            .unwrap_or_else(|e| panic!("{prefix}: {e:?}"));

        let arg = match parsed {
            EpycCommands::EpycUA(a)
            | EpycCommands::EpycRU(a)
            | EpycCommands::EpycEN(a) => a,
        };
        assert_eq!(arg, "top 10", "{prefix}");
    }
}

#[test]
fn epyc_requires_the_bang_prefix() {
    assert!(EpycCommands::parse("/epyc top 10", BOT).is_err());
    assert!(EpycCommands::parse("epyc top 10", BOT).is_err());
}

#[test]
fn epyc_accepts_no_argument() {
    assert!(EpycCommands::parse("!epyc", BOT).is_ok());
}


#[test]
fn admin_commands_parse_with_their_arguments() {
    match AdminCommands::parse("/promote 42", BOT).unwrap() {
        AdminCommands::Promote(arg) => assert_eq!(arg, "42"),
        other => panic!("{other:?}"),
    }

    match AdminCommands::parse("/repost +dm +chats", BOT).unwrap() {
        AdminCommands::Repost(arg) => assert_eq!(arg, "+dm +chats"),
        other => panic!("{other:?}"),
    }
}


#[test]
fn every_inline_command_alias_parses() {
    // (alias, the variant it must resolve to, identified by a sibling alias)
    let groups: [&[&str]; 4] = [
        &["ім'я", "імя", "имя", "name", "ad"],
        &["хрю", "hru", "grunt", "xort"],
        &["прапор", "флаг", "flag", "bayraq"],
        &["гіф", "гиф", "gif"],
    ];

    for group in groups {
        let first = InlineCommands::from_str(group[0])
            .unwrap_or_else(|_| panic!("{} did not parse", group[0]));

        for alias in group {
            let parsed = InlineCommands::from_str(alias)
                .unwrap_or_else(|_| panic!("{alias} did not parse"));
            assert_eq!(
                std::mem::discriminant(&parsed),
                std::mem::discriminant(&first),
                "{alias} resolved to a different variant than {}",
                group[0]
            );
        }
    }
}

#[test]
fn every_inline_keyword_alias_parses() {
    let groups: [&[&str]; 7] = [
        &["ім'я", "імя", "имя", "name", "ad"],
        &["хряк", "свиня", "свинья", "pig", "donuz"],
        &["ос", "oc"],
        &["хрю", "hru", "grunt", "xort"],
        &["прапор", "флаг", "flag", "bayraq"],
        &["мова", "язык", "lang", "dil"],
        &["гіф", "гиф", "gif"],
    ];

    for group in groups {
        let first = InlineKeywords::from_str(group[0])
            .unwrap_or_else(|_| panic!("{} did not parse", group[0]));

        for alias in group {
            let parsed = InlineKeywords::from_str(alias)
                .unwrap_or_else(|_| panic!("{alias} did not parse"));
            assert_eq!(
                std::mem::discriminant(&parsed),
                std::mem::discriminant(&first),
                "{alias} resolved to a different variant than {}",
                group[0]
            );
        }
    }
}

#[test]
fn the_latin_oc_and_the_cyrillic_oc_both_parse() {
    // "ос" (Cyrillic о+с) and "oc" (Latin o+c) look identical but are
    // different byte sequences; both are user-typed.
    let cyrillic = "\u{043e}\u{0441}";
    let latin = "oc";

    assert_ne!(cyrillic, latin);
    assert!(InlineKeywords::from_str(cyrillic).is_ok());
    assert!(InlineKeywords::from_str(latin).is_ok());
}

#[test]
fn inline_keyword_matching_is_exact() {
    for text in ["", "flags", " flag", "FLAG", "flag "] {
        assert!(InlineKeywords::from_str(text).is_err(), "{text:?}");
    }
}

#[test]
fn every_inline_command_alias_is_also_an_inline_keyword() {
    // `InlineCommands` is the subset of keywords that take a payload; each
    // must still be routable as a keyword.
    for alias in [
        "ім'я", "імя", "имя", "name", "ad", "хрю", "hru", "grunt", "xort",
        "прапор", "флаг", "flag", "bayraq", "гіф", "гиф", "gif",
    ] {
        assert!(
            InlineKeywords::from_str(alias).is_ok(),
            "{alias} is an InlineCommand but not an InlineKeyword"
        );
    }
}
