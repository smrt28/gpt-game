use crate::to_html::ToHtmlEx;
use std::ops::Deref;
use gloo_storage::{LocalStorage, Storage};
use log::info;
use yew::{function_component, html, props, use_effect_with, use_state, Callback, Html, Properties};
use crate::{use_navigator_expect, Route};
use crate::com::{fetch_pending, fetch_text, send_question};
use crate::ask_prompt_component::*;
use std::rc::Rc;
use web_sys::console::info_1;
use yew::prelude::*;
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
        }
    }
}


#[derive(Properties, PartialEq)]
pub struct BoardProps {
    pub board: UseReducerHandle<BoardState>,
}


#[function_component(Board)]
pub fn board(props: &BoardProps) -> Html {
    let game = &props.board.deref().game;

    html! {
        <div class="board">
            { game.to_html() }
        </div>
    }
}


/*

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    #[prop_or_default]
    pub pending: Option<bool>,
}

#[function_component]
pub fn Board(props: &Props) -> Html {
    html! {
        <div>
        <b>{"Debug:"}</b>
        <pre>{"pending: "} {props.pending}</pre>
        </div>
    }
}

*/