use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

use crate::ports::GreetError;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error(transparent)]
    Greet(#[from] GreetError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, msg) = match &self {
            ApiError::BadRequest(m) => (StatusCode::BAD_REQUEST, m.clone()),
            ApiError::Greet(GreetError::InvalidName(m)) => {
                (StatusCode::BAD_REQUEST, format!("invalid name: {m}"))
            }
            ApiError::Greet(GreetError::Backend(m)) => {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("internal: {m}"))
            }
        };
        (status, Json(json!({ "error": msg }))).into_response()
    }
}
