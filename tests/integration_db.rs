// @EXAMPLE-FILE notes
// Integration tests for the `Note` example domain — deleted by
// `just clean-examples` along with the rest of the Note scaffolding.
#![cfg(feature = "test-helpers")]

//! Integration tests for PgNotes against real Postgres.
//! These tests require `just dev` to be running (or a DATABASE_URL pointing to a live Postgres).
//! Run with: `just test-integration` or `cargo test --test integration_db --all-features -- --ignored`

use rust_app_template::adapters::PgNotes;
use rust_app_template::ports::{NewNote, NoteRepository, RepoError};

async fn fresh_pool() -> PgNotes {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://app:app@localhost:5432/app".to_string());
    let pool = PgNotes::connect(&database_url)
        .await
        .expect("failed to connect to database");
    pool.reset_for_tests()
        .await
        .expect("failed to reset database");
    pool
}

#[tokio::test]
#[ignore]
async fn create_and_get_roundtrip() {
    // Arrange
    let pg = fresh_pool().await;

    // Act - create a note
    let created = pg
        .create(NewNote {
            body: "test note".to_string(),
        })
        .await
        .expect("create should succeed");

    // Act - get the same note
    let retrieved = pg
        .get(created.id)
        .await
        .expect("get should succeed")
        .expect("note should exist");

    // Assert
    assert_eq!(created.id, retrieved.id);
    assert_eq!(created.body, "test note");
    assert_eq!(created.body, retrieved.body);
    assert_eq!(created.created_at, retrieved.created_at);
}

#[tokio::test]
#[ignore]
async fn list_returns_desc_by_created_at() {
    // Arrange
    let pg = fresh_pool().await;

    // Create 3 notes
    let note1 = pg
        .create(NewNote {
            body: "first".to_string(),
        })
        .await
        .expect("create should succeed");

    let note2 = pg
        .create(NewNote {
            body: "second".to_string(),
        })
        .await
        .expect("create should succeed");

    let note3 = pg
        .create(NewNote {
            body: "third".to_string(),
        })
        .await
        .expect("create should succeed");

    // Act - list all notes
    let listed = pg.list(10).await.expect("list should succeed");

    // Assert - returns 3 notes in descending order
    assert_eq!(listed.len(), 3);
    assert_eq!(listed[0].id, note3.id);
    assert_eq!(listed[1].id, note2.id);
    assert_eq!(listed[2].id, note1.id);
    // Verify descending order
    assert!(listed[0].created_at >= listed[1].created_at);
    assert!(listed[1].created_at >= listed[2].created_at);
}

#[tokio::test]
#[ignore]
async fn oversize_body_rejected() {
    // Arrange
    let pg = fresh_pool().await;
    let oversize_body = "x".repeat(4097);

    // Act
    let result = pg
        .create(NewNote {
            body: oversize_body,
        })
        .await;

    // Assert
    match result {
        Err(RepoError::Validation(msg)) => {
            assert!(msg.contains("exceeds"));
        }
        other => panic!("expected Validation error, got {:?}", other),
    }
}

#[tokio::test]
#[ignore]
async fn nonexistent_id_is_none() {
    // Arrange
    let pg = fresh_pool().await;
    let fake_id = rust_app_template::domain::NoteId::new_v4();

    // Act
    let result = pg.get(fake_id).await.expect("get should succeed");

    // Assert
    assert!(result.is_none());
}

#[tokio::test]
#[ignore]
async fn limit_zero_returns_empty() {
    // Arrange
    let pg = fresh_pool().await;

    // Create a note
    pg.create(NewNote {
        body: "test".to_string(),
    })
    .await
    .expect("create should succeed");

    // Act
    let listed = pg.list(0).await.expect("list should succeed");

    // Assert
    assert_eq!(listed.len(), 0);
}
