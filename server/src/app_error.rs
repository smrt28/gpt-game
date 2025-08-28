#![allow(dead_code)]

use axum::{response::{IntoResponse, Response}, http::StatusCode, Json};
use serde::Serialize;

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
        let http_status = match self {
            AppError::Timeout => StatusCode::OK,
            AppError::Pending => StatusCode::OK,
            AppError::InvalidInput => StatusCode::BAD_REQUEST,
            AppError::InvalidToken => StatusCode::BAD_REQUEST,
            AppError::GameNotFound => StatusCode::NOT_FOUND,
            AppError::JsonError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Any(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let status = match self {
            AppError::Timeout => "timeout",
            AppError::Pending => "pending",
            _ => "error"
        };

        let body = serde_json::json!({
            "status": status,
           // "message": self.to_string(),
        });
        (http_status, Json(body)).into_response()
    }
}