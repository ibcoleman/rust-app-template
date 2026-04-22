pub mod error;
pub mod greet;
pub mod notes;

use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::trace::TraceLayer;

use crate::ports::{GreetingPort, NoteRepository};

#[derive(Clone)]
pub struct AppState {
    pub greeter: Arc<dyn GreetingPort>,
    pub notes: Arc<dyn NoteRepository>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/api/greet", get(greet::handler))
        .route(
            "/api/notes",
            post(notes::create_note).get(notes::list_notes),
        )
        .route("/api/notes/{id}", get(notes::get_note))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn healthz() -> &'static str {
    "ok"
}
