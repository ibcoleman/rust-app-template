pub mod error;
pub mod greet;
// @EXAMPLE-BLOCK-START notes
pub mod notes;
// @EXAMPLE-BLOCK-END notes

use std::sync::Arc;

use axum::{
    body::Body,
    extract::Path,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use rust_embed::RustEmbed;
use tower_http::trace::TraceLayer;

use crate::ports::GreetingPort;
// @EXAMPLE-BLOCK-START notes
use axum::routing::post;
use crate::ports::NoteRepository;
// @EXAMPLE-BLOCK-END notes

#[derive(RustEmbed)]
#[folder = "frontend/dist/"]
struct Assets;

#[derive(Clone)]
pub struct AppState {
    pub greeter: Arc<dyn GreetingPort>,
    // @EXAMPLE-BLOCK-START notes
    pub notes: Arc<dyn NoteRepository>,
    // @EXAMPLE-BLOCK-END notes
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/assets/{*path}", get(asset))
        .route("/healthz", get(healthz))
        .route("/api/greet", get(greet::handler))
        // @EXAMPLE-BLOCK-START notes
        .route(
            "/api/notes",
            post(notes::create_note).get(notes::list_notes),
        )
        .route("/api/notes/{id}", get(notes::get_note))
        // @EXAMPLE-BLOCK-END notes
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn index() -> Response {
    match Assets::get("index.html") {
        Some(f) => Response::builder()
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(Body::from(f.data.into_owned()))
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response()),
        None => (StatusCode::INTERNAL_SERVER_ERROR, "index.html missing").into_response(),
    }
}

async fn asset(Path(path): Path<String>) -> Response {
    let full = format!("assets/{path}");
    match Assets::get(&full) {
        Some(f) => {
            let mime = mime_guess::from_path(&full).first_or_octet_stream();
            Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(Body::from(f.data.into_owned()))
                .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn healthz() -> &'static str {
    "ok"
}
