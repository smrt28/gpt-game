#![allow(dead_code)]

use std::cell::RefCell;
use std::ops::Deref;
use std::sync::Arc;
use shared::token::*;
use dashmap::DashMap;
use dashmap::mapref::one::RefMut;
use tracing_subscriber::prelude::__tracing_subscriber_Layer;
use shared::messages::*;
use crate::app_error::AppError;
use tokio::sync::Notify;
use tokio::time::{self, Instant};
pub fn sanitize_question(question: &String) -> Option<String> {
    if question.len() > 120 {
        return None;
    }
    Some(question.clone())
}
use std::time::Duration;
use serde_json::json;
use tracing::info;
use crate::token_gen::TokenGen;

#[derive(Default)]
struct StateHelper {
    notifier: Arc<Notify>,
}

pub struct GameManager {
    game_states: Arc<DashMap<Token, GameState>>,
    helpers: Arc<DashMap<Token, StateHelper>>,
}

impl GameManager {
    pub fn new() -> Self {
        GameManager {
            game_states: Arc::new(DashMap::new()),
            helpers: Arc::new(DashMap::new()),
        }
    }

    fn get_game(&self, token: &Token) -> Result<RefMut<'_, Token, GameState>, AppError> {
        match self.game_states.get_mut(token) {
            Some(rv) => Ok(rv),
            None => Err(AppError::GameNotFound),
        }
    }

    fn get_notifier(&self, token: &Token) -> Result<Arc<Notify>, AppError> {
        let h = self.helpers.get(token).ok_or(AppError::GameNotFound)?;
        Ok(h.notifier.clone())
    }

    pub fn notice_answer(&self, token: &Token, _answer: &Answer) -> Result<(), AppError> {
        self.get_notifier(token)?.notify_waiters();
        Ok(())
    }

    pub async fn wait_for_answer(&self, token: &Token, t: Duration ) -> Result<(), AppError> {
        info!("waiting for answer BEGIN");
        let res = match time::timeout(t, self.get_notifier(token)?.notified()).await {
            Ok(_) => Ok(()),
            Err(_) => Err(AppError::Timeout),
        };
        info!("waiting for answer END");
        res
    }

    pub fn delete_game(&self, token: &Token) -> Result<(), AppError> {
        let notifier = self.get_notifier(token)?;
        self.game_states.remove(token);
        self.helpers.remove(token);
        notifier.notify_waiters();
        Ok(())
    }

    pub fn new_game(&self, target: &str) -> Token {
        let token = Token::new(TokenType::Game);
        let mut game = GameState::default();
        game.target = Some(target.to_string());
        self.game_states.insert(token.clone(), game);
        self.helpers.insert(token.clone(), StateHelper::default());
        token
    }

    pub fn get_target(&self, token: &Token) -> Result<String, AppError> {
        let g = self.get_game(token)?;
        if let Some(target) = &g.target {
            return Ok(target.clone());
        }
        Err(AppError::InternalServerError)
    }

    pub fn set_pending_question(&self, token: &Token, question: &String) -> Result<(), AppError> {
        let mut g = self.get_game(token)?;
        if g.pending_question.is_some() {
            info!("Can't ask while previous question is pending.");
            return Err(AppError::Pending);
        }
        g.pending_question = Some(Question{text: question.clone()});
        Ok(())
    }

    pub fn answer_pending_question(&self, token: &Token, answer: &Answer) -> Result<(), AppError> {
        let mut g = self.get_game(token)?;
        if g.pending_question.is_none() {
            return Ok(());
        }
        let mut record = Record::new(g.pending_question.take().unwrap().text);
        record.set_answer(answer);
        g.add_record(record);
        Ok(())
    }

    pub fn handle_error_response(&self, token: &Token, error: GameError) -> Result<(), AppError> {
        let mut g = self.get_game(token)?;
        g.error = Some(error);
        g.pending_question = None;
        Ok(())
    }

    pub fn is_pending(&self, token: &Token) -> Result<bool, AppError> {
        Ok(self.get_game(token)?.pending_question.is_some())
    }

    pub fn game_to_value(&self, token: &Token) -> Result<serde_json::Value, AppError> {
        Ok(serde_json::to_value(self.get_game(token)?.deref())?)
    }

    pub fn get_game_state(&self, token: &Token) -> Result<GameState, AppError> {
        Ok(self.get_game(token)?.deref().clone())
    }
}