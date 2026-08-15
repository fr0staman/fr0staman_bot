//! A missing key is invisible at compile time — `lng` renders
//! `lang: key '...' not found` straight into a user-facing message.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use crate::{
    config::consts::DEFAULT_LANG_TAG,
    lang::{get_langs, lng},
    test_support::init_lang,
};

fn locales_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("locales")
}

fn load_all() -> BTreeMap<String, BTreeSet<String>> {
    let mut out = BTreeMap::new();

    for entry in std::fs::read_dir(locales_dir()).expect("no locales/ dir") {
        let path = entry.expect("bad dir entry").path();
        let Some(tag) = path
            .file_name()
            .and_then(|n| n.to_str())
            .and_then(|n| n.strip_suffix(".json"))
        else {
            continue;
        };

        let raw = std::fs::read_to_string(&path).expect("unreadable locale");
        let json: serde_json::Value =
            serde_json::from_str(&raw).expect("locale is not valid JSON");
        let object = json.as_object().expect("locale is not a JSON object");

        out.insert(tag.to_owned(), object.keys().cloned().collect());
    }

    out
}

#[test]
fn the_expected_locales_are_present() {
    let all = load_all();
    let tags: Vec<&str> = all.keys().map(String::as_str).collect();

    assert_eq!(tags, ["az", "en", "ru", "uk"]);
    assert!(all.contains_key(DEFAULT_LANG_TAG));
}

/// Keys a locale may carry without the default locale having them.
const LOCALE_LOCAL_KEYS: &[&str] = &["AUTHOR_OF_THIS_TRANSLATION"];

#[test]
fn every_locale_has_the_same_keys_as_the_default_one() {
    let all = load_all();
    let reference = &all[DEFAULT_LANG_TAG];

    let mut problems = Vec::new();

    for (tag, keys) in &all {
        if tag == DEFAULT_LANG_TAG {
            continue;
        }

        let missing: Vec<_> = reference.difference(keys).collect();
        let extra: Vec<_> = keys
            .difference(reference)
            .filter(|k| !LOCALE_LOCAL_KEYS.contains(&k.as_str()))
            .collect();

        if !missing.is_empty() {
            problems.push(format!("{tag} is missing: {missing:?}"));
        }
        if !extra.is_empty() {
            problems.push(format!("{tag} has keys {DEFAULT_LANG_TAG} lacks: {extra:?}"));
        }
    }

    assert!(problems.is_empty(), "locale drift:\n{}", problems.join("\n"));
}

#[test]
fn every_locale_value_is_a_string() {
    // `Locale::new` panics on a non-string value, taking the bot down at
    // startup.
    for entry in std::fs::read_dir(locales_dir()).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let raw = std::fs::read_to_string(&path).unwrap();
        let json: serde_json::Value = serde_json::from_str(&raw).unwrap();

        for (key, value) in json.as_object().unwrap() {
            assert!(
                value.is_string(),
                "{}: key {key} is {value:?}, not a string",
                path.display()
            );
        }
    }
}

#[test]
fn every_key_resolves_in_every_locale_at_runtime() {
    init_lang();

    let all = load_all();
    let reference = &all[DEFAULT_LANG_TAG];
    let langs = get_langs();

    for (ltag, _) in langs.iter().enumerate() {
        for key in reference {
            let text = lng(key, ltag);
            assert!(
                !text.starts_with("lang: key"),
                "tag {ltag} could not resolve {key}"
            );
        }
    }
}

#[test]
fn the_achievement_names_cover_every_code() {
    use crate::services::achievements::Ach;
    use strum::VariantArray;

    init_lang();

    let langs = get_langs();

    for (ltag, tag) in langs.iter().enumerate() {
        for ach in Ach::VARIANTS {
            let key = format!("Achievement_{}", *ach as i16);
            let text = lng(&key, ltag);
            assert!(
                !text.starts_with("lang:"),
                "{tag} is missing {key} ({ach:?})"
            );
        }
    }
}

#[test]
fn the_growth_status_messages_cover_every_status() {
    use crate::enums::PigGrowthStatus;

    init_lang();

    for (ltag, tag) in get_langs().iter().enumerate() {
        for status in [
            PigGrowthStatus::Lost,
            PigGrowthStatus::Maintained,
            PigGrowthStatus::Gained,
        ] {
            let key = format!("GamePigGrowMessage_{}", status.into_str());
            assert!(!lng(&key, ltag).starts_with("lang:"), "{tag}: {key}");
        }
    }
}

#[test]
fn the_duel_result_messages_cover_every_outcome() {
    use crate::enums::DuelResult;

    init_lang();

    for (ltag, tag) in get_langs().iter().enumerate() {
        for status in [
            DuelResult::Draw,
            DuelResult::Win,
            DuelResult::Critical,
            DuelResult::Knockout,
        ] {
            let key = format!("InlineDuelMessage_{}", status.into_str());
            assert!(!lng(&key, ltag).starts_with("lang:"), "{tag}: {key}");
        }
    }
}

#[test]
fn every_advertised_command_has_a_description_in_every_locale() {
    use crate::{config::consts::IGNORED_COMMANDS, enums::MyCommands};
    use teloxide::utils::command::BotCommands;

    init_lang();

    let mut commands = MyCommands::bot_commands();
    commands.retain(|c| !IGNORED_COMMANDS.contains(&c.command.as_str()));

    for (ltag, tag) in get_langs().iter().enumerate() {
        for command in &commands {
            let key = format!("{}_desc", command.command);
            let text = lng(&key, ltag);
            assert!(!text.starts_with("lang:"), "{tag} is missing {key}");
            // Telegram rejects command descriptions over 256 chars.
            assert!(text.len() <= 256, "{tag}/{key} is {} bytes", text.len());
        }
    }
}

#[test]
fn the_plural_forms_exist_for_every_unit() {
    init_lang();

    for (ltag, tag) in get_langs().iter().enumerate() {
        for unit in ["hour", "minute", "second"] {
            for rule in 0..3 {
                let key = format!("{unit}_{rule}");
                assert!(!lng(&key, ltag).starts_with("lang:"), "{tag}: {key}");
            }
        }
    }
}
