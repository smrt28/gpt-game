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
    let your_name = use_state(|| String::new());
    let identity_to_guess = use_state(|| String::new());

    let on_language_changed = {
        let language_render_trigger = language_render_trigger.clone();
        Callback::from(move |_| {
            language_render_trigger.set(*language_render_trigger + 1);
        })
    };

    let on_name_change = {
        let your_name = your_name.clone();
        Callback::from(move |e: web_sys::Event| {
            let input: HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
            your_name.set(input.value());
        })
    };

    let on_identity_change = {
        let identity_to_guess = identity_to_guess.clone();
        Callback::from(move |e: web_sys::Event| {
            let textarea: HtmlTextAreaElement = e.target().unwrap().dyn_into().unwrap();
            identity_to_guess.set(textarea.value());
        })
    };

    let on_cancel = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::Game);
        })
    };

    let on_create = {
        let your_name = your_name.clone();
        let identity_to_guess = identity_to_guess.clone();
        let navigator = navigator.clone();
        Callback::from(move |_| {
            // TODO: Implement create custom game logic here
            // For now, just navigate back to game page
            log::info!("Creating custom game - Name: {}, Identity: {}", *your_name, *identity_to_guess);
            navigator.push(&Route::Game);
        })
    };

    html! {
        <div class="custom-game-design">
            <h1>{t("ui.custom_game_design")}</h1>
            <div class="design-container">
                <div class="input-group">
                    <label for="your-name">{t("custom.your_name_label")}</label>
                    <input
                        type="text" 
                        value={(*your_name).clone()}
                        onchange={on_name_change}
                        placeholder={t("custom.identity_placeholder")}
                    />
                </div>
                
                <div class="input-group">
                    <label for="identity-to-guess">{t("custom.identity_label")}</label>
                    <textarea
                        rows="4"
                        value={(*identity_to_guess).clone()}
                        onchange={on_identity_change}
                        placeholder={t("custom.your_name_placeholder")}
                    />
                </div>
                
                <div class="button-group">
                    <button class="cancel-button" onclick={on_cancel}>
                        {t("custom.cancel_button")}
                    </button>
                    <button class="create-button" onclick={on_create} disabled={your_name.is_empty() || identity_to_guess.is_empty()}>
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