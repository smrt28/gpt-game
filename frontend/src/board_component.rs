use std::ops::Deref;
use gloo_storage::{LocalStorage, Storage};
use log::info;
use yew::{function_component, html, use_effect_with, use_state, Callback, Html, Properties};
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
    pending: Option<bool>,
    game: GameState,
}

pub enum ServerErrorDetail {
    General
}

pub enum Act {
    SetPending(Option<bool>),
    ServerError(ServerErrorDetail),
    Update(GameState),
}

impl Reducible for BoardState {
    type Action = Act;
    fn reduce(self: Rc<Self>, act: Act) -> Rc<Self> {
        info!("BoardState::reduce");
        match act {
            Act::SetPending(p) => Rc::new(BoardState {
                pending: p,
                game: self.game.clone(),
            }),
            Act::ServerError(d) => {
                let mut res = (*self).clone();
                res.pending = Some(false);
                Rc::new(res)
            },
            Act::Update(g) => {
                info!("BoardState::reduce: update");
                let mut res = (*self).clone();
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
    let state = match props.board.pending {
        None => "N/A",
        Some(true) => "Pending",
        Some(false) => "Done"
    };

    html! {
        <pre>{"BOARD>: "}{state}</pre>
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