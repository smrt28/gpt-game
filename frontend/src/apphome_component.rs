use gloo_storage::{LocalStorage, Storage};
use yew::{function_component, html, use_state, Callback, Html};
use crate::language_selector_component::LanguageSelector;
use crate::locale::{get_current_language, t};
use log::info;
use wasm_bindgen_futures::spawn_local;
use crate::server_query::fetch_new_game_token;
use yew_router::hooks::use_navigator;
use crate::Route;

#[function_component(AppHome)]
pub fn app_home() -> Html {
    let navigator = use_navigator().expect("Must be used within a Router");
    let render_trigger = use_state(|| 0);

    // Helper function to create new game and navigate
    let create_game_and_navigate = |route: Route| {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            let navigator = navigator.clone();
            let route = route.clone();
            spawn_local(async move {
                info!("Creating new game, lang={}", get_current_language().to_code());
                match fetch_new_game_token().await {
                    Ok(new_token) => {
                        if LocalStorage::set("token", &new_token).is_ok() {
                            navigator.push(&route);
                        } else {
                            log::error!("Failed to save token to localStorage");
                            navigator.push(&Route::Error);
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to create new game: {e:?}");
                        navigator.push(&Route::Error);
                    }
                }
            });
        })
    };

    let on_new_game = create_game_and_navigate(Route::Game);
    let on_new_custom_game = create_game_and_navigate(Route::CustomGameDesign);

    let on_language_changed = {
        let render_trigger = render_trigger.clone();
        Callback::from(move |_| {
            render_trigger.set(*render_trigger + 1);
        })
    };

    html! {
        <>
            <h1>{ t("ui.page_title") }</h1>
            <button class="new-game" onclick={on_new_game}>{ t("ui.new_game") }</button>
            <button class="new-game" onclick={on_new_custom_game}>{ t("ui.new_custom_game") }</button>
            <div class="language-bar">
                <LanguageSelector on_language_changed={Some(on_language_changed)} />
            </div>
        </>
    }
}