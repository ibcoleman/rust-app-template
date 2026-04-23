pub mod greeting;
pub mod notes;

pub use greeting::{GreetError, GreetingPort};
pub use notes::{NewNote, Note, NoteRepository, RepoError};
