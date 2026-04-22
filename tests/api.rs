mod support;

use axum_test::TestServer;
use rust_app_template::adapters::StaticGreeter;
use rust_app_template::http::{router, AppState};
use rust_app_template::ports::GreetError;
use std::sync::Arc;
use support::FakeGreeter;

#[tokio::test]
async fn healthz_returns_ok() {
    // Arrange
    let state = AppState {
        greeter: Arc::new(StaticGreeter::new()),
    };
    let app = router(state);
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
    let state = AppState {
        greeter: Arc::new(StaticGreeter::new()),
    };
    let app = router(state);
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
    let state = AppState {
        greeter: Arc::new(StaticGreeter::new()),
    };
    let app = router(state);
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
    let state = AppState {
        greeter: Arc::new(StaticGreeter::new()),
    };
    let app = router(state);
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
