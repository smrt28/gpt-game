use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum Language {
    #[serde(rename = "en")]
    English,
    #[serde(rename = "cs")]
    Czech,
}

impl Language {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "en" | "english" => Some(Language::English),
            "cs" | "czech" | "česky" | "cesky" => Some(Language::Czech),
            _ => None,
        }
    }
    
    pub fn to_instruction(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Czech => "Czech",
        }
    }
    
    pub fn to_code(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Czech => "cs",
        }
    }
    
    pub fn to_display_name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Czech => "Česky",
        }
    }
}

impl Default for Language {
    fn default() -> Self {
        Language::English
    }
}

#[derive(Clone, Debug)]
pub struct Translations {
    translations: HashMap<Language, HashMap<String, String>>,
}


impl Translations {
    pub fn new() -> Self {
        Self {
            translations: HashMap::new(),
        }
    }

    pub fn get(&self, lang: &Language, key: &str) -> String {
        self.translations.get(lang)
            .and_then(|eng_map| eng_map.get(key))
            .cloned()
            .unwrap_or_else(|| {
                format!("MISSING_LOCALE_KEY[{}]", key)
            })
    }

    fn add(&mut self, lang: Language, key: &str, value: &str) {
        self.translations.entry(lang)
            .or_default()
            .insert(key.to_string(), value.to_string());
    }
}

pub struct TranslationInserter<'a> {
    translations: &'a mut Translations,
    lang: Language,
}

impl<'a> TranslationInserter<'a>  {
    pub fn new(lang: Language, translations: &'a mut Translations) -> Self {
        Self { lang, translations }
    }

    pub fn add(&mut self, key: &str, value: &str) {
        self.translations.add(self.lang.clone(), key, value);
    }
}

pub trait Localizer {
    fn to_localized_string(&self, lang: &Language) -> String;
}