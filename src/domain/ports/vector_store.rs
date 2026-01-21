use crate::domain::{errors::DomainError, DocumentChunk, Embedding, SearchResult};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait VectorStore: Send + Sync {
    async fn upsert(&self, chunk: &DocumentChunk, embedding: &Embedding)
        -> Result<(), DomainError>;
    async fn search(
        &self,
        query: &Embedding,
        top_k: usize,
    ) -> Result<Vec<SearchResult>, DomainError>;
    async fn delete_by_document(&self, document_id: Uuid) -> Result<(), DomainError>;
}
