pub mod error;
pub mod greet;

use std::sync::Arc;

use axum::{routing::get, Router};
use tower_http::trace::TraceLayer;

use crate::ports::GreetingPort;

#[derive(Clone)]
pub struct AppState {
    pub greeter: Arc<dyn GreetingPort>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/api/greet", get(greet::handler))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn healthz() -> &'static str {
    "ok"
}
