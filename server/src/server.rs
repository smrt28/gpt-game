
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use axum::{
    body::Bytes,
    extract::{ConnectInfo, Path, Query, Request, State},
    http::{HeaderMap, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use tokio::{net::TcpListener, sync::Mutex};
use tower::ServiceBuilder;
use tower_http::{
    classify::{ServerErrorsAsFailures, SharedClassifier},
    services::{ServeDir, ServeFile},
    trace::{DefaultMakeSpan, DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::{info, warn, Level};

use crate::{
    app_error::*,
    client_pool::*,
    game_manager::*,
    game_prompt::GameStepBuilder,
    gpt::*,
    token_gen::TokenGen,
    Config,
};
use shared::{
    messages::{status_response, GameError, ServerResponse, Status},
    token::*,
};
use serde::de::Deserializer;
use shared::locale::Language;
use crate::locale::t;

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
struct NewGameParam {
    #[serde(default)]
    lang: Option<String>,
}

impl NewGameParam {
    fn get_language(&self) -> Language {
        match &self.lang {
            Some(lang) => {
                Language::from_str(&lang).unwrap_or_else(||
                    Language::English)
            },
            None => Language::English,
        }
    }
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
    #[allow(dead_code)]
    counter: Mutex<u32>,
    client_factory: Arc<ClientsPool::<GptClient>>,
    config: Config,
    game_manager: GameManager,
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


async fn log_requests(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request,
    next: Next,
) -> Response {
    let real_ip = get_real_ip(req.headers(), addr.ip());
    info!("{} {} {}", req.method(), req.uri().path_and_query().map(|x| x.as_str()).unwrap_or(""), real_ip);
    next.run(req).await
}

fn get_real_ip(headers: &HeaderMap, fallback_ip: std::net::IpAddr) -> std::net::IpAddr {
    // Try X-Forwarded-For header first
    if let Some(xff) = headers.get("x-forwarded-for") {
        if let Ok(xff_str) = xff.to_str() {
            // X-Forwarded-For can be "client, proxy1, proxy2"
            // We want the first (leftmost) IP which is the original client
            if let Some(first_ip) = xff_str.split(',').next() {
                if let Ok(ip) = first_ip.trim().parse() {
                    return ip;
                }
            }
        }
    }
    
    // Try X-Real-IP header
    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            if let Ok(ip) = ip_str.parse() {
                return ip;
            }
        }
    }
    
    // Fallback to the direct connection IP
    fallback_ip
}

fn logging() -> TraceLayer<SharedClassifier<ServerErrorsAsFailures>> {
    TraceLayer::new_for_http()
        .make_span_with(
            DefaultMakeSpan::new()
                .level(Level::DEBUG)
                .include_headers(false)
        )
        .on_request(
            DefaultOnRequest::new()
                .level(Level::DEBUG)
        )
        .on_response(
            DefaultOnResponse::new()
                .level(Level::DEBUG)
                .include_headers(false)
        )
        .on_failure(DefaultOnFailure::new().level(Level::ERROR))
}

async fn redirect_to_game() -> Redirect {
    Redirect::to("/run/game")
}

pub async fn run_server(
    config: &Config,
    factory: Arc<dyn PollableClientFactory<GptClient> + Send + Sync>,) -> anyhow::Result<()> {
    let state = Shared::new(AppState::new(factory, config));
    tracing::info!("starting server on port {}", config.www.port);

    let mut app = Router::new()
        .route("/api/token", get(index))
        .route("/api/game/new", get(new_game))
        .route("/api/game/{token}/ask", post(ask))
        .route("/api/game/{token}", get(game))
        .route("/", get(redirect_to_game))
        .fallback(get(handler_404))
        ;

    let static_svc = ServiceBuilder::new()
        .layer(logging())
        .service(
            ServeDir::new(&config.www.www)
                .append_index_html_on_directories(true)
                .precompressed_br()
                .precompressed_gzip(),
        );
    app = app.nest_service("/static", static_svc);



    let static_svc = ServiceBuilder::new()
        .layer(logging())
        .service(
            ServeDir::new(&config.www.dist)
                .append_index_html_on_directories(true)
                .precompressed_br()
                .precompressed_gzip()
                .not_found_service(ServeFile::new(PathBuf::new().join(&config.www.dist).join("index.html")))
        );

    app = app.nest_service("/run", static_svc);

    app = app.layer(logging());

    let app = app
        .fallback(handler_404)
        .with_state(state)
        .layer(middleware::from_fn(log_requests))
        .layer(logging());

    let addr = SocketAddr::from(([127, 0, 0, 1], config.www.port));
    let listener = TcpListener::bind(addr).await.context("Failed to bind to address")?;

    info!("server bound to {} - ready to accept connections", addr);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .context("Server error")?;

    info!("server shutting down");
    Ok(())
}

async fn handler_404() -> impl IntoResponse {
    info!("404 - route not found");
    (StatusCode::NOT_FOUND, "Not found")
}


async fn game(
    headers: HeaderMap,
    State(state): State<Shared>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(token_str): Path<String>,
    Query(query): Query<WaitParam>
) -> Result<String, AppError> {
    let real_ip = get_real_ip(&headers, addr.ip());
    info!("game request from {}", real_ip);
    query.check()?;
    let token = Token::from_string(token_str.as_str()).map_err(|e| {
        warn!("invalid token from {}: {} - {}", real_ip, token_str, e);
        e
    })?;
    let pending = state.game_manager.is_pending(&token)?;

    if pending && query.wait == 1 {
        info!("client {} waiting for answer", real_ip);
        let res = state.game_manager.wait_for_answer(&token, Duration::new(5, 0)).await;
        if res.is_err() {
            info!("answer not ready for {}, reason={}", real_ip, res.as_ref().unwrap_err());
            res?;
        }
        info!("answer ready for {}", real_ip);
    }

    let status = if pending {Status::Pending} else {Status::Ok};

    if query.quiet == 1 && status == Status::Pending {
        let game_state = ServerResponse::<()>::from_status(status);
        return Ok(game_state.to_response()?);
    }

    let game_state = state.game_manager.get_game_state(&token)?;
    Ok(ServerResponse::from_content(status, game_state).to_response()?)
}


fn normalize_cheat(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == ' ')
        .flat_map(|c| c.to_uppercase())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_give_up(s: &str) -> bool {
    let n = normalize_cheat(s);
    matches!(n.as_str(), "IM LOSER" | "I AM LOSER" | "IM A LOSER" | "KONEC" | "konec")
}

async fn ask(
    headers: HeaderMap,
    State(state): State<Shared>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(token_str): Path<String>,
    body: Bytes,
) -> Result<String, AppError> {
    let real_ip = get_real_ip(&headers, addr.ip());
    let token = Token::from_string(token_str.as_str()).map_err(|e| {
        warn!("invalid token from {}: {} - {}", real_ip, token_str, e);
        e
    })?;
    
    if !state.game_manager.is_game_active(&token)? {
        warn!("asking in inactive game {} {}", token.to_string(), real_ip);
        return Err(AppError::InactiveGame);
    }

    let Some(question) = sanitize_question(&String::from_utf8_lossy(&body).to_string()) else {
        info!("invalid question from {}", real_ip);
        return Err(AppError::InvalidToken);
    };

    info!("question from {}: \"{}\"", real_ip, question);

    let mut gpt_client = state.client_factory.pop();
    gpt_client.update().await.unwrap();


    let language = state.game_manager.get_language(&token)?;

    let question_builder = GameStepBuilder::new(&state.config)
        .set_target(&state.game_manager.get_target(&token)?)
        .set_language(&language)
        .set_question(&question)
        .create()?
    ;

    state.game_manager.set_pending_question(&token, &question_builder.get_original_question())?;

    // !!! ASK !!!
    tokio::spawn(async move {

        if is_give_up(&question) {
            info!("giving up {}: \"{}\"", real_ip, question);

            let template = t(&language, "game.final_answer");
            let final_message = template.replace("{}", &question_builder.get_target());
            let answer = shared::messages::Answer::get_final_answer(&final_message);
            let _ = state.game_manager.answer_pending_question(&token, &answer);
            let _ = state.game_manager.finish_game(&token);
            return
        }

        info!("sending question to GPT for {}: \"{}\"", real_ip, question);
        let result = gpt_client.client().ask(&question_builder.build_question(),
                                             &question_builder.build_params(&state.config)).await;

        match result {
            Ok(gpt_answer) => {
                info!("GPT response received OK for {}", real_ip);


                let s = gpt_answer.to_string().unwrap_or("UNABLE; this is weird".to_string());

                let answer = shared::messages::Answer::parse_from_string(&s);
                let _ = state.game_manager.answer_pending_question(&token, &answer);
            }
            Err(err) => {
                info!("GPT response ERROR for {}: {}", real_ip, err);
                let _ = state.game_manager.handle_error_response(&token,
                       GameError::GPTError(err.to_string()));
            }
        }
    });

    // -----------
    Ok(status_response(Status::Ok))
}

async fn index(
    headers: HeaderMap,
    State(_state): State<Shared>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>
) -> String {
    let real_ip = get_real_ip(&headers, addr.ip());
    let token = Token::new(TokenType::Answer).to_string();
    info!("token request from {}: {}", real_ip, token);
    token
}


async fn new_game(
    headers: HeaderMap,
    State(state): State<Shared>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Query(game_params): Query<NewGameParam>
) -> Result<String, AppError> {
    let real_ip = get_real_ip(&headers, addr.ip());
    let identity = crate::locale::get_random_identity(&game_params.get_language()).ok_or(AppError::InternalServerError)?;
    let game_token = state.game_manager.new_game(&identity, game_params.get_language()).to_string();
    info!("new-game-created-for {}: {}", real_ip, game_token);
    Ok(game_token)
}
