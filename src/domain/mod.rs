/// Upper bound on the `name` argument accepted by the greeting port.
/// Keeps responses bounded in size and exercises the `BadRequest` path.
pub const MAX_GREET_NAME_LEN: usize = 64;

// @EXAMPLE-BLOCK-START notes
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Upper bound on note body length in bytes.
pub const MAX_NOTE_BODY_LEN: usize = 4096;

/// Newtype wrapper over `Uuid` for note identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NoteId(pub(crate) Uuid);

impl NoteId {
    pub fn new_v4() -> Self {
        Self(Uuid::new_v4())
    }

    /// Extract the inner Uuid.
    pub fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl fmt::Display for NoteId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for NoteId {
    type Err = uuid::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(s).map(Self)
    }
}
// @EXAMPLE-BLOCK-END notes
