use std::fs;
use std::path::Path;
use std::sync::OnceLock;
use teloxide::types::User;
use walkdir::WalkDir;

use crate::config::consts::DEFAULT_LANG_TAG;

// Access to localize
pub static LANG: OnceLock<Locale> = OnceLock::new();

pub type LocaleTag = usize;

#[derive(Debug)]
struct Lang {
    tag: String,
    map: ahash::AHashMap<String, String>,
}

#[derive(Debug)]
pub struct Locale {
    langs: Vec<Lang>,
    def_tag: usize,
}

impl Locale {
    pub fn new(set_def_tag: &str) -> Self {
        Self::load_from("locales/", set_def_tag)
    }

    /// Same as [`Locale::new`], but with an explicit directory.
    ///
    /// [`Locale::new`] resolves `locales/` relative to the current working
    /// directory, which is fine for the bot but brittle for tests; they pass
    /// a path anchored to `CARGO_MANIFEST_DIR` instead.
    pub fn load_from(dir: impl AsRef<Path>, set_def_tag: &str) -> Self {
        let mut langs = vec![];

        // Load "tag".json from directory
        for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }

            let Some(file_name_parts) =
                entry.file_name().to_str().and_then(|v| v.split_once(".json"))
            else {
                continue;
            };

            // Extract filename as tag
            let tag = file_name_parts.0.to_string();

            // Open file
            let Ok(file) = fs::File::open(entry.path()) else {
                log::error!(
                    "Locale::new() open error '{}'",
                    entry.path().display()
                );
                continue;
            };

            // Read data
            let Ok(data) = serde_json::from_reader(file) else {
                log::error!(
                    "Locale::new() read error '{}'",
                    entry.path().display()
                );
                continue;
            };

            // Get an json object
            let serde_json::Value::Object(json_map) = data else {
                log::error!(
                    "Locale::new() wrong json '{}'",
                    entry.path().display()
                );
                continue;
            };

            // Store
            let mut map = ahash::AHashMap::default();
            for (key, value) in json_map.into_iter() {
                let value = match value {
                    serde_json::Value::String(value) => value,
                    _ => panic!("Locale::new(): only String can be passed!"),
                };
                map.insert(key, value);
            }
            let lang = Lang { tag, map };
            langs.push(lang);
        }

        // Sorting for binary search
        langs.sort_by(|a, b| a.tag.cmp(&b.tag));

        // After sort, store default locale
        let def_tag = langs
            .binary_search_by(|elem| elem.tag.as_str().cmp(set_def_tag))
            .expect("Invalid default lang!");

        let info = langs.iter().fold(String::from("Loaded lang:"), |acc, l| {
            format!("{} {}", acc, l.tag)
        });
        log::info!("{} | default: {}", info, set_def_tag);

        Self { langs, def_tag }
    }
}

pub trait InnerLang {
    fn args<T>(self, hash_args: &[(&str, T)]) -> String
    where
        T: std::fmt::Display;
}

impl InnerLang for String {
    fn args<T>(mut self, hash_args: &[(&str, T)]) -> String
    where
        T: std::fmt::Display,
    {
        let mut key_replace = String::with_capacity(32);

        for (key, value) in hash_args {
            key_replace.push('{');
            key_replace.push_str(key);
            key_replace.push('}');
            self = self.replace(&key_replace, &value.to_string());
            key_replace.clear();
        }
        self
    }
}

#[inline]
pub fn lng(key: &str, tag: LocaleTag) -> String {
    let s = LANG.get().expect("Lang is not set!");

    if tag >= s.langs.len() {
        return format!(
            "lang: too big tag '{}' for langs '{}'",
            tag,
            s.langs.len()
        );
    }

    let res = &s.langs[tag].map;

    let Some(res) = res.get(key) else {
        return format!("lang: key '{}' not found", key);
    };

    res.to_owned()
}

#[inline]
pub fn get_tag_opt(from: Option<&User>) -> &str {
    let Some(from) = from else { return DEFAULT_LANG_TAG };

    get_tag(from)
}

#[inline]
pub fn get_tag(from: &User) -> &str {
    from.language_code.as_deref().unwrap_or(DEFAULT_LANG_TAG)
}

/// Priority by "if exists"
/// first tag? || second tag? || fallback_tag
/// In bot functionality that means
/// user forced lang || Chat forced lang || user.language_code
#[inline]
pub fn tag_one_two_or(
    first_opt_tag: Option<&str>,
    second_opt_tag: Option<&str>,
    fallback_tag: &str,
) -> LocaleTag {
    if let Some(tag) = tag_opt(first_opt_tag) {
        return tag;
    }

    if let Some(tag) = tag_opt(second_opt_tag) {
        return tag;
    }

    tag(fallback_tag)
}

/// Priority by "if exists"
/// first tag? || fallback_tag
/// In bot functionality that means
/// user forced lang || user.language_code
#[inline]
pub fn tag_one_or(
    first_opt_tag: Option<&str>,
    fallback_tag: &str,
) -> LocaleTag {
    tag_opt(first_opt_tag).unwrap_or_else(|| tag(fallback_tag))
}

#[inline]
pub fn tag_opt(opt_tag: Option<&str>) -> Option<LocaleTag> {
    let tag = opt_tag?;

    let s = LANG.get()?;

    s.langs.binary_search_by_key(&tag, |elem| &elem.tag).ok()
}

