use std::sync::Arc;
use tracing::instrument;
use uuid::Uuid;

use crate::domain::{chunk_content, ports::DocumentStore, Document, DocumentChunk, DomainError};

pub struct DocumentService {
    store: Arc<dyn DocumentStore>,
    chunk_size: usize,
}

impl DocumentService {
    pub fn new(store: Arc<dyn DocumentStore>) -> Self {
        Self {
            store,
            chunk_size: 1000,
        }
    }

    pub fn with_chunk_size(store: Arc<dyn DocumentStore>, chunk_size: usize) -> Self {
        Self { store, chunk_size }
    }

    #[instrument(skip(self, content), fields(name))]
    pub async fn ingest(
        &self,
        name: &str,
        content: &str,
    ) -> Result<(Document, Vec<DocumentChunk>), DomainError> {
        let doc = Document::new(name);
        self.store.save_document(&doc).await?;

        let chunks = chunk_content(doc.id, content, self.chunk_size);
        if !chunks.is_empty() {
            self.store.save_chunks(&chunks).await?;
        }

        Ok((doc, chunks))
    }

    #[instrument(skip(self))]
    pub async fn get(&self, id: Uuid) -> Result<Option<Document>, DomainError> {
        self.store.get_document(id).await
    }

    #[instrument(skip(self))]
    pub async fn get_with_chunks(
        &self,
        id: Uuid,
    ) -> Result<Option<(Document, Vec<DocumentChunk>)>, DomainError> {
        match self.store.get_document(id).await? {
            Some(doc) => {
                let chunks = self.store.get_chunks(id).await?;
                Ok(Some((doc, chunks)))
            }
            None => Ok(None),
        }
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        self.store.delete_document(id).await
    }
}
