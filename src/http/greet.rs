use std::sync::Arc;

use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use serde::Deserialize;

use crate::http::{error::ApiError, AppState};
use crate::ports::GreetingPort;

#[derive(Deserialize)]
pub struct GreetQuery {
    pub name: Option<String>,
}

pub async fn handler(
    State(state): State<AppState>,
    Query(q): Query<GreetQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let greeter: &Arc<dyn GreetingPort> = &state.greeter;
    let msg = greeter.greet(q.name.as_deref()).await?;
    Ok(msg)
}
