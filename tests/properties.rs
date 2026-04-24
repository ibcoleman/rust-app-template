use proptest::prelude::*;
use rust_app_template::adapters::StaticGreeter;
use rust_app_template::domain::MAX_GREET_NAME_LEN;
use rust_app_template::ports::{GreetError, GreetingPort};
// @EXAMPLE-BLOCK-START notes
use rust_app_template::ports::{NewNote, NoteRepository};
use std::sync::Arc;
// @EXAMPLE-BLOCK-END notes

mod support;
// @EXAMPLE-BLOCK-START notes
use support::InMemoryNoteRepository;
// @EXAMPLE-BLOCK-END notes

proptest! {
    /// Property 1: Valid names produce a response containing the name verbatim and ending with "!"
    /// For any `s: String` of length `1..=MAX_GREET_NAME_LEN` (ASCII alphanumeric + spaces),
    /// `StaticGreeter::greet(Some(&s))` returns `Ok(msg)` where `msg.contains(&s)` and `msg.ends_with("!")`.
    #[test]
    fn greet_contains_name(s in "[a-zA-Z ]{1,64}") {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(StaticGreeter::new().greet(Some(&s)));

        // Future-proof: guards against non-ASCII regex flags.
        prop_assume!(s.len() <= MAX_GREET_NAME_LEN);

        let msg = result.unwrap();
        prop_assert!(msg.contains(&s), "Expected '{}' to contain '{}'", msg, s);
        prop_assert!(msg.ends_with('!'), "Expected '{}' to end with '!'", msg);
    }

    /// Property 2: Overlong names always produce `InvalidName`
    /// For any `s: String` with `s.len() > MAX_GREET_NAME_LEN`,
    /// `greet(Some(&s))` returns `Err(GreetError::InvalidName(_))`.
    #[test]
    fn overlong_is_invalid(
        s in proptest::collection::vec(any::<char>(), (MAX_GREET_NAME_LEN + 1)..256)
            .prop_map(|chars| chars.into_iter().collect::<String>())
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let err = runtime
            .block_on(StaticGreeter::new().greet(Some(&s)))
            .unwrap_err();

        prop_assert!(
            matches!(err, GreetError::InvalidName(_)),
            "Expected InvalidName, got {:?}",
            err
        );
    }

    // @EXAMPLE-BLOCK-START notes
    /// Property 3 (Notes): For any body of length 1..=MAX_NOTE_BODY_LEN,
    /// create → get round-trip returns Some(Note) with matching body.
    #[test]
    fn note_create_get_roundtrip(body in "[a-zA-Z0-9 ]{1,4096}") {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let repo = Arc::new(InMemoryNoteRepository::new());

        let created = runtime.block_on(repo.create(NewNote {
            body: body.clone(),
        })).expect("create should succeed");

        let retrieved = runtime.block_on(repo.get(created.id))
            .expect("get should succeed")
            .expect("note should exist after create");

        prop_assert_eq!(created.id, retrieved.id);
        prop_assert_eq!(created.body, body.clone());
        prop_assert_eq!(retrieved.body, body);
    }

    /// Property 4 (Notes): Insert N notes, list(limit) returns min(N, limit)
    /// notes in strictly non-increasing created_at order.
    #[test]
    fn note_list_limit_and_ordering(
        bodies in prop::collection::vec("[a-zA-Z0-9]{1,100}", 1..=10)
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let repo = Arc::new(InMemoryNoteRepository::new());

        let limit = (bodies.len() as u32).saturating_sub(1);

        // Create all notes
        for body in &bodies {
            runtime.block_on(repo.create(NewNote {
                body: body.clone(),
            })).expect("create should succeed");
        }

        // List with limit
        let listed = runtime.block_on(repo.list(limit))
            .expect("list should succeed");

        // Check count: min(N, limit)
        let expected_count = std::cmp::min(bodies.len(), limit as usize);
        prop_assert_eq!(listed.len(), expected_count, "Expected {} notes, got {}", expected_count, listed.len());

        // Check ordering: strictly non-increasing created_at
        for i in 0..listed.len().saturating_sub(1) {
            prop_assert!(
                listed[i].created_at >= listed[i + 1].created_at,
                "Notes should be in non-increasing created_at order"
            );
        }
    }
    // @EXAMPLE-BLOCK-END notes
}
