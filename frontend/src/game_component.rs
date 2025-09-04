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
use crate::locale::{t, tf, get_current_language, set_language};
use shared::locale::Language;
use wasm_bindgen_futures::spawn_local;
use shared::messages::{GameState, ServerResponse, Status};


// http://localhost:3000/run/game

#[function_component]
pub fn Game() -> Html {
    info!("Game component loaded");
    let navigator = use_navigator().expect("Must be used within a Router");
    let version = use_state(|| 0);
    let pending = use_state(|| false);
    let active_game = use_state(|| false);
    let show_instructions = use_state(|| true);
    let language_version = use_state(|| 0); // For triggering re-renders when language changes
    let show_language_dialog = use_state(|| false);
    let pending_language = use_state(|| None::<Language>);
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
        let navigator = navigator.clone();
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
                                            if let Some(content) = server_response.content {
                                                board.dispatch(Act::Update(content));
                                            }
                                            active_game.set(true);
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
        let language_version = language_version.clone();
        let active_game = active_game.clone();
        let show_language_dialog = show_language_dialog.clone();
        let pending_language = pending_language.clone();
        Callback::from(move |lang: Language| {
            if *active_game {
                // Show confirmation dialog if game is active
                pending_language.set(Some(lang));
                show_language_dialog.set(true);
            } else {
                // Switch language immediately if no game is active
                set_language(lang);
                language_version.set(*language_version + 1);
            }
        })
    };

    let on_confirm_language_change = {
        let language_version = language_version.clone();
        let show_language_dialog = show_language_dialog.clone();
        let pending_language = pending_language.clone();
        let board = board.clone();
        let active_game = active_game.clone();
        Callback::from(move |confirm: bool| {
            show_language_dialog.set(false);
            if confirm {
                if let Some(lang) = (*pending_language).clone() {
                    set_language(lang);
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

    let current_language = get_current_language();

    html! {
        <>
            <div class="language-selector">
                <button 
                    class={if current_language == Language::English { "flag-button active" } else { "flag-button" }}
                    onclick={
                        let on_language_change = on_language_change.clone();
                        Callback::from(move |_| on_language_change.emit(Language::English))
                    }
                    title="English"
                >
                    {"🇬🇧"}
                </button>
                <button 
                    class={if current_language == Language::Czech { "flag-button active" } else { "flag-button" }}
                    onclick={
                        let on_language_change = on_language_change.clone();
                        Callback::from(move |_| on_language_change.emit(Language::Czech))
                    }
                    title="Česky"
                >
                    {"🇨🇿"}
                </button>
            </div>

            // Language confirmation dialog
            if *show_language_dialog {
                <div class="dialog-overlay">
                    <div class="dialog-box">
                        <p class="dialog-message">{t("dialog.confirm_language_switch")}</p>
                        <div class="dialog-buttons">
                            <button 
                                class="dialog-button dialog-button--yes"
                                onclick={
                                    let on_confirm = on_confirm_language_change.clone();
                                    Callback::from(move |_| on_confirm.emit(true))
                                }
                            >
                                {t("dialog.yes")}
                            </button>
                            <button 
                                class="dialog-button dialog-button--no"
                                onclick={
                                    let on_confirm = on_confirm_language_change.clone();
                                    Callback::from(move |_| on_confirm.emit(false))
                                }
                            >
                                {t("dialog.no")}
                            </button>
                        </div>
                    </div>
                </div>
            }

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
                    </div>
                }
            </div>
        </>
    }
}