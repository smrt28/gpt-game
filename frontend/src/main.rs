mod ask_prompt_component;
mod board_component;
mod game_component;
mod locale;
mod server_query;
mod to_html;

use gloo_storage::{LocalStorage, Storage};
use log::info;
use yew::prelude::*;
use yew_router::prelude::*;
use yew_router::hooks::use_navigator;

use crate::game_component::Game;
use crate::server_query::fetch_text;


#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/game")]
    Game,
    #[at("/error")]
    Error,
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(route: Route) -> Html {
    info!("Switching to route");
    match route {
        Route::Game => html! { <Game /> },
        Route::Error => html! { <Error /> },
        Route::NotFound => html! { <h1>{ "404" }</h1> },
    }
}

#[function_component]
fn Home() -> Html {
    let navigator = use_navigator().expect("Must be used within a Router");
    let onclick = Callback::from(move |_| {
        let navigator = navigator.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match fetch_text("/api/game/new").await {
                Ok(token) => {
                    info!("Got token: {:?}", token);
                    if LocalStorage::set("token", &token).is_ok() {
                        navigator.push(&Route::Game);
                    } else {
                        navigator.push(&Route::Error);
                    }
                }
                Err(_) => {
                    navigator.push(&Route::Error);
                }
            }
        });
    });
    
    html! {
        <div>
            <h1>{ "Game" }</h1>
            <button {onclick}>{ "New game" }</button>
        </div>
    }
}




#[function_component]
fn App() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

#[function_component]
fn Error() -> Html {
    html! {
        <h1>{ "Server Error" }</h1>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}