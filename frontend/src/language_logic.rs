use shared::locale::Language;
use crate::locale::{get_current_language, set_language};

/// Simple utility functions for language management
pub fn should_show_language_dialog(target_lang: Language, has_active_game: bool) -> bool {
    if get_current_language() == target_lang {
        return false;
    }
    has_active_game
}

pub fn switch_language(lang: Language) {
    set_language(lang);
}