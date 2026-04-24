pub mod greeting;
// @EXAMPLE-BLOCK-START notes
pub mod notes;
// @EXAMPLE-BLOCK-END notes

pub use greeting::{GreetError, GreetingPort};
// @EXAMPLE-BLOCK-START notes
pub use notes::{NewNote, Note, NoteRepository, RepoError};
// @EXAMPLE-BLOCK-END notes
