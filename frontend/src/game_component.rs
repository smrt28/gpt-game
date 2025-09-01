use std::cell::Cell;
use std::rc::Rc;
use std::slice::SliceIndex;
use gloo_storage::{LocalStorage, Storage};
use log::info;
use yew::{function_component, html, use_effect, use_effect_with, use_reducer, use_state, Callback, Html, Properties};
use crate::{board_component, use_navigator_expect, Route};
use crate::com::{fetch_pending, fetch_text, send_question};
use crate::ask_prompt_component::*;
use crate::board_component::*;
use wasm_bindgen_futures::spawn_local;
use shared::messages::{GameState, ServerResponse, Status};


// http://localhost:3000/run/game

#[function_component]
pub fn Game() -> Html {
    info!("Game");
    let navigator = use_navigator_expect();
    let version = use_state(|| { 0 });
    let pending = use_state(|| { false });
    let active_game = use_state(|| { false });

    let active_game_on_load = active_game.clone();
    let active_game_render = active_game.clone();

    let mut try_token = true;

    let token: String = match LocalStorage::get("token") {
        Ok(Some(token)) => token,
        _ => {
            try_token = false;
            "".to_string()
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
                if let Err(e) = send_question(&token, &text).await {
                    info!("error: {:?}", e);
                    return;
                };
                version.set(1 + *version);
            });
        })
    };

    use_effect_with(*version, {
        let token = token.clone();
        let board = board.clone();
        let navigator = navigator.clone();
        let pending = pending.clone();
        let ver = version.clone();

        move |curr_ver: &i32| {
            let curr = *curr_ver;
            let cancelled = Rc::new(Cell::new(false));
            let cancel_for_task = cancelled.clone();
            let cancel_for_cleanup = cancelled.clone();
            spawn_local(async move {
                let guard = scopeguard::guard((), |_| {
                    pending.set(false);
                });

                let mut quiet = 0;
                let mut wait = 0;
                let mut error = 0;
                loop {
                    if !try_token {
                        error = 10;
                        info!("missing token");
                        break;
                    }
                    if cancel_for_task.get() { break; }
                    info!("polling; iteration={}", curr);
                    match fetch_text(&format!("/api/game/{token}?wait={wait}&quiet={quiet}")).await {
                        Ok(res) => {
                            let game_state = ServerResponse::<GameState>::from_response(res.as_str());

                            match game_state {
                                Ok(server_response) => {
                                    match server_response.status {
                                        Status::Ok => {
                                            if let Some(content) = server_response.content {
                                                board.dispatch(Act::Update(content));
                                            }
                                            break;
                                        }
                                        Status::Error => {
                                            error = 1;
                                            break;
                                        }
                                        Status::Pending => {
                                            if quiet == 0 {
                                                quiet = 1;
                                                if let Some(content) = server_response.content {
                                                    board.dispatch(Act::Update(content));
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    error = 2;
                                    break;
                                }
                            }
                        },
                        Err(e) => {
                            error = 3;
                            log::error!("fetch /api/game failed: {e:?}");
                            break;
                        },
                    }
                    wait = 1;
                } // loop

                if error != 0 {
                    active_game_on_load.set(false);
                    log::info!("error polling: {}", error);
                    navigator.push(&Route::Game);
                    board.dispatch(Act::InvalidGame);
                } else {
                    active_game_on_load.set(true);
                }
            });

            move || cancel_for_cleanup.set(true)
        }
    });

    let navigator = navigator.clone();
    let v2 = version.clone();
    let on_new_game = {
        let v2 = v2.clone();
        Callback::from(move |_| {
            let v2 = v2.clone();
            let navigator = navigator.clone();
            spawn_local(async move {
                match fetch_text("/api/game/new").await {
                    Ok(token) => {
                        v2.set(*v2 + 1);
                        info!("new token: {token}");
                        LocalStorage::set("token", &token).ok();
                        navigator.push(&Route::Game);
                    }
                    Err(_) => navigator.push(&Route::Error),
                }
            });
        })
    };

    html! {
        <>
        <h1>{ "Guess Who" }</h1>

        <Board board={board.clone()}
            on_new_game={on_new_game}/>

        if *active_game_render {
            <AskPrompt prompt={"I'm someone or something, guess who I'm. Your question is:"}
                on_send={on_send}
                disabled={*pending}
                token={Some(token.clone())}
            />
        }
        </>
    }


}