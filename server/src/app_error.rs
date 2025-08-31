#![allow(dead_code)]

use axum::{response::{IntoResponse, Response}, http::StatusCode, Json};
use serde::Serialize;
use shared::messages::Status;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("invalid token")]
    InvalidToken,

    #[error("pending")]
    Pending,

    #[error("game not found")]
    GameNotFound,

    #[error("invalid input")]
    InvalidInput,

    #[error(transparent)]
    Any(#[from] anyhow::Error),

    #[error("internal server error")]
    JsonError(#[from] serde_json::Error),

    #[error("internal server error")]
    InternalServerError,

    #[error("timeout")]
    Timeout
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let http_status = StatusCode::OK;
        let status = match self {
            AppError::Pending => Status::Pending,
            AppError::Timeout => Status::Pending,
            _ => Status::Error
        };

        let invalid_token = match self {
            AppError::InvalidToken => true,
            AppError::GameNotFound => true,
            _ => false
        };

        let body = serde_json::json!({
            "status": status,
            "invalid_token": invalid_token,
            "message": self.to_string()
        });
        (http_status, Json(body)).into_response()
    }
}