use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::{errors::DomainError, Document, DocumentChunk};

#[async_trait]
pub trait DocumentStore: Send + Sync {
    async fn save_document(&self, doc: &Document) -> Result<(), DomainError>;
    async fn get_document(&self, id: Uuid) -> Result<Option<Document>, DomainError>;
    async fn delete_document(&self, id: Uuid) -> Result<(), DomainError>;
    async fn save_chunks(&self, chunks: &[DocumentChunk]) -> Result<(), DomainError>;
    async fn get_chunks(&self, document_id: Uuid) -> Result<Vec<DocumentChunk>, DomainError>;
}
