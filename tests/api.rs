mod support;

use axum_test::TestServer;
use rust_app_template::adapters::StaticGreeter;
use rust_app_template::http::{router, AppState};
use rust_app_template::ports::GreetError;
use std::sync::Arc;
use support::FakeGreeter;
// @EXAMPLE-BLOCK-START notes
use axum::http::StatusCode;
use support::InMemoryNoteRepository;
// @EXAMPLE-BLOCK-END notes

// A helper that builds AppState for greet/healthz tests.
// When clean-examples strips the `notes` field from AppState, the
// struct literal here reduces back to a single field.
fn test_state() -> AppState {
    AppState {
        greeter: Arc::new(StaticGreeter::new()),
        // @EXAMPLE-BLOCK-START notes
        notes: Arc::new(InMemoryNoteRepository::new()),
        // @EXAMPLE-BLOCK-END notes
    }
}

#[tokio::test]
async fn healthz_returns_ok() {
    // Arrange
    let app = router(test_state());
    let server = TestServer::new(app).expect("failed to create test server");

    // Act
    let resp = server.get("/healthz").await;

    // Assert
    resp.assert_status_ok();
    resp.assert_text("ok");
}

#[tokio::test]
async fn greet_with_name_returns_personalized_greeting() {
    // Arrange - AC2.1: GET /api/greet?name=Ian → 200, body "Hello, Ian!"
    let app = router(test_state());
    let server = TestServer::new(app).expect("failed to create test server");

    // Act
    let resp = server.get("/api/greet?name=Ian").await;

    // Assert
    resp.assert_status_ok();
    resp.assert_text("Hello, Ian!");
}

#[tokio::test]
async fn greet_without_name_returns_default_greeting() {
    // Arrange - AC2.2: GET /api/greet (no name param) → 200, body "Hello, world!"
    let app = router(test_state());
    let server = TestServer::new(app).expect("failed to create test server");

    // Act
    let resp = server.get("/api/greet").await;

    // Assert
    resp.assert_status_ok();
    resp.assert_text("Hello, world!");
}

#[tokio::test]
async fn greet_with_overlong_name_returns_400() {
    // Arrange - AC2.5: GET /api/greet?name=<65-char string> → 400 Bad Request,
    // JSON body contains "error" key with "invalid name" prefix
    let app = router(test_state());
    let server = TestServer::new(app).expect("failed to create test server");

    // A 65-character name (exceeds MAX_GREET_NAME_LEN=64)
    let overlong_name = "a".repeat(65);

    // Act
    let resp = server
        .get(&format!("/api/greet?name={}", overlong_name))
        .await;

    // Assert
    resp.assert_status_bad_request();
    let body = resp.json::<serde_json::Value>();
    assert!(body["error"].as_str().unwrap().starts_with("invalid name"));
}

#[tokio::test]
async fn greet_backend_error_returns_500() {
    // Arrange - Test that a 500 is returned when the backend errors,
    // with body `{"error": "internal: boom"}`
    let fake_greeter = Arc::new(FakeGreeter::default());
    fake_greeter.expect(Some("x"), Err(GreetError::Backend("boom".to_string())));

    let state = AppState {
        greeter: fake_greeter,
        // @EXAMPLE-BLOCK-START notes
        notes: Arc::new(InMemoryNoteRepository::new()),
        // @EXAMPLE-BLOCK-END notes
    };
    let app = router(state);
    let server = TestServer::new(app).expect("failed to create test server");

    // Act
    let resp = server.get("/api/greet?name=x").await;

    // Assert
    resp.assert_status_internal_server_error();
    let body = resp.json::<serde_json::Value>();
    assert_eq!(body["error"].as_str(), Some("internal: boom"));
}