#[inline]
pub fn tag(tag: &str) -> LocaleTag {
    let Some(s) = LANG.get() else { return 0 };

    s.langs.binary_search_by_key(&tag, |elem| &elem.tag).unwrap_or(s.def_tag)
}

pub fn get_langs() -> Vec<String> {
    let s = LANG.get().expect("No langs set currently!");

    s.langs.iter().map(|item| item.tag.clone()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::init_lang;


    #[test]
    fn args_substitutes_every_placeholder() {
        let out = "Hi {name}, you weigh {weight} kg"
            .to_owned()
            .args(&[("name", "Pig"), ("weight", "42")]);

        assert_eq!(out, "Hi Pig, you weigh 42 kg");
    }

    #[test]
    fn args_replaces_a_repeated_placeholder_everywhere() {
        let out = "{n} and {n}".to_owned().args(&[("n", 7)]);
        assert_eq!(out, "7 and 7");
    }

    #[test]
    fn a_missing_placeholder_leaves_the_template_untouched() {
        let out = "no slots here".to_owned().args(&[("name", "Pig")]);
        assert_eq!(out, "no slots here");
    }

    #[test]
    fn an_unsupplied_placeholder_is_left_verbatim() {
        let out = "Hi {name}, {missing}".to_owned().args(&[("name", "Pig")]);
        assert_eq!(out, "Hi Pig, {missing}");
    }

    #[test]
    fn substitution_is_sequential_so_a_value_can_be_rewritten() {
        // Each key is applied in turn over the whole string, so a value
        // containing a later key's placeholder gets substituted too.
        let out = "{a}".to_owned().args(&[("a", "{b}"), ("b", "boom")]);
        assert_eq!(out, "boom");

        // The reverse order is safe.
        let out = "{a}".to_owned().args(&[("b", "boom"), ("a", "{b}")]);
        assert_eq!(out, "{b}");
    }

    #[test]
    fn args_accepts_any_display_value() {
        let out = "{a}/{b}/{c}".to_owned().args(&[("a", "x"), ("b", "y"), ("c", "z")]);
        assert_eq!(out, "x/y/z");

        let out = "{n}".to_owned().args(&[("n", -12i64)]);
        assert_eq!(out, "-12");
    }


    #[test]
    fn known_tags_resolve_and_unknown_ones_fall_back_to_the_default() {
        init_lang();

        for known in ["uk", "en", "ru", "az"] {
            assert_eq!(
                tag_opt(Some(known)).map(|t| get_langs()[t].clone()),
                Some(known.to_owned())
            );
        }

        assert_eq!(tag_opt(Some("zz")), None);
        assert_eq!(tag_opt(None), None);

        // `tag` swallows the miss and returns the default instead.
        assert_eq!(tag("zz"), tag(DEFAULT_LANG_TAG));
    }

    #[test]
    fn the_language_priority_chain_prefers_user_then_chat_then_client() {
        init_lang();

        let user = tag("en");
        let chat = tag("ru");
        let client = tag("az");

        // User setting wins.
        assert_eq!(tag_one_two_or(Some("en"), Some("ru"), "az"), user);
        // No user setting: chat wins.
        assert_eq!(tag_one_two_or(None, Some("ru"), "az"), chat);
        // Neither: the Telegram client language.
        assert_eq!(tag_one_two_or(None, None, "az"), client);
        // Unknown client language: the default.
        assert_eq!(tag_one_two_or(None, None, "zz"), tag(DEFAULT_LANG_TAG));
    }

    #[test]
    fn an_unknown_user_or_chat_setting_falls_through_rather_than_defaulting() {
        init_lang();

        // "zz" is not a locale, so it is skipped and the chat setting applies.
        assert_eq!(tag_one_two_or(Some("zz"), Some("ru"), "az"), tag("ru"));
        assert_eq!(tag_one_or(Some("zz"), "az"), tag("az"));
    }

    #[test]
    fn get_tag_uses_the_clients_language_code() {
        let json = r#"{
            "id": 1,
            "is_bot": false,
            "first_name": "T",
            "language_code": "en"
        }"#;
        let user: User = serde_json::from_str(json).unwrap();
        assert_eq!(get_tag(&user), "en");

        let json = r#"{"id": 1, "is_bot": false, "first_name": "T"}"#;
        let no_lang: User = serde_json::from_str(json).unwrap();
        assert_eq!(get_tag(&no_lang), DEFAULT_LANG_TAG);

        assert_eq!(get_tag_opt(None), DEFAULT_LANG_TAG);
        assert_eq!(get_tag_opt(Some(&user)), "en");
    }


    #[test]
    fn lng_returns_a_marker_string_for_a_missing_key() {
        init_lang();

        assert_eq!(
            lng("DefinitelyNotAKey", tag(DEFAULT_LANG_TAG)),
            "lang: key 'DefinitelyNotAKey' not found"
        );
    }

    #[test]
    fn lng_returns_a_marker_string_for_an_out_of_range_tag() {
        init_lang();

        let out = lng("HelpMessage", 9_999);
        assert!(out.starts_with("lang: too big tag"), "{out}");
    }

    #[test]
    fn the_default_locale_resolves_real_text() {
        let ltag = init_lang();
        let text = lng("HelpMessage", ltag);

        assert!(!text.starts_with("lang:"), "{text}");
        assert!(!text.is_empty());
    }
}
