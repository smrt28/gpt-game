use gloo_storage::{LocalStorage, Storage};
use log::info;
use yew::{function_component, html, use_effect, use_effect_with, use_reducer, use_state, Callback, Html, Properties};
use crate::{board_component, use_navigator_expect, Route};
use crate::com::{fetch_pending, fetch_text, send_question};
use crate::ask_prompt_component::*;
use crate::board_component::*;
use wasm_bindgen_futures::spawn_local;
use shared::messages::{response_to_content, GameState};

#[function_component]
pub fn Game() -> Html {


    //props!{board_component::Props}
    let navigator = use_navigator_expect();
    let debug = use_state(|| { String::new() });
    //let board_props = use_state(|| board_component::Props {
    //    pending: None
    //});
    let question_in_air = use_state(|| { false });

    let token: String = match LocalStorage::get("token").ok() {
        Some(token) => token,
        None => {
            navigator.push(&Route::Home);
            return html! {};
        }
    };


    let board = use_reducer(BoardState::default);

    let on_send = {
        let token = token.clone();
        let board = board.clone();
        let debug = debug.clone();
        Callback::from(move |text: String| {
            let board = board.clone();
            let token = token.clone();
            let debug = debug.clone();
            spawn_local(async move {
                board.dispatch(Act::SetPending(Some(true)));
                let res = send_question(&token, &text).await.unwrap();
                debug.set(res);
            });
        })
    };

    {
        let debug = debug.clone();
        let token = token.clone();
        let board = board.clone();
        use_effect(move || {
            spawn_local(async move {
                let mut pooling = true;
                while pooling {
                    pooling = false;
                    match fetch_text(&format!("/api/game/{token}?wait=true")).await {
                        Ok(res) => {
                            info!("got response: {:?}", res);
                            let Ok(status)
                                = serde_json::from_str::<crate::game_response::Status>(&res) else {
                                info!("failed to parse status response: {:?}", res);
                                navigator.push(&Route::Home);
                                return;
                            };

                            info!("status: {}", status.status);
                            if status.status == "error" {
                                navigator.push(&Route::Home);
                                return;
                            }



                            if let Err(e) = response_to_content::<GameState>(&res) {
                                info!("error: {:?}", e);
                            }

                            /*
                            let Ok(value) = serde_json::from_str::<GameState>(&res) else {
                                info!("failed to parse game response: {:?}", res);
                                navigator.push(&Route::Home);
                                return;
                            };

                            board.dispatch(Act::Update(value));

                             */

                            if status.should_repeat() {
                                pooling = true;
                            }
                        },
                        Err(e) => {
                            navigator.push(&Route::Home);
                            log::error!("fetch /api/game failed: {e:?}");
                        },
                    }
                }


                /*
                match fetch_text(&format!("/api/game/{token}")).await {
                    Ok(res) => {
                        info!("res: {:?}", res);
                    }
                    Err(e) => {}
                }
               */


            });
            || ()
        });
    }

    
    html! {
        <>
        <Board board={board.clone()}/>
        <AskPrompt prompt={"Make your guess..."}
            on_send={on_send}
        />
        <pre>{"token:"}{token.to_string()}</pre>
        <pre>{"Debug:"}{debug.to_string()}</pre>
        </>
    }


}


/*

use std::rc::Rc;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
struct BoardState { pending: Option<bool> }

enum Act { SetPending(Option<bool>) }

impl Reducible for BoardState {
    type Action = Act;
    fn reduce(self: Rc<Self>, act: Act) -> Rc<Self> {
        match act {
            Act::SetPending(p) => Rc::new(BoardState { pending: p }),
        }
    }
}

let board = use_reducer(|| BoardState { pending: None });

// update (cheap)
board.dispatch(Act::SetPending(Some(false)));

// read
html! { <Board pending={board.pending} /> } // or clone if needed


 */