//! Shared test-support module. Fakes and helpers live here.
//!
//! Phase 2: empty.
//! Phase 3: adds `FakeGreeter`.
//! Phase 4: adds `InMemoryNoteRepository`.

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use rust_app_template::ports::{GreetError, GreetingPort};
// @EXAMPLE-BLOCK-START notes
use rust_app_template::domain::NoteId;
use rust_app_template::ports::{NewNote, Note, NoteRepository, RepoError};
use time::OffsetDateTime;
// @EXAMPLE-BLOCK-END notes

/// Fake implementation of `GreetingPort` for testing.
/// Holds a `HashMap<Option<String>, Result<String, GreetError>>` so tests can seed canned responses.
#[derive(Default)]
pub struct FakeGreeter {
    responses: Mutex<HashMap<Option<String>, Result<String, GreetError>>>,
}

#[allow(dead_code)]
impl FakeGreeter {
    /// Configure an expected response for a given name.
    pub fn expect(&self, name: Option<&str>, resp: Result<String, GreetError>) {
        self.responses
            .lock()
            .unwrap()
            .insert(name.map(str::to_string), resp);
    }
}

#[async_trait]
impl GreetingPort for FakeGreeter {
    async fn greet(&self, name: Option<&str>) -> Result<String, GreetError> {
        let key = name.map(str::to_string);
        self.responses
            .lock()
            .unwrap()
            .get(&key)
            .cloned()
            .unwrap_or_else(|| panic!("FakeGreeter: no response seeded for {key:?}"))
    }
}

// @EXAMPLE-BLOCK-START notes
/// In-memory implementation of `NoteRepository` for testing.
/// Stores notes in a `Mutex<Vec<Note>>`. Uses `Mutex.lock().unwrap()` which is fine in tests.
#[derive(Default)]
pub struct InMemoryNoteRepository {
    notes: Mutex<Vec<Note>>,
}

impl InMemoryNoteRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl NoteRepository for InMemoryNoteRepository {
    async fn create(&self, new: NewNote) -> Result<Note, RepoError> {
        let note = Note {
            id: NoteId::new_v4(),
            body: new.body,
            created_at: OffsetDateTime::now_utc(),
        };

        self.notes.lock().unwrap().push(note.clone());
        Ok(note)
    }

    async fn get(&self, id: NoteId) -> Result<Option<Note>, RepoError> {
        Ok(self
            .notes
            .lock()
            .unwrap()
            .iter()
            .find(|n| n.id == id)
            .cloned())
    }

    async fn list(&self, limit: u32) -> Result<Vec<Note>, RepoError> {
        let mut notes = self.notes.lock().unwrap().clone();
        // Sort by created_at descending
        notes.sort_by_key(|n| std::cmp::Reverse(n.created_at));
        Ok(notes.into_iter().take(limit as usize).collect())
    }
}
// @EXAMPLE-BLOCK-END notes
