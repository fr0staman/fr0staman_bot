use std::fs;
use std::sync::OnceLock;
use teloxide::types::User;
use walkdir::WalkDir;

use crate::consts::DEFAULT_LANG_TAG;

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
        let mut langs = vec![];

        // Load "tag".json from directory
        for entry in WalkDir::new("locales/").into_iter().filter_map(|e| e.ok())
        {
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

    s.langs.binary_search_by(|elem| elem.tag.as_str().cmp(tag)).ok()
}

#[inline]
pub fn tag(tag: &str) -> LocaleTag {
    let Some(s) = LANG.get() else { return 0 };

    s.langs
        .binary_search_by(|elem| elem.tag.as_str().cmp(tag))
        .unwrap_or(s.def_tag)
}

pub fn get_langs() -> Vec<&'static str> {
    let s = LANG.get().expect("No langs set currently!");

    s.langs.iter().map(|item| item.tag.as_str()).collect()
}
