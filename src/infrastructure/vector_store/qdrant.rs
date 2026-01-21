use async_trait::async_trait;
use qdrant_client::qdrant::{
    Condition, CreateCollectionBuilder, DeletePointsBuilder, Distance, Filter, PointStruct,
    SearchPointsBuilder, UpsertPointsBuilder, VectorParamsBuilder,
};
use qdrant_client::{Payload, Qdrant};
use uuid::Uuid;

use crate::domain::{ports::VectorStore, DocumentChunk, DomainError, Embedding, SearchResult};

pub struct QdrantVectorStore {
    client: Qdrant,
    collection: String,
    dimension: usize,
}

impl QdrantVectorStore {
    pub async fn new(url: &str, collection: &str, dimension: usize) -> Result<Self, DomainError> {
        let client = Qdrant::from_url(url)
            .build()
            .map_err(|e| DomainError::external(e.to_string()))?;

        let store = Self {
            client,
            collection: collection.to_string(),
            dimension,
        };

        store.ensure_collection().await?;

        Ok(store)
    }

    async fn ensure_collection(&self) -> Result<(), DomainError> {
        let collections = self
            .client
            .list_collections()
            .await
            .map_err(|e| DomainError::external(e.to_string()))?;

        let exists = collections
            .collections
            .iter()
            .any(|c| c.name == self.collection);

        if !exists {
            self.client
                .create_collection(
                    CreateCollectionBuilder::new(&self.collection)
                        .vectors_config(VectorParamsBuilder::new(
                            self.dimension as u64,
                            Distance::Cosine,
                        )),
                )
                .await
                .map_err(|e| DomainError::external(e.to_string()))?;
        }

        Ok(())
    }

    fn uuid_to_point_id(id: Uuid) -> u64 {
        let bytes = id.as_bytes();
        u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ])
    }
}

#[async_trait]
impl VectorStore for QdrantVectorStore {
    async fn upsert(&self, chunk: &DocumentChunk, embedding: &Embedding) -> Result<(), DomainError> {
        let payload: Payload = serde_json::json!({
            "chunk_id": chunk.id.to_string(),
            "document_id": chunk.document_id.to_string(),
            "content": chunk.content,
            "chunk_index": chunk.chunk_index,
        })
        .try_into()
        .map_err(|_| DomainError::internal("Failed to create payload"))?;

        let point = PointStruct::new(
            Self::uuid_to_point_id(chunk.id),
            embedding.as_slice().to_vec(),
            payload,
        );

        self.client
            .upsert_points(UpsertPointsBuilder::new(&self.collection, vec![point]))
            .await
            .map_err(|e| DomainError::external(e.to_string()))?;

        Ok(())
    }

    async fn search(&self, query: &Embedding, top_k: usize) -> Result<Vec<SearchResult>, DomainError> {
        let results = self
            .client
            .search_points(
                SearchPointsBuilder::new(&self.collection, query.as_slice().to_vec(), top_k as u64)
                    .with_payload(true),
            )
            .await
            .map_err(|e| DomainError::external(e.to_string()))?;

        let search_results: Vec<SearchResult> = results
            .result
            .into_iter()
            .filter_map(|point| {
                let payload = point.payload;

                let chunk_id: Uuid = payload
                    .get("chunk_id")?
                    .as_str()?
                    .parse()
                    .ok()?;
                let document_id: Uuid = payload
                    .get("document_id")?
                    .as_str()?
                    .parse()
                    .ok()?;
                let content = payload.get("content")?.as_str()?.to_string();
                let chunk_index = payload.get("chunk_index")?.as_integer()? as usize;

                let chunk = DocumentChunk {
                    id: chunk_id,
                    document_id,
                    content,
                    chunk_index,
                    metadata: Default::default(),
                };

                Some(SearchResult {
                    chunk,
                    score: point.score,
                })
            })
            .collect();

        Ok(search_results)
    }

    async fn delete_by_document(&self, document_id: Uuid) -> Result<(), DomainError> {
        let filter = Filter::must([Condition::matches(
            "document_id",
            document_id.to_string(),
        )]);

        self.client
            .delete_points(
                DeletePointsBuilder::new(&self.collection)
                    .points(filter),
            )
            .await
            .map_err(|e| DomainError::external(e.to_string()))?;

        Ok(())
    }
}
