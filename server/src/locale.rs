#![allow(dead_code)]

use std::collections::HashMap;
use std::path::PathBuf;
use shared::locale::{Language, TranslationInserter, Translations};
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
    translations: Translations,
    identities: HashMap<Language, Identities>,
}

impl LocaleManager {
    pub fn new(config: &Config) -> Self {
        let mut manager = Self {
            translations: Translations::new(),
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
        let mut i = TranslationInserter::new(Language::English, &mut self.translations);

        // User-facing error messages only
        i.add("error.invalid_token", "invalid token");
        i.add("error.pending", "pending");
        i.add("error.game_not_found", "game not found");
        i.add("error.invalid_input", "invalid input");
        i.add("error.internal_server_error", "internal server error");
        i.add("error.timeout", "timeout");
        i.add("error.not_found", "Not found");

        // Game responses (user-facing)
        i.add("game.final_answer", "I'm {}");
        i.add("game.weird_question", "Weird question, skip...");
        i.add("game.gpt_fallback", "UNABLE; this is weird");

        // Cheat detection phrases (user input matching)
        i.add("cheat.im_loser", "IM LOSER");
        i.add("cheat.i_am_loser", "I AM LOSER");
        i.add("cheat.im_a_loser", "IM A LOSER");
    }
    
    fn load_czech(&mut self) {
        let mut i = TranslationInserter::new(Language::Czech, &mut self.translations);

        // User-facing error messages only
        i.add("error.invalid_token", "neplatný token");
        i.add("error.pending", "čeká se");
        i.add("error.game_not_found", "hra nenalezena");
        i.add("error.invalid_input", "neplatný vstup");
        i.add("error.internal_server_error", "vnitřní chyba serveru");
        i.add("error.timeout", "časový limit");
        i.add("error.not_found", "Nenalezeno");

        // Game responses (user-facing)
        i.add("game.final_answer", "Jsem {}");
        i.add("game.weird_question", "Podivná otázka, přeskočit...");
        i.add("game.gpt_fallback", "NEMOHU; to je divné");

        // Cheat detection phrases (user input matching)
        i.add("cheat.im_loser", "JSEM PORAŽENÝ");
        i.add("cheat.i_am_loser", "JÁ JSEM PORAŽENÝ");
        i.add("cheat.im_a_loser", "JSEM NEÚSPĚŠNÝ");
    }
    
    pub fn get(&self, lang: &Language, key: &str) -> String {
        self.translations.get(lang, key)
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
            log::warn!("No identities found for language {:?}", lang);
            None
        } else {
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
