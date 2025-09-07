use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use dashmap::mapref::one::RefMut;
use serde_json;
use tokio::sync::Notify;
use tokio::time;
use tracing::info;

use shared::locale::Language;
use shared::messages::*;
use shared::token::*;

use crate::app_error::AppError;
use crate::token_gen::TokenGen;

pub fn sanitize_question(question: &str) -> Option<String> {
    if question.len() > 120 {
        return None;
    }
    Some(question.to_string())
}

#[derive(Default)]
struct StateHelper {
    notifier: Arc<Notify>,
}




pub struct GameManager {
    game_states: Arc<DashMap<Token, GameState>>,
    helpers: Arc<DashMap<Token, StateHelper>>,
    custom_games: Arc<DashMap<Token, GameTemplate>>,
}

impl GameManager {
    pub fn new() -> Self {
        Self {
            game_states: Arc::new(DashMap::new()),
            helpers: Arc::new(DashMap::new()),
            custom_games: Arc::new(DashMap::new()),
        }
    }

    fn get_game(&self, token: &Token) -> Result<RefMut<'_, Token, GameState>, AppError> {
        self.game_states.get_mut(token).ok_or(AppError::GameNotFound)
    }

    fn get_notifier(&self, token: &Token) -> Result<Arc<Notify>, AppError> {
        let h = self.helpers.get(token).ok_or(AppError::GameNotFound)?;
        Ok(h.notifier.clone())
    }

    #[allow(dead_code)]
    pub fn notice_answer(&self, token: &Token, _answer: &Answer) -> Result<(), AppError> {
        self.get_notifier(token)?.notify_waiters();
        Ok(())
    }

    pub async fn wait_for_answer(&self, token: &Token, timeout: Duration) -> Result<(), AppError> {
        info!("Waiting for answer BEGIN");
        let result = time::timeout(timeout, self.get_notifier(token)?.notified()).await
            .map_err(|_| AppError::Timeout)?;
        info!("Waiting for answer END");
        Ok(result)
    }

    #[allow(dead_code)]
    pub fn delete_game(&self, token: &Token) -> Result<(), AppError> {
        let notifier = self.get_notifier(token)?;
        self.game_states.remove(token);
        self.helpers.remove(token);
        notifier.notify_waiters();
        Ok(())
    }

    #[allow(dead_code)]
    pub fn define_game_template(&self, template: &GameTemplate) -> Result<Token, AppError> {
        let token = Token::new(TokenType::GameTemplate);
        self.custom_games.insert(token, template.clone());
        Ok(token)
    }

    pub fn get_game_template(&self, token: &Token) -> Result<GameTemplate, AppError> {
        Ok(self.custom_games.get(token).ok_or(AppError::GameNotFound)?.deref().clone())
    }

    #[allow(dead_code)]
    pub fn new_game_from_template(&self, template_token: &Token) -> Result<Token, AppError> {
        let template = self.custom_games.get(template_token).ok_or(AppError::GameNotFound)?.deref().clone();
        Ok(self.new_game(&template.identity, template.language, Some(template.properties)))
    }

    pub fn new_game(&self, identity: &str, lang: Language, custom_info: Option<CustomGameInfo>) -> Token {
        let token = Token::new(TokenType::Game);
        let mut game = GameState::default();
        game.lang = lang.clone();
        game.identity = Some(identity.to_string());

        if let Some(custom_info) = custom_info {
            game.is_custom = true;
            game.custom_info = Some(custom_info);
        }

        self.game_states.insert(token, game);
        self.helpers.insert(token, StateHelper::default());
        info!("*** New game: {}; [{}]; lang={}", token.to_string(), identity, lang.to_code());
        token
    }

    pub fn get_target(&self, token: &Token) -> Result<String, AppError> {
        let game = self.get_game(token)?;
        game.identity.clone().ok_or(AppError::InternalServerError)
    }
    
    pub fn get_language(&self, token: &Token) -> Result<Language, AppError> {
        let game = self.get_game(token)?;
        Ok(game.lang.clone())
    }

    pub fn set_pending_question(&self, token: &Token, question: &str) -> Result<(), AppError> {
        let mut game = self.get_game(token)?;
        if game.pending_question.is_some() {
            info!("Can't ask while previous question is pending.");
            return Err(AppError::Pending);
        }
        game.pending_question = Some(Question { text: question.to_string() });
        Ok(())
    }

    pub fn answer_pending_question(&self, token: &Token, answer: &Answer) -> Result<(), AppError> {
        let mut game = self.get_game(token)?;
        let Some(pending_question) = game.pending_question.take() else {
            return Ok(());
        };

        if answer.verdict == Some(Verdict::Final) {
            game.game_ended = true;
        }

        let mut record = Record::new(pending_question.text);
        record.set_answer(answer);
        game.add_record(record);

        // Don't lock the game and the notificator map simultaneously to prevent potential deadlocks.
        drop(game);

        // This is important to notify the client that the answer is ready.
        if let Ok(notifier) = self.get_notifier(token) {
            notifier.notify_one();
        }

        Ok(())
    }

    pub fn handle_error_response(&self, token: &Token, error: GameError) -> Result<(), AppError> {
        let mut game = self.get_game(token)?;
        game.error = Some(error);
        game.pending_question = None;
        Ok(())
    }

    pub fn is_pending(&self, token: &Token) -> Result<bool, AppError> {
        Ok(self.get_game(token)?.pending_question.is_some())
    }

    #[allow(dead_code)]
    pub fn game_to_value(&self, token: &Token) -> Result<serde_json::Value, AppError> {
        Ok(serde_json::to_value(self.get_game(token)?.deref())?)
    }

    pub fn get_game_state(&self, token: &Token) -> Result<GameState, AppError> {
        let mut game = self.get_game(token)?.clone();

        if !game.game_ended {
            game.clear_comments();
        }

        Ok(game)
    }

    pub fn finish_game(&self, token: &Token) -> Result<(), AppError> {
        self.get_game(token)?.game_ended = true;
        Ok(())
    }
    
    pub fn is_game_active(&self, token: &Token) -> Result<bool, AppError> {
        let game = self.get_game(token)?;
        Ok(!game.game_ended)
    }

    #[allow(dead_code)]
    pub fn game_ended(&self, token: &Token) -> bool {
        self.get_game(token)
            .map(|game| game.game_ended)
            .unwrap_or(true)
    }
}