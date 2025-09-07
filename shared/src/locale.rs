use std::collections::HashMap;
use serde::{Deserialize, Serialize};

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