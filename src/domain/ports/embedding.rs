use crate::domain::{errors::DomainError, Embedding};
use async_trait::async_trait;

#[async_trait]
pub trait EmbeddingService: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Embedding, DomainError>;
    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Embedding>, DomainError>;
    fn dimension(&self) -> usize;
}
