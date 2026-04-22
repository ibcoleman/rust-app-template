mod support;

use axum_test::TestServer;
use rust_app_template::http::{router, AppState};

#[tokio::test]
async fn healthz_returns_ok() {
    // Arrange
    let app = router(AppState::default());
    let server = TestServer::new(app).expect("failed to create test server");

    // Act
    let resp = server.get("/healthz").await;

    // Assert
    resp.assert_status_ok();
    resp.assert_text("ok");
}
