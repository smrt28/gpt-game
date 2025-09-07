use log::info;
use yew::{function_component, html, Html, use_state, Callback};
use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use wasm_bindgen::JsCast;
use yew_router::hooks::use_navigator;
use crate::locale::t;
use crate::language_selector_component::LanguageSelector;
use crate::Route;

#[function_component(CustomGameDesign)]
pub fn custom_game_design() -> Html {
    let navigator = use_navigator().expect("Must be used within a Router");
    let language_render_trigger = use_state(|| 0u32);
    let identity_to_guess = use_state(|| String::new());
    let comment = use_state(|| String::new());

    let on_language_changed = {
        let language_render_trigger = language_render_trigger.clone();
        Callback::from(move |_| {
            language_render_trigger.set(*language_render_trigger + 1);
        })
    };

    let on_identity_input = {
        let identity_to_guess = identity_to_guess.clone();
        Callback::from(move |e: web_sys::InputEvent| {
            if let Some(target) = e.target() {
                if let Ok(input) = target.dyn_into::<HtmlInputElement>() {
                    identity_to_guess.set(input.value());
                }
            }
        })
    };

    let on_comment_input = {
        let comment = comment.clone();
        Callback::from(move |e: web_sys::InputEvent| {
            if let Some(target) = e.target() {
                if let Ok(textarea) = target.dyn_into::<HtmlTextAreaElement>() {
                    comment.set(textarea.value());
                }
            }
        })
    };

    let on_cancel = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::AppHome);
        })
    };

    let on_create = {
        let identity_to_guess = identity_to_guess.clone();
        let comment = comment.clone();
        //let navigator = navigator.clone();
        Callback::from(move |_| {
            // TODO: Implement create custom game logic here
            // For now, just navigate back to game page
            log::info!("Creating custom game - Identity: {}, Comment: {}", *identity_to_guess, *comment);
        })
    };

    html! {
        <div class="custom-game-design">
            <h1>{t("ui.custom_game_design")}</h1>
            <div class="design-container">
                <div class="input-group">
                    <label for="identity-to-guess">{t("custom.identity_label")}</label>
                    <input
                        type="text" 
                        value={(*identity_to_guess).clone()}
                        oninput={on_identity_input}
                        placeholder={t("custom.identity_placeholder")}
                    />
                </div>

                <div class="input-group">
                    <label for="comment">{t("custom.comment_label")}</label>
                    <textarea
                        rows="5"
                        value={(*comment).clone()}
                        oninput={on_comment_input}
                        placeholder={t("custom.comment_placeholder")}
                    />
                </div>

                <div class="button-group">
                    <button class="cancel-button" onclick={on_cancel}>
                        {t("custom.cancel_button")}
                    </button>
                    <button class="create-button" onclick={on_create}
                        disabled={identity_to_guess.is_empty()}>
                        {t("custom.create_button")}
                    </button>
                </div>
            </div>
            <div class="language-bar">
                <LanguageSelector on_language_changed={Some(on_language_changed)} />
            </div>
        </div>
    }
}