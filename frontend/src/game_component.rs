use std::cell::Cell;
use std::rc::Rc;
use gloo_storage::{LocalStorage, Storage};
use log::info;
use yew::{function_component, html, use_effect_with, use_reducer, use_state, Callback, Html};
use crate::Route;
use yew_router::hooks::use_navigator;
use crate::server_query::{fetch_new_game_token, fetch_text, send_question};
use crate::ask_prompt_component::AskPrompt;
use crate::board_component::{Act, Board, BoardState};
use crate::locale::{t, get_current_language};
use crate::language_logic::{should_show_language_dialog, switch_language};
use crate::language_selector_component::LanguageSelector;
use shared::locale::Language;
use wasm_bindgen_futures::spawn_local;
use shared::messages::{GameState, ServerResponse, Status};
use shared::gpt::check_question;


// http://localhost:3000/run/game

#[function_component]
pub fn Game() -> Html {
    info!("Game component loaded");
    let navigator = use_navigator().expect("Must be used within a Router");
    let version = use_state(|| 0);
    let pending = use_state(|| false);
    let active_game = use_state(|| false);
    let show_instructions = use_state(|| true);
    let show_language_dialog = use_state(|| false);
    let pending_language = use_state(|| None::<Language>);
    let language_version = use_state(|| 0);
    let board = use_reducer(BoardState::default);

    let (token, has_token) = match LocalStorage::get::<String>("token") {
        Ok(token) => (token, true),
        _ => (String::new(), false),
    };

    let on_send = {
        let token = token.clone();
        let version = version.clone();
        let pending = pending.clone();
        Callback::from(move |text: String| {
            if !check_question(&text).is_ok() {
                return
            }
            let (token, version, pending) = (token.clone(), version.clone(), pending.clone());
            spawn_local(async move {
                pending.set(true);

                if let Err(e) = send_question(&token, &text).await {
                    info!("Error sending question: {:?}", e);
                    return;
                }
                version.set(*version + 1);
            });
        })
    };

    use_effect_with(*version, {
        let token = token.clone();
        let board = board.clone();
        let _navigator = navigator.clone();
        let pending = pending.clone();
        let active_game = active_game.clone();

        move |_: &i32| {
            let cancelled = Rc::new(Cell::new(false));
            let cancel_for_task = cancelled.clone();
            let cancel_for_cleanup = cancelled.clone();
            
            spawn_local(async move {
                let _guard = scopeguard::guard((), |_| {
                    pending.set(false);
                });

                if !has_token {
                    info!("Missing token, cannot poll game state");
                    active_game.set(false);
                    return;
                }

                let mut quiet = 0;
                let mut wait = 0;
                
                loop {
                    if cancel_for_task.get() { 
                        break; 
                    }
                    
                    let url = format!("/api/game/{token}?wait={wait}&quiet={quiet}");
                    match fetch_text(&url).await {
                        Ok(res) => {
                            match ServerResponse::<GameState>::from_response(&res) {
                                Ok(server_response) => {
                                    match server_response.status {
                                        Status::Ok => {
                                            let mut active = true;
                                            if let Some(content) = server_response.content {
                                                active = !content.game_ended;
                                                board.dispatch(Act::Update(content));
                                            }
                                            active_game.set(active);
                                            break;
                                        }
                                        Status::Error => {
                                            log::error!("Server returned error");
                                            active_game.set(false);
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
                                Err(_) => {
                                    log::error!("Failed to parse server response");
                                    active_game.set(false);
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to fetch game state: {e:?}");
                            active_game.set(false);
                            board.dispatch(Act::InvalidGame);
                            break;
                        }
                    }
                    wait = 1;
                }
            });

            move || cancel_for_cleanup.set(true)
        }
    });

    let on_new_game = {
        let version = version.clone();
        let navigator = navigator.clone();
        Callback::from(move |_| {
            let (version, navigator) = (version.clone(), navigator.clone());
            spawn_local(async move {
                info!("Creating new game, lang={}", get_current_language().to_code());
                match fetch_new_game_token().await {
                    Ok(new_token) => {
                        info!("Created new game with token: {new_token}");
                        if LocalStorage::set("token", &new_token).is_ok() {
                            version.set(*version + 1);
                            navigator.push(&Route::Game);
                        } else {
                            log::error!("Failed to save token to localStorage");
                            navigator.push(&Route::Error);
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to create new game: {e:?}");
                        navigator.push(&Route::Error);
                    }
                }
            });
        })
    };

    let toggle_instructions = {
        let show_instructions = show_instructions.clone();
        Callback::from(move |_| {
            show_instructions.set(!*show_instructions);
        })
    };

    let on_language_change = {
        let show_language_dialog = show_language_dialog.clone();
        let pending_language = pending_language.clone();
        let language_version = language_version.clone();
        let active_game = active_game.clone();
        
        Callback::from(move |lang: Language| {
            if should_show_language_dialog(lang.clone(), *active_game) {
                pending_language.set(Some(lang));
                show_language_dialog.set(true);
            } else {
                // Switch immediately if no active game
                switch_language(lang);
                language_version.set(*language_version + 1);
            }
        })
    };

    let on_confirm_language_change = {
        let show_language_dialog = show_language_dialog.clone();
        let pending_language = pending_language.clone();
        let language_version = language_version.clone();
        let board = board.clone();
        let active_game = active_game.clone();
        
        Callback::from(move |confirmed: bool| {
            show_language_dialog.set(false);
            
            if confirmed {
                if let Some(lang) = (*pending_language).clone() {
                    switch_language(lang);
                    language_version.set(*language_version + 1);
                    // Clear the token and invalidate the game state
                    let _ = LocalStorage::delete("token");
                    board.dispatch(Act::InvalidGame);
                    active_game.set(false);
                }
            }
            
            pending_language.set(None);
        })
    };


    html! {
        <>


            <h1>{t("ui.game_header")}</h1>

            <Board board={board.clone()} on_new_game={on_new_game} />

            if *active_game {
                <AskPrompt 
                    prompt={t("game.prompt")}
                    on_send={on_send}
                    disabled={*pending}
                    token={Some(token.clone())}
                />
            }

            <div class="instructions-container">
                <button class="instructions-toggle" onclick={toggle_instructions}>
                    <span class="toggle-icon">
                        {if *show_instructions { "▼" } else { "▶" }}
                    </span>
                    {t("game.instructions_toggle")}
                </button>
                
                if *show_instructions {
                    <div class="instructions">
                        <ul>
                            <li>{t("game.rule1")}</li>
                            <li>{t("game.rule2")}</li>
                            <li>{t("game.rule3")}</li>

                        </ul>
                        <div class="game-bar">
            <LanguageSelector
                on_language_change={on_language_change}
                pending_language={(*pending_language).clone()}
                show_dialog={*show_language_dialog}
                on_confirm_change={on_confirm_language_change}
            />
            </div>
                    </div>
                }
            </div>
        </>
    }
}