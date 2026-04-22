pub mod error;

use axum::{extract::State, routing::get, Router};
use tower_http::trace::TraceLayer;

#[derive(Clone, Default)]
pub struct AppState {
    // Fields added in Phase 3 (greeter) and Phase 4 (notes).
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn healthz(State(_): State<AppState>) -> &'static str {
    "ok"
}
