use std::sync::Arc;
use tracing::instrument;

use crate::domain::{
    ports::{EmbeddingService, VectorStore},
    DocumentChunk, DomainError, SearchResult,
};

pub struct RagService {
    embedding: Arc<dyn EmbeddingService>,
    vector_store: Arc<dyn VectorStore>,
    default_top_k: usize,
}

impl RagService {
    pub fn new(
        embedding: Arc<dyn EmbeddingService>,
        vector_store: Arc<dyn VectorStore>,
        default_top_k: usize,
    ) -> Self {
        Self {
            embedding,
            vector_store,
            default_top_k,
        }
    }

    #[instrument(skip(self), fields(top_k))]
    pub async fn retrieve(&self, query: &str) -> Result<Vec<SearchResult>, DomainError> {
        self.retrieve_top_k(query, self.default_top_k).await
    }

    #[instrument(skip(self))]
    pub async fn retrieve_top_k(
        &self,
        query: &str,
        top_k: usize,
    ) -> Result<Vec<SearchResult>, DomainError> {
        let embedding = self.embedding.embed(query).await?;
        self.vector_store.search(&embedding, top_k).await
    }

    #[instrument(skip(self, chunk), fields(chunk_id = %chunk.id))]
    pub async fn index_chunk(&self, chunk: &DocumentChunk) -> Result<(), DomainError> {
        let embedding = self.embedding.embed(&chunk.content).await?;
        self.vector_store.upsert(chunk, &embedding).await
    }

    #[instrument(skip(self, chunks), fields(count = chunks.len()))]
    pub async fn index_chunks(&self, chunks: &[DocumentChunk]) -> Result<(), DomainError> {
        if chunks.is_empty() {
            return Ok(());
        }

        let texts: Vec<&str> = chunks.iter().map(|c| c.content.as_str()).collect();
        let embeddings = self.embedding.embed_batch(&texts).await?;

        for (chunk, embedding) in chunks.iter().zip(embeddings.iter()) {
            self.vector_store.upsert(chunk, embedding).await?;
        }

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn delete_document(&self, document_id: uuid::Uuid) -> Result<(), DomainError> {
        self.vector_store.delete_by_document(document_id).await
    }
}
