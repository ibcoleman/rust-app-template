//! Shared test-support module. Fakes and helpers live here.
//!
//! Phase 2: empty.
//! Phase 3: adds `FakeGreeter`.
//! Phase 4: adds `InMemoryNoteRepository`.

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use rust_app_template::ports::{GreetError, GreetingPort};

/// Fake implementation of `GreetingPort` for testing.
/// Holds a `HashMap<Option<String>, Result<String, GreetError>>` so tests can seed canned responses.
#[derive(Default)]
pub struct FakeGreeter {
    responses: Mutex<HashMap<Option<String>, Result<String, GreetError>>>,
}

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
            .unwrap_or_else(|| Ok(format!("fake:{:?}", key)))
    }
}
