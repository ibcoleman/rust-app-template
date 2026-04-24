use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

use crate::ports::GreetError;
// @EXAMPLE-BLOCK-START notes
use crate::ports::RepoError;
// @EXAMPLE-BLOCK-END notes

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error(transparent)]
    Greet(#[from] GreetError),
    // @EXAMPLE-BLOCK-START notes
    #[error(transparent)]
    Repo(#[from] RepoError),
    // @EXAMPLE-BLOCK-END notes
    /// Returned by route handlers when a requested resource is absent.
    #[error("not found")]
    NotFound,
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
            // @EXAMPLE-BLOCK-START notes
            ApiError::Repo(RepoError::Validation(m)) => (StatusCode::BAD_REQUEST, m.clone()),
            ApiError::Repo(RepoError::Backend(m)) => {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("internal: {m}"))
            }
            // @EXAMPLE-BLOCK-END notes
            ApiError::NotFound => (StatusCode::NOT_FOUND, "not found".to_string()),
        };
        (status, Json(json!({ "error": msg }))).into_response()
    }
}