// @EXAMPLE-BLOCK-START notes
// AC2.3: create → get returns the same note
#[tokio::test]
async fn note_create_and_get_roundtrip() {
    // Arrange
    let notes = Arc::new(InMemoryNoteRepository::new());
    let state = AppState {
        greeter: Arc::new(StaticGreeter::new()),
        notes: notes.clone(),
    };
    let app = router(state);
    let server = TestServer::new(app).expect("failed to create test server");

    // Act - create a note
    let create_resp = server
        .post("/api/notes")
        .json(&serde_json::json!({"body": "test note"}))
        .await;

    // Assert - created successfully (201 Created)
    create_resp.assert_status(StatusCode::CREATED);
    let created: serde_json::Value = create_resp.json();
    let note_id = created["id"].as_str().expect("created note has id");
    let body = created["body"].as_str().expect("created note has body");
    assert_eq!(body, "test note");

    // Act - get the same note
    let get_resp = server.get(&format!("/api/notes/{}", note_id)).await;

    // Assert - get returns the same note
    get_resp.assert_status_ok();
    let retrieved: serde_json::Value = get_resp.json();
    assert_eq!(retrieved["id"].as_str(), Some(note_id));
    assert_eq!(retrieved["body"].as_str(), Some("test note"));
}

// AC2.4: list respects limit and returns in descending created_at order
#[tokio::test]
async fn note_list_respects_limit_and_ordering() {
    // Arrange
    let notes = Arc::new(InMemoryNoteRepository::new());
    let state = AppState {
        greeter: Arc::new(StaticGreeter::new()),
        notes: notes.clone(),
    };
    let app = router(state);
    let server = TestServer::new(app).expect("failed to create test server");

    // Create 3 notes
    for i in 1..=3 {
        server
            .post("/api/notes")
            .json(&serde_json::json!({"body": format!("note {}", i)}))
            .await
            .assert_status(StatusCode::CREATED);
    }

    // Act - list with limit=2
    let resp = server.get("/api/notes?limit=2").await;

    // Assert - returns exactly 2 notes in descending order
    resp.assert_status_ok();
    let list: Vec<serde_json::Value> = resp.json();
    assert_eq!(list.len(), 2);
    // Check that created_at is in non-increasing order (most recent first)
    if let (Some(first_ts), Some(second_ts)) = (
        list[0]["created_at"].as_str(),
        list[1]["created_at"].as_str(),
    ) {
        assert!(first_ts >= second_ts, "Notes should be in descending order");
    }
}

// AC2.6: oversize body returns 400 Validation error
#[tokio::test]
async fn note_create_with_oversize_body_returns_400() {
    // Arrange
    let notes = Arc::new(InMemoryNoteRepository::new());
    let state = AppState {
        greeter: Arc::new(StaticGreeter::new()),
        notes: notes.clone(),
    };
    let app = router(state);
    let server = TestServer::new(app).expect("failed to create test server");

    // Create a body that exceeds MAX_NOTE_BODY_LEN (4096 bytes)
    let oversize_body = "x".repeat(4097);

    // Act
    let resp = server
        .post("/api/notes")
        .json(&serde_json::json!({"body": oversize_body}))
        .await;

    // Assert
    resp.assert_status_bad_request();
    let body = resp.json::<serde_json::Value>();
    let error_msg = body["error"].as_str().unwrap();
    assert!(error_msg.starts_with("body exceeds"));
}

// AC2.7: get non-existent note returns 404 Not Found
#[tokio::test]
async fn note_get_nonexistent_returns_404() {
    // Arrange
    let notes = Arc::new(InMemoryNoteRepository::new());
    let state = AppState {
        greeter: Arc::new(StaticGreeter::new()),
        notes: notes.clone(),
    };
    let app = router(state);
    let server = TestServer::new(app).expect("failed to create test server");

    // Use an arbitrary UUID that was never created
    let fake_uuid = "00000000-0000-0000-0000-000000000000";

    // Act
    let resp = server.get(&format!("/api/notes/{}", fake_uuid)).await;

    // Assert
    resp.assert_status_not_found();
}

// AC2.8: list with limit=0 returns empty array
#[tokio::test]
async fn note_list_with_limit_zero_returns_empty() {
    // Arrange
    let notes = Arc::new(InMemoryNoteRepository::new());
    let state = AppState {
        greeter: Arc::new(StaticGreeter::new()),
        notes: notes.clone(),
    };
    let app = router(state);
    let server = TestServer::new(app).expect("failed to create test server");

    // Create a note so we have something to list
    server
        .post("/api/notes")
        .json(&serde_json::json!({"body": "test"}))
        .await
        .assert_status(StatusCode::CREATED);

    // Act
    let resp = server.get("/api/notes?limit=0").await;

    // Assert
    resp.assert_status_ok();
    let list: Vec<serde_json::Value> = resp.json();
    assert_eq!(list.len(), 0);
}
// @EXAMPLE-BLOCK-END notes
