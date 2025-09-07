use std::collections::HashMap;
use log::info;
use shared::locale::Language;

pub struct Helper<'a> {
    map: &'a mut HashMap<String, String>,
}

impl<'a> Helper<'a> {
    pub fn new(map: &'a mut HashMap<String, String>) -> Self {
        Self {
            map
        }
    }

    pub fn add(&mut self, key: &str, value: &str) {
        self.map.insert(key.to_string(), value.to_string());
    }
}


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
        let mut helper = Helper::new(&mut strings);
        crate::locale::en::load(&mut helper);
        self.translations.insert(Language::English, strings);
    }
    
    fn load_czech(&mut self) {
        let mut strings = HashMap::new();
        let mut helper = Helper::new(&mut strings);
        crate::locale::cs::load(&mut helper);
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

pub fn t_for_language(lang: &Language, key: &str) -> String {
    LOCALE_MANAGER.with(|manager| manager.borrow().get_for_language(lang, key))
}

#[allow(dead_code)]
pub fn tf(key: &str, args: &[&str]) -> String {
    LOCALE_MANAGER.with(|manager| manager.borrow().get_formatted(key, args))
}