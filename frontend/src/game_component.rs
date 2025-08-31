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

#[function_component]
pub fn Game() -> Html {
    let navigator = use_navigator_expect();
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
                if curr > 0 {
                    let guard = scopeguard::guard((), |_| {
                        pending.set(false);
                    });

                    loop {
                        if cancel_for_task.get() { break; }
                        info!("polling...{}", curr);
                        match fetch_text(&format!("/api/game/{token}?wait=1&quiet=1")).await {
                            Ok(res) => {
                                info!("got response: {:?}", res);
                                match ServerResponse::<GameState>::from_response(res.as_str()) {
                                    Ok(server_response) => {
                                        match server_response.status {
                                            Status::Ok => {
                                                break;
                                            }
                                            Status::Error => { break; }
                                            Status::Pending => {}
                                        }
                                    }
                                    Err(e) => { break; }
                                }
                            },
                            Err(e) => {
                                log::error!("fetch /api/game failed: {e:?}");
                                break;
                            },
                        }
                    }
                } else {
                    info!("initial fetch");
                }

                if cancel_for_task.get() {
                    return;
                }

                match fetch_text(&format!("/api/game/{token}")).await {
                    Ok(text) => {
                        if let Ok(resp) = ServerResponse::<GameState>::from_response(text.as_str()) {
                            if resp.status == Status::Error {
                                navigator.push(&Route::Home);
                            }
                            if resp.status == Status::Pending {
                                pending.set(true);
                                ver.set(1 + curr);
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

            move || cancel_for_cleanup.set(true)
        }
    });


    html! {
        <>
        <Board board={board.clone()}/>
        <hr/>
        <AskPrompt prompt={"Make your guess..."}
            on_send={on_send}
            disabled={*pending}
        />
        <pre>{"token:"}{token.to_string()}</pre>
        </>
    }


}