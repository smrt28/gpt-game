use crate::locale::{Language, Localizer};
use crate::messages::GameTemplateStatus;

impl Localizer for GameTemplateStatus {
    fn to_localized_string(&self, lang: &Language) -> String {
        match lang {
            Language::Czech => {
                match self {
                    GameTemplateStatus::Ok => "ok",
                    GameTemplateStatus::EmptyIdentity => "You muse enter the identity.",
                    GameTemplateStatus::ToLongIdentity => "Identity is too long.",
                    GameTemplateStatus::NotSet => "Not set",
                }
            }

            Language::English => {
                match self {
                    GameTemplateStatus::Ok => "ok",
                    GameTemplateStatus::EmptyIdentity => "You muse enter the identity.",
                    GameTemplateStatus::ToLongIdentity => "Identity is too long.",
                    GameTemplateStatus::NotSet => "Not set",
                }
            }
        }.to_string()
    }
}