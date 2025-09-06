mod ask_prompt_component;
mod board_component;
mod custom_game_design_component;
mod game_component;
mod language_logic;
mod language_selector_component;
mod locale;
mod server_query;
mod to_html;
mod apphome_component;

use log::info;
use yew::prelude::*;
use yew_router::prelude::*;
use crate::game_component::Game;
use crate::custom_game_design_component::CustomGameDesign;
//use crate::Route::Home;
//use crate::server_query::fetch_text;


#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/home")]
    AppHome,
    #[at("/game")]
    Game,
    #[at("/custom-game")]
    CustomGameDesign,
    #[at("/error")]
    Error,
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(route: Route) -> Html {
    info!("Switching to route");
    match route {
        Route::AppHome => html! { <AppHome /> },
        Route::Game => html! { <Game /> },
        Route::CustomGameDesign => html! { <CustomGameDesign /> },
        Route::Error => html! { <Error /> },
        Route::NotFound => html! { <h1>{ "404" }</h1> },
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

#[function_component]
fn AppHome() -> Html {
    html! {
        <h1>{ "Home" }</h1>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}