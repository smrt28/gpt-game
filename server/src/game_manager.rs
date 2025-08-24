#![allow(dead_code)]

use std::sync::Arc;
use crate::token::*;
use dashmap::DashMap;
use dashmap::mapref::one::RefMut;
use tracing_subscriber::prelude::__tracing_subscriber_Layer;
use shared::messages::*;
use crate::app_error::AppError;
//use tokio::sync::Notify;
pub fn sanitize_question(question: &String) -> Option<String> {
    if question.len() > 120 {
        return None;
    }
    Some(question.clone())
}

pub struct GameManager {
    game_states: Arc<DashMap<Token, GameState>>,
}

impl GameManager {
    pub fn new() -> Self {
        GameManager {
            game_states: Arc::new(DashMap::new()),
        }
    }

    pub fn get_game(&self, token: &Token) -> Result<RefMut<Token, GameState>, AppError> {
        match self.game_states.get_mut(token) {
            Some(rv) => Ok(rv),
            None => Err(AppError::GameNotFound),
        }
    }

    pub fn notice_answer(&self, token: &Token, answer: &Answer) -> Result<(), AppError> {
        self.get_game(token)?.answer_pending_question(answer);
        Ok(())
    }

    pub fn new_game(&self) -> Token {
        let token = Token::new(TokenType::Game);
        self.game_states.insert(token.clone(), GameState::default());
        token
    }
}