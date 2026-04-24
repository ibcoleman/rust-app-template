// @EXAMPLE-FILE notes
// HTTP handlers for the `Note` example domain — deleted by
// `just clean-examples`. Pattern reference for JSON-in / JSON-out routes
// with path + query extractors.

use std::str::FromStr;
use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use crate::domain::{NoteId, MAX_NOTE_BODY_LEN};
use crate::http::{error::ApiError, AppState};
use crate::ports::{NewNote, NoteRepository};

#[derive(Deserialize)]
pub struct ListQuery {
    pub limit: Option<u32>,
}

pub async fn create_note(
    State(state): State<AppState>,
    Json(new): Json<NewNote>,
) -> Result<impl IntoResponse, ApiError> {
    if new.body.len() > MAX_NOTE_BODY_LEN {
        return Err(ApiError::BadRequest(format!(
            "body exceeds {MAX_NOTE_BODY_LEN} bytes"
        )));
    }
    let notes: &Arc<dyn NoteRepository> = &state.notes;
    let note = notes.create(new).await?;
    Ok((StatusCode::CREATED, Json(note)))
}

pub async fn get_note(
    State(state): State<AppState>,
    Path(id_str): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let id =
        NoteId::from_str(&id_str).map_err(|e| ApiError::BadRequest(format!("invalid id: {e}")))?;
    match state.notes.get(id).await? {
        Some(n) => Ok(Json(n)),
        None => Err(ApiError::NotFound),
    }
}

pub async fn list_notes(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let limit = q.limit.unwrap_or(20);
    if limit == 0 {
        return Ok(Json(Vec::<crate::ports::Note>::new()));
    }
    Ok(Json(state.notes.list(limit).await?))
}
