#![allow(dead_code)]

use std::collections::HashMap;
use std::path::PathBuf;
use shared::locale::Language;
use crate::config::Config;

#[derive(Default, Clone, Debug)]
pub struct Identities {
    pub list: Vec<String>,
}

impl Identities {
    pub fn read(path: PathBuf) -> Result<Self, anyhow::Error> {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("Failed to read identities file {:?}: {}", path, e))?;
        
        let list: Vec<String> = content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| line.to_string())
            .collect();
        
        if list.is_empty() {
            return Err(anyhow::anyhow!("No identities found in file {:?}", path));
        }
        
        Ok(Self { list })
    }
}


#[derive(Debug, Clone)]
pub struct LocaleManager {
    translations: HashMap<Language, HashMap<String, String>>,
    identities: HashMap<Language, Identities>,

}

impl LocaleManager {
    pub fn new(config: &Config) -> Self {
        let mut manager = Self {
            translations: HashMap::new(),
            identities: HashMap::new(),
        };
        
        manager.load_english();
        manager.load_czech();

        // Load identities for each language
        if let Ok(english_identities) = Identities::read(config.get_identities_file(&Language::English)) {
            manager.identities.insert(Language::English, english_identities);
        } else {
            log::warn!("Failed to load English identities, using empty list");
            manager.identities.insert(Language::English, Identities::default());
        }

        if let Ok(czech_identities) = Identities::read(config.get_identities_file(&Language::Czech)) {
            manager.identities.insert(Language::Czech, czech_identities);
        } else {
            log::warn!("Failed to load Czech identities, using empty list");
            manager.identities.insert(Language::Czech, Identities::default());
        }

        manager
    }
    
    fn load_english(&mut self) {
        let mut strings = HashMap::new();
        
        // User-facing error messages only
        strings.insert("error.invalid_token".to_string(), "invalid token".to_string());
        strings.insert("error.pending".to_string(), "pending".to_string());
        strings.insert("error.game_not_found".to_string(), "game not found".to_string());
        strings.insert("error.invalid_input".to_string(), "invalid input".to_string());
        strings.insert("error.internal_server_error".to_string(), "internal server error".to_string());
        strings.insert("error.timeout".to_string(), "timeout".to_string());
        strings.insert("error.not_found".to_string(), "Not found".to_string());
        
        // Game responses (user-facing)
        strings.insert("game.final_answer".to_string(), "I'm {}".to_string());
        strings.insert("game.weird_question".to_string(), "Weird question, skip...".to_string());
        strings.insert("game.gpt_fallback".to_string(), "UNABLE; this is weird".to_string());
        
        // Cheat detection phrases (user input matching)
        strings.insert("cheat.im_loser".to_string(), "IM LOSER".to_string());
        strings.insert("cheat.i_am_loser".to_string(), "I AM LOSER".to_string());
        strings.insert("cheat.im_a_loser".to_string(), "IM A LOSER".to_string());
        
        self.translations.insert(Language::English, strings);
    }
    
    fn load_czech(&mut self) {
        let mut strings = HashMap::new();
        
        // User-facing error messages only
        strings.insert("error.invalid_token".to_string(), "neplatný token".to_string());
        strings.insert("error.pending".to_string(), "čeká se".to_string());
        strings.insert("error.game_not_found".to_string(), "hra nenalezena".to_string());
        strings.insert("error.invalid_input".to_string(), "neplatný vstup".to_string());
        strings.insert("error.internal_server_error".to_string(), "vnitřní chyba serveru".to_string());
        strings.insert("error.timeout".to_string(), "časový limit".to_string());
        strings.insert("error.not_found".to_string(), "Nenalezeno".to_string());
        
        // Game responses (user-facing)
        strings.insert("game.final_answer".to_string(), "Jsem {}".to_string());
        strings.insert("game.weird_question".to_string(), "Podivná otázka, přeskočit...".to_string());
        strings.insert("game.gpt_fallback".to_string(), "NEMOHU; to je divné".to_string());
        
        // Cheat detection phrases (user input matching)
        strings.insert("cheat.im_loser".to_string(), "JSEM PORAŽENÝ".to_string());
        strings.insert("cheat.i_am_loser".to_string(), "JÁ JSEM PORAŽENÝ".to_string());
        strings.insert("cheat.im_a_loser".to_string(), "JSEM NEÚSPĚŠNÝ".to_string());
        
        self.translations.insert(Language::Czech, strings);
    }
    
