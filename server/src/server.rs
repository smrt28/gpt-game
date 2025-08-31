
#![allow(dead_code)]

use axum::{
    extract::ConnectInfo,
    extract::Path,
    extract::State,
    routing::get,
    Router,
};
use anyhow::{Context, Error, Result};
use std::net::SocketAddr;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;
use axum::response::{Html, IntoResponse};
use tokio::{net::TcpListener, sync::Mutex};
use std::sync::Mutex as StdMutex;
use std::time::Duration;
use axum::extract::Query;
use axum::handler::Handler;
use axum::http::StatusCode;
use axum::body::Bytes;
use axum::routing::post;
use clap::builder::Str;
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::{gpt, GptClientFactory};
use crate::client_pool::*;
use crate::gpt::*;
use shared::shared_error::*;
use tokio::time::timeout;
use tower_http::services::ServeDir;
use tower_http::trace::{DefaultMakeSpan, DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::{info, Level};
use tracing_subscriber::fmt::layer;
use tower_http::classify::{ServerErrorsAsFailures, SharedClassifier};
use tower::ServiceBuilder;
use shared::messages::{ status_response, GameError, GameState, ServerResponse, Status, Verdict};
use shared::token::*;
use crate::game_manager::*;
use crate::app_error::*;
use shared::token::TokenType::Answer;
use axum::extract::rejection::QueryRejection;
use serde::de::{self, Deserializer};
use shared::token;
use crate::token_gen::TokenGen;
use crate::game_prompt::GameStepBuilder;

fn de_opt_bool<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Option::<String>::deserialize(deserializer)?;
    Ok(match s.as_deref() {
        Some("1") | Some("true") | Some("on") | Some("yes") => 1,
        Some("0") | Some("false") | Some("off") | Some("no") => 0,
        Some(_other) => -1,
        None => 0,
    })
}

#[derive(Deserialize)]
struct WaitParam {
    #[serde(default, deserialize_with = "de_opt_bool")]
    wait: i32,
    #[serde(default, deserialize_with = "de_opt_bool")]
    quiet: i32,
}

impl WaitParam {
    fn check(&self) -> Result<(), AppError> {
        if self.wait == -1 || self.quiet == -1 {
            return Err(AppError::InvalidInput);
        }
        Ok(())
    }
}

struct AppState {
    counter: Mutex<u32>,
    client_factory: Arc<ClientsPool::<GptClient>>,
    config: Config,
    game_manager: GameManager,
}

#[derive(Default, Clone)]
pub struct Config {
    pub www_root_path: Option<PathBuf>,
    pub port: u16,
}

impl AppState {
    fn new(factory: Arc<dyn PollableClientFactory<GptClient> + Send + Sync>, config: &Config) -> Self {
        Self {
            counter: Mutex::new(0),
            client_factory: Arc::new(ClientsPool::<GptClient>::new(factory)),
            config: config.clone(),
            game_manager: GameManager::new()
        }
    }
}

type Shared = Arc<AppState>;


fn logging() -> TraceLayer<SharedClassifier<ServerErrorsAsFailures>> {
    TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::DEBUG))
        .on_request(DefaultOnRequest::new().level(Level::DEBUG))
        .on_response(DefaultOnResponse::new().level(Level::DEBUG))
        .on_failure(DefaultOnFailure::new().level(Level::ERROR))
}

pub async fn run_server(
    config: &Config,
    factory: Arc<dyn PollableClientFactory<GptClient> + Send + Sync>,) -> anyhow::Result<()> {
    let state = Shared::new(AppState::new(factory, config));
    tracing::info!("starting server on port {}", config.port);

    let mut app = Router::new()
        .route("/api/token", get(index))
        .route("/api/game/new", get(new_game))
        .route("/api/game/{token}/ask", post(ask))
        .route("/api/game/{token}", get(game))
        .fallback(get(handler_404))
        ;

    if let Some(root) = &config.www_root_path {
        let static_svc = ServiceBuilder::new()
            .layer(logging())
            .service(
                ServeDir::new(root)
                    .append_index_html_on_directories(true)
                    .precompressed_br()
                    .precompressed_gzip(),
            );
        app = app.nest_service("/static", static_svc);
    }

    app = app.layer(logging());

    let app = app
        .fallback(handler_404)
        .with_state(state)
        .layer(logging());

    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
    let listener = TcpListener::bind(addr).await?;

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Not found")
}


async fn game(State(state): State<Shared>,
                      ConnectInfo(_addr): ConnectInfo<SocketAddr>,
                      Path(token_str): Path<String>,
                      Query(query): Query<WaitParam>

) -> Result<String, AppError> {
    info!("game");
    query.check()?;
    let token = Token::from_string(token_str.as_str())?;
    let pending = state.game_manager.is_pending(&token)?;

    if pending && query.wait == 1 {
        info!("waiting");
        let res = state.game_manager.wait_for_answer(&token, Duration::new(5, 0)).await;
        if res.is_err() {
            info!("answer not ready, reason={}", res.as_ref().unwrap_err());
            res?;
        }
        info!("ready");
    }

    let status = if pending {Status::Pending} else {Status::Ok};

    if query.quiet == 1 && status == Status::Pending {
        let game_state = ServerResponse::<()>::from_status(status);
        return Ok(game_state.to_response()?);
    }

    let game_state = state.game_manager.get_game_state(&token)?;
    Ok(ServerResponse::from_content(status, game_state).to_response()?)
}


async fn ask(
    State(state): State<Shared>,
    ConnectInfo(_addr): ConnectInfo<SocketAddr>,
    Path(token_str): Path<String>,
    body: Bytes,
) -> Result<String, AppError> {
    let token = Token::from_string(token_str.as_str())?;

    let Some(question) = sanitize_question(&String::from_utf8_lossy(&body).to_string()) else {
        return Err(AppError::InvalidToken);
    };

    let mut gpt_client = state.client_factory.pop();
    gpt_client.update().await.unwrap();


    let mut question_builder = GameStepBuilder::default();

    question_builder
        .set_target(state.game_manager.get_target(&token)?.as_str())
        .set_question(question.as_str())
        .check()?
    ;

    state.game_manager.set_pending_question(&token, &question_builder.get_original_question())?;

    // !!! ASK !!!
    tokio::spawn(async move {
        info!("got client, asking: {}", question);
        let result = gpt_client.client().ask(&question_builder.build_question(),
                                             &question_builder.build_params()).await;

        match result {
            Ok(gpt_answer) => {
                info!("answer received; OK");

                let s = gpt_answer.to_string().unwrap_or("UNABLE; this is weird".to_string());

                let answer = shared::messages::Answer::parse_from_string(&s);
                let _ = state.game_manager.answer_pending_question(&token, &answer);
            }
            Err(err) => {
                info!("answer received; ERR");
                let _ = state.game_manager.handle_error_response(&token,
                       GameError::GPTError(err.to_string()));
            }
        }
    });

    // -----------
    Ok(status_response(Status::Ok))
}

async fn index(State(_state): State<Shared>,
             ConnectInfo(_addr): ConnectInfo<SocketAddr>) -> String {
    Token::new(TokenType::Answer).to_string()
}


async fn new_game(State(state): State<Shared>,
               ConnectInfo(_addr): ConnectInfo<SocketAddr>) -> String {
    state.game_manager.new_game("Yellow submarine").to_string()
}
