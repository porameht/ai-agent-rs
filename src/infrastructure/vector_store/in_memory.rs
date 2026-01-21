use async_trait::async_trait;
use std::sync::RwLock;
use uuid::Uuid;

use crate::domain::{ports::VectorStore, DocumentChunk, DomainError, Embedding, SearchResult};

pub struct InMemoryVectorStore {
    chunks: RwLock<Vec<(DocumentChunk, Embedding)>>,
}

impl InMemoryVectorStore {
    pub fn new() -> Self {
        Self {
            chunks: RwLock::new(Vec::new()),
        }
    }
}

impl Default for InMemoryVectorStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VectorStore for InMemoryVectorStore {
    async fn upsert(
        &self,
        chunk: &DocumentChunk,
        embedding: &Embedding,
    ) -> Result<(), DomainError> {
        let mut store = self
            .chunks
            .write()
            .map_err(|e| DomainError::internal(e.to_string()))?;

        store.retain(|(c, _)| c.id != chunk.id);
        store.push((chunk.clone(), embedding.clone()));
        Ok(())
    }

    async fn search(
        &self,
        query: &Embedding,
        top_k: usize,
    ) -> Result<Vec<SearchResult>, DomainError> {
        let store = self
            .chunks
            .read()
            .map_err(|e| DomainError::internal(e.to_string()))?;

        let mut results: Vec<(SearchResult, f32)> = store
            .iter()
            .map(|(chunk, embedding)| {
                let score = query.cosine_similarity(embedding);
                (
                    SearchResult {
                        chunk: chunk.clone(),
                        score,
                    },
                    score,
                )
            })
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        Ok(results.into_iter().take(top_k).map(|(r, _)| r).collect())
    }

    async fn delete_by_document(&self, document_id: Uuid) -> Result<(), DomainError> {
        let mut store = self
            .chunks
            .write()
            .map_err(|e| DomainError::internal(e.to_string()))?;

        store.retain(|(chunk, _)| chunk.document_id != document_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_upsert_and_search() {
        let store = InMemoryVectorStore::new();
        let doc_id = Uuid::new_v4();

        let chunk = DocumentChunk::new(doc_id, "test content", 0);
        let embedding = Embedding::new(vec![1.0, 0.0, 0.0]);

        store.upsert(&chunk, &embedding).await.unwrap();

        let query = Embedding::new(vec![1.0, 0.0, 0.0]);
        let results = store.search(&query, 1).await.unwrap();

        assert_eq!(results.len(), 1);
        assert!((results[0].score - 1.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_delete_by_document() {
        let store = InMemoryVectorStore::new();
        let doc_id = Uuid::new_v4();

        let chunk = DocumentChunk::new(doc_id, "test", 0);
        let embedding = Embedding::new(vec![1.0, 0.0, 0.0]);

        store.upsert(&chunk, &embedding).await.unwrap();
        store.delete_by_document(doc_id).await.unwrap();

        let query = Embedding::new(vec![1.0, 0.0, 0.0]);
        let results = store.search(&query, 10).await.unwrap();

        assert!(results.is_empty());
    }
}
