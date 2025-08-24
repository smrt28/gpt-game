#![allow(dead_code)]

use axum::{response::{IntoResponse, Response}, http::StatusCode, Json};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("invalid token")]
    InvalidToken,
    #[error("game not found")]
    GameNotFound,

    #[error("invalid input")]
    InvalidInput,

    #[error(transparent)]
    Any(#[from] anyhow::Error),

    #[error("internal server error")]
    JsonError(#[from] serde_json::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::InvalidInput => StatusCode::BAD_REQUEST,
            AppError::InvalidToken => StatusCode::BAD_REQUEST,
            AppError::GameNotFound => StatusCode::NOT_FOUND,
            AppError::JsonError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Any(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = serde_json::json!({
            "status": "error",
            "message": self.to_string(),
        });
        (status, Json(body)).into_response()
    }
}