use std::fs;
use std::sync::OnceLock;
use teloxide::types::User;
use walkdir::WalkDir;

use crate::consts::DEFAULT_LANG_TAG;

// Access to localize
pub static LOC: OnceLock<Locale> = OnceLock::new();

pub type LocaleTag = usize;

#[derive(Debug)]
struct Lang {
    tag: String,
    map: serde_json::Map<String, serde_json::Value>,
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
            if entry.file_type().is_file()
                && entry.file_name().to_string_lossy().ends_with(".json")
            {
                // Extract filename as tag
                let tag = entry
                    .file_name()
                    .to_str()
                    .unwrap()
                    .split_once(".json")
                    .unwrap()
                    .0
                    .to_string();

                // Open file
                if let Ok(file) = fs::File::open(entry.path()) {
                    // Read data
                    if let Ok(data) = serde_json::from_reader(file) {
                        // Get as JSON object
                        let json: serde_json::Value = data;
                        if let Some(map) = json.as_object() {
                            // Store
                            let lang = Lang { tag, map: map.to_owned() };
                            langs.push(lang);
                        } else {
                            log::error!(
                                "Locale::new() wrong json '{}'",
                                entry.path().display()
                            )
                        }
                    } else {
                        log::error!(
                            "Locale::new() read error '{}'",
                            entry.path().display()
                        )
                    }
                } else {
                    log::error!(
                        "Locale::new() open error '{}'",
                        entry.path().display()
                    )
                }
            }
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
    fn args<T>(self, hash_args: &[(&str, T)]) -> String
    where
        T: std::fmt::Display,
    {
        let mut res = self;

        let mut key_replace = String::with_capacity(64);

        for (key, value) in hash_args.iter() {
            key_replace.push('{');
            key_replace.push_str(key);
            key_replace.push('}');
            res = res.replace(&key_replace, &value.to_string());
            key_replace.clear();
        }
        res
    }
}

pub fn lng(key: &str, tag: LocaleTag) -> String {
    let s = match LOC.get() {
        Some(s) => s,
        None => return String::from("lang: error"),
    };

    if tag >= s.langs.len() {
        return format!(
            "lang: too big tag '{}' for langs '{}'",
            tag,
            s.langs.len()
        );
    }

    let res = &s.langs[tag].map;

    let res = match res.get(key) {
        Some(data) => data,
        None => return format!("lang: key '{}' not found", key),
    };

    let res = match res {
        serde_json::Value::String(res) => res,
        _ => return format!("lang: key '{}' not a string", key),
    };

    res.to_owned()
}

pub fn get_tag_opt(from: Option<&User>) -> &str {
    let Some(from) = from else {
        return DEFAULT_LANG_TAG
    };

    from.language_code.as_deref().unwrap_or(DEFAULT_LANG_TAG)
}

pub fn get_tag(from: &User) -> &str {
    from.language_code.as_deref().unwrap_or(DEFAULT_LANG_TAG)
}

pub fn tag(tag: &str) -> LocaleTag {
    let s = match LOC.get() {
        Some(s) => s,
        None => return 0,
    };

    s.langs
        .binary_search_by(|elem| elem.tag.as_str().cmp(tag))
        .unwrap_or(s.def_tag)
}

pub fn get_langs() -> Vec<&'static str> {
    let s = LOC.get().expect("No langs set currently!");

    s.langs.iter().map(|item| item.tag.as_str()).collect()
}
