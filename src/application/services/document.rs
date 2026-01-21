use std::sync::Arc;
use tracing::instrument;
use uuid::Uuid;

use crate::domain::{ports::DocumentStore, ChunkMetadata, Document, DocumentChunk, DomainError};

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

        let chunks = self.chunk_content(&doc.id, content);
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
        let doc = self.store.get_document(id).await?;
        match doc {
            Some(d) => {
                let chunks = self.store.get_chunks(id).await?;
                Ok(Some((d, chunks)))
            }
            None => Ok(None),
        }
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        self.store.delete_document(id).await
    }

    fn chunk_content(&self, document_id: &Uuid, content: &str) -> Vec<DocumentChunk> {
        let paragraphs: Vec<&str> = content
            .split("\n\n")
            .filter(|s| !s.trim().is_empty())
            .collect();

        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut chunk_index = 0;

        for para in paragraphs {
            let trimmed = para.trim();

            if !current_chunk.is_empty()
                && current_chunk.len() + trimmed.len() + 2 > self.chunk_size
            {
                chunks.push(DocumentChunk {
                    id: Uuid::new_v4(),
                    document_id: *document_id,
                    content: current_chunk.clone(),
                    chunk_index,
                    metadata: ChunkMetadata::default(),
                });
                current_chunk.clear();
                chunk_index += 1;
            }

            if !current_chunk.is_empty() {
                current_chunk.push_str("\n\n");
            }
            current_chunk.push_str(trimmed);
        }

        if !current_chunk.is_empty() {
            chunks.push(DocumentChunk {
                id: Uuid::new_v4(),
                document_id: *document_id,
                content: current_chunk,
                chunk_index,
                metadata: ChunkMetadata::default(),
            });
        }

        chunks
    }
}
