use std::ops::Deref;
use std::rc::Rc;

use log::info;
use yew::{function_component, html, Callback, Html, Properties, Reducible, UseReducerHandle};

use crate::locale::t;
use crate::to_html::{ToHtmlEx, ToHtmlExArgs};
use shared::messages::GameState;
#[derive(Clone, PartialEq, Default)]
pub struct BoardState {
    game: Option<GameState>,
}

#[allow(dead_code)]
pub enum ServerErrorDetail {
    General
}

#[allow(dead_code)]
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
            Act::ServerError(_d) => {
                Rc::new((*self).clone())
            },
            Act::Update(next) => {
                Rc::new(BoardState { game: Some(next) })
            }
            Act::InvalidGame => {
                Rc::new(BoardState {
                    game: None
                })
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
    let _game = &props.board.deref().game;

    let onclick = {
        let cb = props.on_new_game.clone();
        Callback::from(move |_| cb.emit(()))
    };

    let mut game_ended = false;


    let game_board_html = if let Some(game_board) = &props.board.game {
        if game_board.game_ended {
            game_ended = true;
        }
        let args = ToHtmlExArgs{
            state: &game_board
        };
        game_board.to_html(&args)
    } else {
        game_ended = true;
        html! {}
    };

    html! {
        <div class="board">

            { game_board_html }

            { if game_ended {
                html! {
                  <div class="button-row">
                    <button class="new-game" {onclick}>{ t("ui.new_game") }</button>

                    <button class="new-game">{ t("ui.new_custom_game") }</button>
                  </div>
                }
            } else {
                html! {

                }
            }}
        </div>
    }

}