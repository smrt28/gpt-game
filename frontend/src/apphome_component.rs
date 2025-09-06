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
    let navigate = use_navigator().expect("Must be used within a Router");
    let navigate2 = navigate.clone();

    let render_trigger = use_state(|| 0);
    let on_new_game = {
        Callback::from(move |_| {
            let new_game_navigate = navigate.clone();
            spawn_local(async move {
                info!("home-new-game {}!", get_current_language().to_code());
                match fetch_new_game_token().await {
                    Ok(new_token) => {
                        LocalStorage::set("token", &new_token).and_then(|()| {
                            new_game_navigate.push(&Route::Game);
                            Ok(())
                        }).ok();
                    }
                    Err(_e) => {

                    }
                }
            });
        })
    };


    let on_new_custom_game = {
        Callback::from(move |_| {
            let new_game_navigate = navigate2.clone();
            spawn_local(async move {
                info!("home-new-game {}!", get_current_language().to_code());
                match fetch_new_game_token().await {
                    Ok(new_token) => {
                        LocalStorage::set("token", &new_token).and_then(|()| {
                            new_game_navigate.push(&Route::CustomGameDesign);
                            Ok(())
                        }).ok();
                    }
                    Err(_e) => {

                    }
                }
            });
        })
    };



    let on_language_changed = {
        let render_trigger = render_trigger.clone();
        Callback::from(move |_| {
            render_trigger.set(*render_trigger + 1);
            info!("language-changed");
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