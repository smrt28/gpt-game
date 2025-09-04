use yew::prelude::*;
use shared::locale::Language;
use crate::locale::{get_current_language, t_for_language};

#[derive(Properties, PartialEq)]
pub struct LanguageSelectorProps {
    pub on_language_change: Callback<Language>,
    pub pending_language: Option<Language>,
    pub show_dialog: bool,
    pub on_confirm_change: Callback<bool>,
}

#[function_component(LanguageSelector)]
pub fn language_selector(props: &LanguageSelectorProps) -> Html {
    let current_language = get_current_language();
    let pl = props.pending_language.as_ref().unwrap_or(&current_language);

    html! {
        <>
            <div class="language-selector">
                <button 
                    class={if current_language == Language::English { "flag-button active" } else { "flag-button" }}
                    onclick={
                        let on_language_change = props.on_language_change.clone();
                        Callback::from(move |_| on_language_change.emit(Language::English))
                    }
                    title="English"
                >
                    {"🇬🇧"}
                </button>
                <button 
                    class={if current_language == Language::Czech { "flag-button active" } else { "flag-button" }}
                    onclick={
                        let on_language_change = props.on_language_change.clone();
                        Callback::from(move |_| on_language_change.emit(Language::Czech))
                    }
                    title="Česky"
                >
                    {"🇨🇿"}
                </button>
            </div>

            {
                if props.show_dialog {
                    html! {
                        <div class="dialog-overlay">
                            <div class="dialog-box">
                                <p class="dialog-message">{t_for_language(pl, "dialog.confirm_language_switch")}</p>
                                <div class="dialog-buttons">
                                    <button 
                                        class="dialog-button dialog-button--yes"
                                        onclick={
                                            let on_confirm = props.on_confirm_change.clone();
                                            Callback::from(move |_| on_confirm.emit(true))
                                        }
                                    >
                                        {"Yes"}
                                    </button>
                                    <button 
                                        class="dialog-button dialog-button--no"
                                        onclick={
                                            let on_confirm = props.on_confirm_change.clone();
                                            Callback::from(move |_| on_confirm.emit(false))
                                        }
                                    >
                                        {"No"}
                                    </button>
                                </div>
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                }
            }
        </>
    }
}

