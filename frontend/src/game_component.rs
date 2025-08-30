use std::slice::SliceIndex;
use gloo_storage::{LocalStorage, Storage};
use log::info;
use yew::{function_component, html, use_effect, use_effect_with, use_reducer, use_state, Callback, Html, Properties};
use crate::{board_component, use_navigator_expect, Route};
use crate::com::{fetch_pending, fetch_text, send_question};
use crate::ask_prompt_component::*;
use crate::board_component::*;
use wasm_bindgen_futures::spawn_local;
use shared::messages::{response_to_content, GameState, ServerResponse, Status};

#[function_component]
pub fn Game() -> Html {
    let navigator = use_navigator_expect();
    let debug = use_state(|| { String::new() });
    let version = use_state(|| { 0 });
    let pending = use_state(|| { false });


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
        let version = version.clone();
        let pending = pending.clone();
        Callback::from(move |text: String| {
            let version = version.clone();
            let token = token.clone();
            let pending = pending.clone();
            spawn_local(async move {
                pending.set(true);
                let _guard = scopeguard::guard((), |_| {
                    pending.set(false);
                    version.set(1 + *version);
                });

                if let Err(e) = send_question(&token, &text).await {
                    info!("error: {:?}", e);
                    return;
                };

                loop {
                    match fetch_text(&format!("/api/game/{token}?wait=1&quiet=1")).await {
                        Ok(res) => {
                            info!("got response: {:?}", res);
                            match ServerResponse::<GameState>::from_response(res.as_str()) {
                                Ok(server_response) => {
                                    match server_response.status {
                                        Status::Ok => { return; }
                                        Status::Error => { return; }
                                        Status::Pending => {}
                                    }
                                }
                                Err(e) => { return; }
                            }
                        },
                        Err(e) => {
                            log::error!("fetch /api/game failed: {e:?}");
                            return;
                        },
                    }
                }
            });
        })
    };

    use_effect_with(version, {
        let token = token.clone();
        let board = board.clone();
        let navigator = navigator.clone();
        move |_| {
            spawn_local(async move {
                match fetch_text(&format!("/api/game/{token}")).await {
                    Ok(text) => {
                        if let Ok(resp) = ServerResponse::<GameState>::from_response(text.as_str()) {
                            if resp.status == Status::Error {
                                navigator.push(&Route::Home);
                            }
                            if let Some(content) = resp.content {
                                board.dispatch(Act::Update(content));
                            }
                        } else {
                            info!("failed to parse game response");
                        }
                    }
                    Err(e) => info!("fetch error: {e:?}"),
                }
                info!("exiting polling loop");
            });
            || ()
        }
    });


    html! {
        <>
        <Board board={board.clone()}/>
        <AskPrompt prompt={"Make your guess..."}
            on_send={on_send}
            disabled={*pending}
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