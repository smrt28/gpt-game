
#![allow(dead_code)]

use axum::{
    extract::ConnectInfo,
    routing::get,
    Router,
    extract::State,
    extract::Path,
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
use crate::{gpt, token, GptClientFactory};
use crate::client_pool::*;
use crate::gpt::*;
use shared::shared_error::*;
use tokio::time::timeout;
use tower_http::services::ServeDir;
use tower_http::trace::{TraceLayer, DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, DefaultOnFailure};
use tracing::{info, Level};
use tracing_subscriber::fmt::layer;
use tower_http::classify::{ServerErrorsAsFailures, SharedClassifier};
use tower::{ServiceBuilder};
use shared::messages::{content_response, status_response, GameError, Status, Verdict};
use crate::token::*;
use crate::game_manager::*;
use crate::app_error::*;
use crate::token::TokenType::Answer;
use axum::extract::rejection::QueryRejection;


use serde::de::{self, Deserializer};
//use log::info;

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
    let token = Token::from_string(token_str.as_str())?;
    let pending = state.game_manager.get_game(&token)?.get_pending_question().is_some();

    info!("pending={}", pending);

    let query_val = query.wait;

    if query_val == -1 {
        return Err(AppError::InvalidInput);
    }

    if pending && query_val == 1 {
        info!("waiting");
        let res = state.game_manager.wait_for_answer(&token, Duration::new(5, 0)).await;
        if res.is_err() {
            info!("answer not ready, reason={}", res.as_ref().unwrap_err());
            res?;
        }
        info!("ready");
    }
    info!("responding...");
    let game = state.game_manager.get_game(&token)?;
    info!("responding...2");
    Ok(content_response(if pending {Status::Pending} else {Status::Ok}, game.deref()))
}


async fn ask(
    State(state): State<Shared>,
    ConnectInfo(_addr): ConnectInfo<SocketAddr>,
    Path(token_str): Path<String>,
    body: Bytes
) -> Result<String, AppError> {
    info!("ggg a");
    let token = Token::from_string(token_str.as_str())?;

    info!("ggg b");
    let Some(question) = sanitize_question(&String::from_utf8_lossy(&body).to_string()) else {
        return Err(AppError::InvalidToken);
    };

    info!("ggg c");
    let mut g = state.game_manager.get_game(&token)?;
    info!("ggg 1");
    if !g.set_pending_question(&question) {
        info!("pending question");
        return Err(AppError::Pending);
    }
    info!("ggg 2");
    let mut gpt_client = state.client_factory.pop();
    gpt_client.update().await.unwrap();

    info!("ggg 3");

    let mut params = QuestionParams::default();
    params.set_instructions("Short minimalistic answer");

    drop(g);
    info!("got client, asking: {}", question);
    let result = gpt_client.client().ask(question.as_str(), &params).await;


    match result {
        Ok(gpt_answer) => {
            info!("answer received; OK");
            let mut answer = shared::messages::Answer::new();
            answer.verdict = Some(Verdict::NotSet);
            answer.comment = gpt_answer.to_string();
            info!("ggg 4");
            let mut g = state.game_manager.get_game(&token)?;
            g.answer_pending_question(&answer);
        }
        Err(err) => {
            info!("answer received; ERR");
            let mut g = state.game_manager.get_game(&token)?;
           g.handle_error_response(GameError::GPTError(err.to_string()));
        }
    }
    info!("ggg 3");
    Ok(status_response(Status::Ok))
}

async fn index(State(_state): State<Shared>,
             ConnectInfo(_addr): ConnectInfo<SocketAddr>) -> String {
    Token::new(TokenType::Answer).to_string()
}


async fn new_game(State(state): State<Shared>,
               ConnectInfo(_addr): ConnectInfo<SocketAddr>) -> String {
    state.game_manager.new_game().to_string()
}
