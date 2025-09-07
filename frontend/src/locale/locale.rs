use log::info;
use shared::locale::{Language, Translations};
use shared::locale::TranslationInserter;
use shared::locale::Localizer;

#[derive(Debug, Clone)]
pub struct LocaleManager {
    translations: Translations,
    current_language: Language,
}

impl LocaleManager {
    pub fn new() -> Self {
        let mut manager = Self {
            translations: Translations::new(),
            current_language: Language::English,
        };

        let mut inserter = TranslationInserter::new(Language::English, &mut manager.translations);
        crate::locale::en::load(&mut inserter);

        let mut inserter = TranslationInserter::new(Language::Czech, &mut manager.translations);
        crate::locale::cs::load(&mut inserter);
        
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
        self.translations.get(lang, key)
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

pub fn t_for_language(lang: &Language, key: &str) -> String {
    LOCALE_MANAGER.with(|manager| manager.borrow().get_for_language(lang, key))
}

#[allow(dead_code)]
pub fn tf(key: &str, args: &[&str]) -> String {
    LOCALE_MANAGER.with(|manager| manager.borrow().get_formatted(key, args))
}

pub fn t_shared<T: Localizer>(val: &T) -> String {
    val.to_localized_string(&get_current_language())
}