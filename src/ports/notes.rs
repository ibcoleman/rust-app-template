// @EXAMPLE-FILE notes
// This whole file is part of the `Note` example domain and gets deleted by
// `just clean-examples`. See docs/ADDING-ADAPTERS.md for the pattern.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::OffsetDateTime;

use crate::domain::NoteId;

#[derive(Debug, Error)]
pub enum RepoError {
    #[error("validation: {0}")]
    Validation(String),
    #[error("backend: {0}")]
    Backend(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: NoteId,
    pub body: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewNote {
    pub body: String,
}

#[async_trait]
pub trait NoteRepository: Send + Sync {
    async fn create(&self, new: NewNote) -> Result<Note, RepoError>;
    async fn get(&self, id: NoteId) -> Result<Option<Note>, RepoError>;
    async fn list(&self, limit: u32) -> Result<Vec<Note>, RepoError>;
}
