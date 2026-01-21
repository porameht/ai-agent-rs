use async_trait::async_trait;
use rig::client::{EmbeddingsClient, ProviderClient};
use rig::embeddings::EmbeddingsBuilder;
use rig::providers::openai;

use crate::domain::{ports::EmbeddingService, DomainError, Embedding};
use crate::infrastructure::config::EmbeddingConfig;

pub struct TextEmbedding {
    model: String,
    dimension: usize,
}

impl TextEmbedding {
    pub fn new() -> Self {
        Self {
            model: "text-embedding-3-small".to_string(),
            dimension: 1536,
        }
    }

    pub fn from_config(config: &EmbeddingConfig) -> Self {
        Self {
            model: config.model.clone(),
            dimension: config.dimension,
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub fn with_dimension(mut self, dimension: usize) -> Self {
        self.dimension = dimension;
        self
    }
}

impl Default for TextEmbedding {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EmbeddingService for TextEmbedding {
    async fn embed(&self, text: &str) -> Result<Embedding, DomainError> {
        let client = openai::Client::from_env();
        let model = client.embedding_model(&self.model);

        let embeddings = EmbeddingsBuilder::new(model)
            .document(text)
            .map_err(|e| DomainError::external(e.to_string()))?
            .build()
            .await
            .map_err(|e| DomainError::external(e.to_string()))?;

        embeddings
            .into_iter()
            .next()
            .map(|(_doc, emb)| {
                let vec_f32: Vec<f32> = emb.first().vec.into_iter().map(|x| x as f32).collect();
                Embedding::new(vec_f32)
            })
            .ok_or_else(|| DomainError::internal("No embedding returned"))
    }

    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Embedding>, DomainError> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let client = openai::Client::from_env();
        let model = client.embedding_model(&self.model);

        let mut builder = EmbeddingsBuilder::new(model);
        for text in texts {
            builder = builder
                .document(*text)
                .map_err(|e| DomainError::external(e.to_string()))?;
        }

        let embeddings = builder
            .build()
            .await
            .map_err(|e| DomainError::external(e.to_string()))?;

        Ok(embeddings
            .into_iter()
            .map(|(_doc, emb)| {
                let vec_f32: Vec<f32> = emb.first().vec.into_iter().map(|x| x as f32).collect();
                Embedding::new(vec_f32)
            })
            .collect())
    }

    fn dimension(&self) -> usize {
        self.dimension
    }
}