    pub fn get(&self, lang: &Language, key: &str) -> String {
        self.translations
            .get(lang)
            .and_then(|lang_map| lang_map.get(key))
            .cloned()
            .unwrap_or_else(|| {
                // Fallback to English if key not found in requested language
                self.translations
                    .get(&Language::English)
                    .and_then(|eng_map| eng_map.get(key))
                    .cloned()
                    .unwrap_or_else(|| {
                        eprintln!("Missing locale key: {} for language: {:?}", key, lang);
                        format!("MISSING_LOCALE_KEY[{}]", key)
                    })
            })
    }
    
    pub fn get_formatted(&self, lang: &Language, key: &str, args: &[&str]) -> String {
        let template = self.get(lang, key);
        let mut result = template;
        
        // Simple replacement - replace {} with arguments in order
        for arg in args {
            if let Some(pos) = result.find("{}") {
                result.replace_range(pos..pos+2, arg);
            }
        }
        
        result
    }
    
    pub fn get_cheat_phrases(&self, lang: &Language) -> Vec<String> {
        vec![
            self.get(lang, "cheat.im_loser"),
            self.get(lang, "cheat.i_am_loser"),
            self.get(lang, "cheat.im_a_loser"),
        ]
    }
    
    pub fn available_languages(&self) -> Vec<Language> {
        vec![Language::English, Language::Czech]
    }
    
    pub fn get_identities(&self, lang: &Language) -> Option<&Identities> {
        self.identities.get(lang)
    }
    
    pub fn get_random_identity(&self, lang: &Language) -> Option<String> {
        let identities = self.identities.get(lang)?;
        if identities.list.is_empty() {
            None
        } else {
            use rand::seq::SliceRandom;
            use rand::prelude::*;
            let mut rng = rand::rng();
            identities.list.choose(&mut rng).cloned()
        }
    }
}

// Global locale manager instance using std::sync::OnceLock for thread safety
static LOCALE_MANAGER: std::sync::OnceLock<LocaleManager> = std::sync::OnceLock::new();

pub fn init_locale(config: &Config) {
    LOCALE_MANAGER.get_or_init(|| LocaleManager::new(config));
}

pub fn get_locale_manager() -> &'static LocaleManager {
    LOCALE_MANAGER.get().expect("Locale manager not initialized. Call init_locale() first.")
}

// Convenience functions
pub fn t(lang: &Language, key: &str) -> String {
    get_locale_manager().get(lang, key)
}

pub fn tf(lang: &Language, key: &str, args: &[&str]) -> String {
    get_locale_manager().get_formatted(lang, key, args)
}

pub fn get_random_identity(lang: &Language) -> Option<String> {
    get_locale_manager().get_random_identity(lang)
}

pub fn get_identities(lang: &Language) -> Option<&'static Identities> {
    get_locale_manager().get_identities(lang)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_language_from_str() {
        assert_eq!(Language::from_str("en"), Some(Language::English));
        assert_eq!(Language::from_str("cs"), Some(Language::Czech));
        assert_eq!(Language::from_str("invalid"), None);
    }
    
    #[test]
    fn test_locale_manager_get() {
        let manager = LocaleManager::new();
        assert_eq!(manager.get(&Language::English, "error.invalid_token"), "invalid token");
        assert_eq!(manager.get(&Language::Czech, "error.invalid_token"), "neplatný token");
    }
    
    #[test]
    fn test_locale_manager_get_formatted() {
        let manager = LocaleManager::new();
        let result = manager.get_formatted(&Language::English, "game.final_answer", &["Pikachu"]);
        assert_eq!(result, "I'm Pikachu");
        
        let result_cs = manager.get_formatted(&Language::Czech, "game.final_answer", &["Pikachu"]);
        assert_eq!(result_cs, "Jsem Pikachu");
    }
}