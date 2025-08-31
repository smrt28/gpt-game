use crate::to_html::ToHtmlEx;
use std::ops::Deref;
use gloo_storage::{LocalStorage, Storage};
use log::info;
use yew::{function_component, html, props, use_effect_with, use_state, Callback, Html, Properties};
use crate::{use_navigator_expect, Route};
use crate::com::{fetch_pending, fetch_text, send_question};
use crate::ask_prompt_component::*;
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use web_sys::console::info_1;
use yew::prelude::*;
use yew_router::hooks::use_navigator;
use shared::messages::GameState;
use crate::game_component::Game;

#[derive(Clone, PartialEq, Default)]
pub struct BoardState {
    game: GameState,
}

pub enum ServerErrorDetail {
    General
}

pub enum Act {
    ServerError(ServerErrorDetail),
    Update(GameState),
    InvalidGame,
}

impl Reducible for BoardState {
    type Action = Act;
    fn reduce(self: Rc<Self>, act: Act) -> Rc<Self> {
        info!("BoardState::reduce");
        match act {
            Act::ServerError(d) => {
                let res = (*self).clone();
                Rc::new(res)
            },
            Act::Update(next) => {
                info!("BoardState::reduce: update");

                let res = BoardState { game: next };
                Rc::new(res)
            }
            Act::InvalidGame => {
                info!("BoardState::reduce: invalid game");
                let mut g = GameState::default();
                g.game_ended = true;
                let res = BoardState {
                    game: g
                };
                Rc::new(res)
            }

        }
    }
}


#[derive(Properties, PartialEq)]
pub struct BoardProps {
    pub board: UseReducerHandle<BoardState>,
    pub on_new_game: Callback<()>,
}


#[function_component(Board)]
pub fn board(props: &BoardProps) -> Html {
    let game = &props.board.deref().game;

/*
    let navigator = use_navigator().unwrap();
    let onclick = Callback::from(move |_| spawn_local({
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
 */
    let onclick = {
        let cb = props.on_new_game.clone();
        Callback::from(move |_| cb.emit(()))
    };

    html! {
        <div class="board">
            { props.board.game.to_html() }
            { if props.board.game.game_ended {
                html! {
                  <div class="button-row">
                    <button class="new-game" {onclick}>{ "New game" }</button>
                  </div>
                }
            } else {
                html! {
                <div class="note">
                  { "Type: \"I'M LOSER\" and I’ll reveal my identity and pick a new one." }
                </div>
                }
            }}
        </div>
    }

}