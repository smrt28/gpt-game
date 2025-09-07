use log::info;
//use log::info;
use yew::{function_component, html, Html, use_state, Callback, use_node_ref};
use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use wasm_bindgen::JsCast;
use yew_router::hooks::use_navigator;
use shared::messages::{CustomGameInfo, GameTemplate, GameTemplateStatus, MAX_IDENTITY_STRING_LEN};
use crate::locale::{get_current_language, t, t_shared};
use crate::language_selector_component::LanguageSelector;
use crate::Route;

#[function_component(CustomGameDesign)]
pub fn custom_game_design() -> Html {
    let navigator = use_navigator().expect("Must be used within a Router");
    let language_render_trigger = use_state(|| 0u32);
    let comment = use_state(|| String::new());
    let form_status = use_state(|| GameTemplateStatus::NotSet);

    let comment_ref = use_node_ref();
    let identity_ref = use_node_ref();

    let on_language_changed = {
        let language_render_trigger = language_render_trigger.clone();
        Callback::from(move |_| {
            language_render_trigger.set(*language_render_trigger + 1);
        })
    };

    let on_identity_input = {
        let form_status = form_status.clone();
        Callback::from(move |e: web_sys::InputEvent| {
            if let Some(target) = e.target() {
                if let Ok(input) = target.dyn_into::<HtmlInputElement>() {
                    match input.value().len() {
                        n if n > MAX_IDENTITY_STRING_LEN => {
                            form_status.set(GameTemplateStatus::ToLongIdentity);
                        },
                        0 => {
                            form_status.set(GameTemplateStatus::EmptyIdentity);
                        },
                        _ => {
                            form_status.set(GameTemplateStatus::Ok);
                        }
                    }
                }
            }
        })
    };

    let on_comment_input = {
        Callback::from(move |e: web_sys::InputEvent| {
            if let Some(target) = e.target() {
                if let Ok(textarea) = target.dyn_into::<HtmlTextAreaElement>() {
                    //comment.set(textarea.value());
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
        let identity_ref = identity_ref.clone();
        let comment_ref = comment_ref.clone();
        let form_status = form_status.clone();
        //let navigator = navigator.clone();
        Callback::from(move |_| {
            let identity_str: String;
            let mut comment_str: String = String::new();

            if let Some(identity_ref) = identity_ref.cast::<HtmlTextAreaElement>() {
                identity_str = identity_ref.value().clone();
                info!("identity: {}", identity_ref.value());
            } else {
                return;
            }

            if let Some(comment_ref) = comment_ref.cast::<HtmlTextAreaElement>() {
                comment_str = comment_ref.value().clone();
            }

            let game_template = GameTemplate {
                identity: identity_str,
                language: get_current_language(),
                properties: CustomGameInfo {
                    comment: Some(comment_str),
                }
            };

            // The checks are handled in oninput callbacks. It should not
            // be possible to push the "Create" button if the form is not valid.
            // And even if it happens, server side does the same checks.

            log::info!("Creating custom game - Identity: {}, Comment: {}",
                identity_str, comment_str);
        })
    };



    html! {

        <div class="custom-game-design">
            <h1>{t("ui.custom_game_design")}</h1>
            <div class="design-container">
                {t_shared(&(*form_status))}
                <div class="input-group">
                    <label for="identity-to-guess">{t("custom.identity_label")}</label>
                    <input
                        maxlength={MAX_IDENTITY_STRING_LEN.to_string()}
                        type="text" 
                        //value={(*identity_to_guess).clone()}
                        oninput={on_identity_input}
                        placeholder={t("custom.identity_placeholder")}
                        ref={identity_ref}
                    />
                </div>

                <div class="input-group">
                    <label for="comment">{t("custom.comment_label")}</label>
                    <textarea
                        rows="5"
                        value={(*comment).clone()}
                        oninput={on_comment_input}
                        placeholder={t("custom.comment_placeholder")}
                        ref={comment_ref}
                    />
                </div>

                <div class="button-group">
                    <button class="cancel-button" onclick={on_cancel}>
                        {t("custom.cancel_button")}
                    </button>
                    <button class="create-button" onclick={on_create}
                        disabled={*form_status != GameTemplateStatus::Ok}>
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