#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

mod ask_prompt_component;
mod com;
mod www_error;
mod game_component;
mod board_component;
mod game_response;
mod to_html;

use std::future::pending;
use std::time::Duration;
use gloo::net::http::{Request, Response};
use yew::prelude::*;
use yew_router::prelude::*;
use yew_router::hooks::use_navigator;
use log::info;
use gloo_storage::{LocalStorage, Storage};
use wasm_bindgen::JsValue;
use anyhow::{Context, Result};
use log::kv::Source;
use crate::com::*;
use crate::game_component::*;

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

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Home /> },
        Route::Game => html! { 
            <Game />
        },
        Route::Error => html! { <Error /> },
        Route::NotFound => html! { <h1>{ "404" }</h1> },
    }
}

#[function_component]
fn Home() -> Html {
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


#[hook]
fn use_navigator_expect() -> Navigator {
    use_navigator().expect("use_navigator_expect: must be rendered inside a <Router>")
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