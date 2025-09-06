use yew::prelude::*;
use shared::locale::Language;
use crate::locale::{get_current_language, t_for_language};
use crate::language_logic::{should_show_language_dialog, switch_language};
use crate::board_component::Act;
use gloo_storage::{LocalStorage, Storage};

#[derive(Properties, PartialEq)]
pub struct LanguageSelectorProps {
    #[prop_or(false)]
    pub has_active_game: bool,
    #[prop_or(None)]
    pub board_dispatch: Option<Callback<Act>>,
    #[prop_or(None)]
    pub on_game_invalidated: Option<Callback<()>>,
    #[prop_or(None)]
    pub on_language_changed: Option<Callback<()>>,
}

#[function_component(LanguageSelector)]
pub fn language_selector(props: &LanguageSelectorProps) -> Html {
    let current_language = get_current_language();
    let show_language_dialog = use_state(|| false);
    let pending_language = use_state(|| None::<Language>);
    let render_trigger = use_state(|| 0u32);
    let pl = pending_language.as_ref().unwrap_or(&current_language);

    let on_language_change = {
        let show_language_dialog = show_language_dialog.clone();
        let pending_language = pending_language.clone();
        let render_trigger = render_trigger.clone();
        let on_language_changed = props.on_language_changed.clone();
        let has_active_game = props.has_active_game;
        
        Callback::from(move |lang: Language| {
            if should_show_language_dialog(lang.clone(), has_active_game) {
                pending_language.set(Some(lang));
                show_language_dialog.set(true);
            } else {
                // Switch immediately if no active game
                switch_language(lang);
                render_trigger.set(*render_trigger + 1);
                if let Some(callback) = &on_language_changed {
                    callback.emit(());
                }
            }
        })
    };

    let on_confirm_language_change = {
        let show_language_dialog = show_language_dialog.clone();
        let pending_language = pending_language.clone();
        let render_trigger = render_trigger.clone();
        let board_dispatch = props.board_dispatch.clone();
        let on_game_invalidated = props.on_game_invalidated.clone();
        let on_language_changed = props.on_language_changed.clone();
        
        Callback::from(move |confirmed: bool| {
            show_language_dialog.set(false);
            
            if confirmed {
                if let Some(lang) = (*pending_language).clone() {
                    switch_language(lang);
                    render_trigger.set(*render_trigger + 1);
                    if let Some(callback) = &on_language_changed {
                        callback.emit(());
                    }
                    // Clear the token and invalidate the game state if callbacks are provided
                    if board_dispatch.is_some() {
                        let _ = LocalStorage::delete("token");
                        if let Some(dispatch) = &board_dispatch {
                            dispatch.emit(Act::InvalidGame);
                        }
                        if let Some(invalidated) = &on_game_invalidated {
                            invalidated.emit(());
                        }
                    }
                }
            }
            
            pending_language.set(None);
        })
    };

    html! {
        <>
            <div class="language-selector">
                <button 
                    class={if current_language == Language::English { "flag-button active" } else { "flag-button" }}
                    onclick={
                        let on_language_change = on_language_change.clone();
                        Callback::from(move |_| on_language_change.emit(Language::English))
                    }
                    title="English"
                >
                    {"🇬🇧"}
                </button>
                <button 
                    class={if current_language == Language::Czech { "flag-button active" } else { "flag-button" }}
                    onclick={
                        let on_language_change = on_language_change.clone();
                        Callback::from(move |_| on_language_change.emit(Language::Czech))
                    }
                    title="Česky"
                >
                    {"🇨🇿"}
                </button>
            </div>

            {
                if *show_language_dialog {
                    html! {
                        <div class="dialog-overlay">
                            <div class="dialog-box">
                                <p class="dialog-message">{t_for_language(pl, "dialog.confirm_language_switch")}</p>
                                <div class="dialog-buttons">
                                    <button 
                                        class="dialog-button dialog-button--yes"
                                        onclick={
                                            let on_confirm = on_confirm_language_change.clone();
                                            Callback::from(move |_| on_confirm.emit(true))
                                        }
                                    >
                                        {"Yes"}
                                    </button>
                                    <button 
                                        class="dialog-button dialog-button--no"
                                        onclick={
                                            let on_confirm = on_confirm_language_change.clone();
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

