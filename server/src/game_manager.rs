#![allow(dead_code)]

use std::cell::RefCell;
use std::sync::Arc;
use crate::token::*;
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

    pub fn get_game(&self, token: &Token) -> Result<RefMut<Token, GameState>, AppError> {
        match self.game_states.get_mut(token) {
            Some(rv) => Ok(rv),
            None => Err(AppError::GameNotFound),
        }
    }

    fn get_notifier(&self, token: &Token) -> Result<Arc<Notify>, AppError> {
        let h = self.helpers.get(token).ok_or(AppError::GameNotFound)?;
        Ok(h.notifier.clone())
    }

    pub fn notice_answer(&self, token: &Token, answer: &Answer) -> Result<(), AppError> {
        self.get_notifier(token)?.notify_waiters();
        Ok(())
    }

    pub async fn wait_for_answer(&self, token: &Token, t: Duration ) -> Result<(), AppError> {
        match time::timeout(t, self.get_notifier(token)?.notified()).await {
            Ok(_) => Ok(()),
            Err(_) => Err(AppError::Timeout),
        }
    }

    pub fn delete_game(&self, token: &Token) -> Result<(), AppError> {
        let notifier = self.get_notifier(token)?;
        self.game_states.remove(token);
        self.helpers.remove(token);
        notifier.notify_waiters();
        Ok(())
    }

    pub fn new_game(&self) -> Token {
        let token = Token::new(TokenType::Game);
        self.game_states.insert(token.clone(), GameState::default());
        self.helpers.insert(token.clone(), StateHelper::default());
        token
    }
}