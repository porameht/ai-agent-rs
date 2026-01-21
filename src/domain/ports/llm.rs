use crate::domain::errors::DomainError;
use async_trait::async_trait;

#[async_trait]
pub trait LlmService: Send + Sync {
    async fn complete(&self, prompt: &str) -> Result<String, DomainError>;
    async fn complete_with_system(&self, system: &str, prompt: &str)
        -> Result<String, DomainError>;
}
