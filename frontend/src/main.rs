#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
mod c1;
mod chat;
mod ask_prompt_component;
mod com;

use std::time::Duration;
use gloo::net::http::{Request, Response};
use yew::prelude::*;
use yew_router::prelude::*;
use yew_router::hooks::use_navigator;
use log::info;
//use gloo_net::http::Request;
//use crate::c1::*;
//use crate::chat::*;
use crate::ask_prompt_component::*;
use gloo_storage::{LocalStorage, Storage};
use wasm_bindgen::JsValue;
use anyhow::{Context, Result};
use crate::com::*;

struct SoftState {
    n: UseStateHandle<i32>,
    error_message: UseStateHandle<String>,
}


#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/game")]
    Game,
    #[at("/error")]
    Error,
    #[not_found]
    #[at("/404")]
    NotFound,
}


#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub is_loading: bool,
    #[prop_or(AttrValue::Static("Bob"))]
    pub name: AttrValue,
}

#[function_component]
fn Hello(&Props { is_loading, ref name }: &Props) -> Html {
    if is_loading {
        html! { "Loading" }
    } else {
        html! { <>{"Hello "}{name}</> }
    }
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Home /> },
        Route::Game => html! { <Game /> },
        Route::Error => html! { <Error /> },
        Route::NotFound => html! { <h1>{ "404" }</h1> },
    }
}

#[function_component]
fn Home() -> Html {
    use_state(|| {});
    let navigator = use_navigator().unwrap();
    let onclick = Callback::from(move |_| wasm_bindgen_futures::spawn_local({
        let navigator = navigator.clone();
        async move {
            if let Ok(token) = fetch_text("/api/game/new").await {
                info!("res: {:?}", token);
                LocalStorage::set("token", &token).unwrap();
                navigator.push(&Route::Game);
            } else {
                navigator.push(&Route::Error);
            }
        }
    }));
    html! {
        <div>
            <h1>{ "Game" }</h1>
            <button {onclick}>{ "New game" }</button>
        </div>
    }
}

#[function_component]
fn Game() -> Html {
    let navigator = use_navigator().unwrap();
    let debug = use_state(|| { String::new() });
    let question_in_air = use_state(|| { false });

    let token: String = match LocalStorage::get("token").ok() {
        Some(token) => token,
        None => {
            navigator.push(&Route::Home);
            return html! {};
        }
    };

    {
        let debug = debug.clone();
        use_effect_with(token.clone(), move |token: &String| {
            let token = token.clone();
            let debug = debug.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match fetch_text(&format!("/api/game/{token}")).await {
                    Ok(res) => debug.set(res),
                    Err(e) => log::error!("fetch /api/game failed: {e:?}"),
                }
            });
            || ()
        });
    }

    let on_send = {
        let token = token.clone();
        let question_in_air = question_in_air.clone();
        Callback::from(move |text: String| {
            let token = token.clone();
            let question_in_air = question_in_air.clone();
            wasm_bindgen_futures::spawn_local(async move {
                question_in_air.set(true);
                if let Ok(res) = send_question(&token, &text).await {
                    info!("res: {:?}", res);
                }
                question_in_air.set(false);
            });
        })
    };

    html! {
        <>
        <h1>{ token }</h1>
        <AskPrompt prompt={"Make your guess..."}
            on_send={on_send}
            disabled={*question_in_air}
        />
        <hr/>
        <pre>{ debug.to_string() }</pre>
        <pre>{ question_in_air.to_string() }</pre>
        </>
    }
}

#[function_component]
fn App() -> Html {
    let counter = use_state(|| 0);
    info!("counter is: {:?}", counter);
        html! {
        <BrowserRouter>
            <Switch<Route> render={switch} /> // <- must be child of <BrowserRouter>
        </BrowserRouter>
    }
}

#[function_component]
fn Error() -> Html {
    html! {
        <h1>{ "Error" }</h1>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
