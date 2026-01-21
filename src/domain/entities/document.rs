use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: Uuid,
    pub name: String,
    pub content_type: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Document {
    pub fn new(name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            content_type: "text/plain".to_string(),
            metadata: serde_json::json!({}),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = content_type.into();
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChunk {
    pub id: Uuid,
    pub document_id: Uuid,
    pub content: String,
    pub chunk_index: usize,
    pub metadata: ChunkMetadata,
}

impl DocumentChunk {
    pub fn new(document_id: Uuid, content: impl Into<String>, chunk_index: usize) -> Self {
        Self {
            id: Uuid::new_v4(),
            document_id,
            content: content.into(),
            chunk_index,
            metadata: ChunkMetadata::default(),
        }
    }

    pub fn with_metadata(mut self, metadata: ChunkMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChunkMetadata {
    pub page: Option<usize>,
    pub section: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub chunk: DocumentChunk,
    pub score: f32,
}
