use std::collections::HashMap;
use log::info;
use shared::locale::Language;

#[derive(Debug, Clone)]
pub struct LocaleManager {
    translations: HashMap<Language, HashMap<String, String>>,
    current_language: Language,
}

impl LocaleManager {
    pub fn new() -> Self {
        let mut manager = Self {
            translations: HashMap::new(),
            current_language: Language::English,
        };
        
        manager.load_english();
        manager.load_czech();
        
        // Try to load language from localStorage
        if let Some(storage_result) = web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .map(|storage| storage.get_item("game_language"))
        {
            if let Ok(Some(lang_str)) = storage_result {
                if let Some(lang) = Language::from_str(&lang_str) {
                    manager.current_language = lang;
                }
            }
        }
        
        manager
    }
    
    fn load_english(&mut self) {
        let mut strings = HashMap::new();
        
        // Page titles and headers
        strings.insert("ui.page_title".to_string(), "Guess Who".to_string());
        strings.insert("ui.game_header".to_string(), "Guess Who".to_string());
        strings.insert("ui.new_game".to_string(), "New game".to_string());
        strings.insert("ui.404".to_string(), "404".to_string());
        strings.insert("ui.server_error".to_string(), "ServerError".to_string());
        
        // Game component
        strings.insert("game.prompt".to_string(), "I have a hidden identity. Try to guess who I am. Ask your question...".to_string());
        strings.insert("game.instructions_toggle".to_string(), "Game Instructions".to_string());
        strings.insert("game.rule1".to_string(), "Only the questions that can be answered with YES or NO are allowed.".to_string());
        strings.insert("game.rule2".to_string(), "If a question cannot be answered with a simple yes/no, the response will be UNABLE.".to_string());
        strings.insert("game.rule3".to_string(), "Type: \"I'M LOSER\", I'll reveal my identity and explain my answers.".to_string());
        
        // UI elements
        strings.insert("ui.send".to_string(), "Send".to_string());
        strings.insert("ui.game_id".to_string(), "Game Id: {}".to_string());
        
        // Verdict labels
        strings.insert("verdict.yes".to_string(), "Yes".to_string());
        strings.insert("verdict.no".to_string(), "No".to_string());
        strings.insert("verdict.unable".to_string(), "Unable".to_string());
        strings.insert("verdict.final".to_string(), "Final".to_string());
        strings.insert("verdict.na".to_string(), "N/A".to_string());
        
        // Language selector
        strings.insert("ui.language".to_string(), "Language".to_string());
        
        // Confirmation dialog
        strings.insert("dialog.confirm_language_switch".to_string(), "Switching languages will end the current game. Do you really want to switch?".to_string());
        strings.insert("dialog.yes".to_string(), "Yes".to_string());
        strings.insert("dialog.no".to_string(), "No".to_string());
        
        self.translations.insert(Language::English, strings);
    }
    
    fn load_czech(&mut self) {
        let mut strings = HashMap::new();
        
        // Page titles and headers
        strings.insert("ui.page_title".to_string(), "Hádej kdo jsem".to_string());
        strings.insert("ui.game_header".to_string(), "Hádej kdo jsem".to_string());
        strings.insert("ui.new_game".to_string(), "Nová hra".to_string());
        strings.insert("ui.404".to_string(), "404".to_string());
        strings.insert("ui.server_error".to_string(), "Chyba serveru".to_string());
        
        // Game component
        strings.insert("game.prompt".to_string(), "Ptej se!".to_string());
        strings.insert("game.instructions_toggle".to_string(), "Pravidla hry".to_string());
        strings.insert("game.rule1".to_string(), "Jsou povoleny pouze otázky, na které lze odpovědět ANO nebo NE.".to_string());
        strings.insert("game.rule2".to_string(), "Pokud na otázku nelze odpovědět jednoduchým ano/ne, odpověď bude NELZE.".to_string());
        strings.insert("game.rule3".to_string(), "Napište: \"konec\" a já odhalím svou identitu a vysvětlím své odpovědi.".to_string());
        
        // UI elements
        strings.insert("ui.send".to_string(), "Odeslat".to_string());
        strings.insert("ui.game_id".to_string(), "ID hry: {}".to_string());
        
        // Verdict labels
        strings.insert("verdict.yes".to_string(), "Ano".to_string());
        strings.insert("verdict.no".to_string(), "Ne".to_string());
        strings.insert("verdict.unable".to_string(), "Nelze".to_string());
        strings.insert("verdict.final".to_string(), "Konec".to_string());
        strings.insert("verdict.na".to_string(), "N/A".to_string());
        
        // Language selector
        strings.insert("ui.language".to_string(), "Jazyk".to_string());
        
        // Confirmation dialog
        strings.insert("dialog.confirm_language_switch".to_string(), "Změna jazyka ukončí aktuální hru. Opravdu mám hru ukončit?".to_string());
        strings.insert("dialog.yes".to_string(), "Ano".to_string());
        strings.insert("dialog.no".to_string(), "Ne".to_string());
        
        self.translations.insert(Language::Czech, strings);
    }
    
    pub fn get_current_language(&self) -> &Language {
        &self.current_language
    }
    
    pub fn set_language(&mut self, lang: Language) {
        let lang_code = lang.to_code();
        self.current_language = lang;
        
        // Save to localStorage
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
        {
            let _ = storage.set_item("game_language", lang_code);
        }
    }
    
    pub fn get(&self, key: &str) -> String {
        self.get_for_language(&self.current_language, key)
    }
    
    pub fn get_for_language(&self, lang: &Language, key: &str) -> String {
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
                        web_sys::console::warn_1(&format!("Missing locale key: {} for language: {:?}", key, lang).into());
                        format!("MISSING_LOCALE_KEY[{}]", key)
                    })
            })
    }
    
    #[allow(dead_code)]
    pub fn get_formatted(&self, key: &str, args: &[&str]) -> String {
        let template = self.get(key);
        let mut result = template;
        
        // Simple replacement - replace {} with arguments in order
        for arg in args {
            if let Some(pos) = result.find("{}") {
                result.replace_range(pos..pos+2, arg);
            }
        }
        
        result
    }
    
    #[allow(dead_code)]
    pub fn available_languages(&self) -> Vec<Language> {
        vec![Language::English, Language::Czech]
    }
}

impl Default for LocaleManager {
    fn default() -> Self {
        Self::new()
    }
}

// Global locale manager instance using thread_local for WASM compatibility
thread_local! {
    static LOCALE_MANAGER: std::cell::RefCell<LocaleManager> = std::cell::RefCell::new(LocaleManager::new());
}

pub fn get_current_language() -> Language {
    LOCALE_MANAGER.with(|manager| manager.borrow().get_current_language().clone())
}

pub fn set_language(lang: Language) {
    info!("set language to: {:?} (code: {})", lang, lang.to_code());
    LOCALE_MANAGER.with(|manager| manager.borrow_mut().set_language(lang));
}

#[allow(dead_code)]
pub fn available_languages() -> Vec<Language> {
    LOCALE_MANAGER.with(|manager| manager.borrow().available_languages())
}

// Convenience functions
pub fn t(key: &str) -> String {
    LOCALE_MANAGER.with(|manager| manager.borrow().get(key))
}

#[allow(dead_code)]
pub fn tf(key: &str, args: &[&str]) -> String {
    LOCALE_MANAGER.with(|manager| manager.borrow().get_formatted(key, args))
}