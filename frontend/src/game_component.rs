use gloo_storage::{LocalStorage, Storage};
use log::info;
use yew::{function_component, html, use_effect, use_effect_with, use_reducer, use_state, Callback, Html, Properties};
use crate::{board_component, use_navigator_expect, Route};
use crate::com::{fetch_pending, fetch_text, send_question};
use crate::ask_prompt_component::*;
use crate::board_component::*;
use wasm_bindgen_futures::spawn_local;

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

    /*
    {
        let board_props = board_props.clone();
        use_effect_with(token.clone(), move |token: &String| {
            let token = token.clone();
            let board_props = board_props.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let res = fetch_pending(token.as_str(), false).await.unwrap();
                board_props.set({
                    let mut next = (*board_props).clone();
                    next.pending = Some(res);
                    next
                });
            });
        });
    }


    {
        let debug = debug.clone();
        use_effect_with(token.clone(), move |token: &String| {
            let token = token.clone();
            let debug = debug.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match fetch_text(&format!("/api/game/{token}/version?wait=1")).await {
                    Ok(res) => debug.set(res),
                    Err(e) => {
                        navigator.push(&Route::Home);
                        log::error!("fetch /api/game failed: {e:?}");
                    },
                }
            });
            || ()
        });
    }


    let board_props = board_props.clone();
    let bb = board_props.clone();
    let on_send = {


        let token = token.clone();
        let question_in_air = question_in_air.clone();
        Callback::from(move |text: String| {
            let bb = board_props.clone();
            let token = token.clone();
            let question_in_air = question_in_air.clone();
            let board_props = board_props.clone();
            wasm_bindgen_futures::spawn_local(async move {
                question_in_air.set(true);
                if let Ok(res) = send_question(&token, &text).await {
                    info!("res: {:?}", res);
                }
                question_in_air.set(false);
                bb.set({
                    let mut next = (*board_props).clone();
                    next.pending = Some(true);
                    next
                });
            });
        })
    };


    let board_props = (*bb).clone();


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

//        <Board ..board_props />
        </>
    }

     */

    /*
    use_effect_with(token.clone(), move |token: &String| {
        board.dispatch(Act::SetPending(Some(false)));
    });
    */

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
                    match fetch_text(&format!("/api/game/{token}/version?wait=true")).await {
                        Ok(res) => {
                            info!("got response: {:?}", res);
                            let Ok(status)
                                = serde_json::from_str::<crate::game_response::Status>(&res) else {
                                info!("failed to parse status response: {:?}", res);
                                navigator.push(&Route::Home);
                                return;
                            };

                            info!("status: {}", status.status);
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