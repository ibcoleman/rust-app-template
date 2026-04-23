use async_trait::async_trait;

use crate::domain::MAX_GREET_NAME_LEN;
use crate::ports::{GreetError, GreetingPort};

/// Pure in-process `GreetingPort` implementation. Length-validates `name`.
#[derive(Clone, Default)]
pub struct StaticGreeter;

impl StaticGreeter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl GreetingPort for StaticGreeter {
    async fn greet(&self, name: Option<&str>) -> Result<String, GreetError> {
        match name {
            Some(n) if n.len() > MAX_GREET_NAME_LEN => Err(GreetError::InvalidName(format!(
                "name must be <= {MAX_GREET_NAME_LEN} bytes (got {})",
                n.len()
            ))),
            Some(n) => Ok(format!("Hello, {n}!")),
            None => Ok("Hello, world!".to_string()),
        }
    }
}
