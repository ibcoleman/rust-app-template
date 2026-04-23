use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum GreetError {
    #[error("invalid name: {0}")]
    InvalidName(String),
    #[error("greeting backend failure: {0}")]
    Backend(String),
}

#[async_trait]
pub trait GreetingPort: Send + Sync {
    /// Returns a greeting. When `name` is `None`, returns the default greeting.
    async fn greet(&self, name: Option<&str>) -> Result<String, GreetError>;
}
