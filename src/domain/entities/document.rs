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

/// Splits content into chunks by paragraph boundaries.
///
/// Paragraphs are joined until they exceed `chunk_size`, then a new chunk starts.
/// Each chunk is assigned a sequential index starting from 0.
pub fn chunk_content(document_id: Uuid, content: &str, chunk_size: usize) -> Vec<DocumentChunk> {
    let paragraphs: Vec<&str> = content
        .split("\n\n")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();

    let mut chunks = Vec::new();
    let mut current_chunk = String::new();
    let mut chunk_index = 0;

    for paragraph in paragraphs {
        let would_exceed = !current_chunk.is_empty()
            && current_chunk.len() + paragraph.len() + 2 > chunk_size;

        if would_exceed {
            chunks.push(DocumentChunk::new(document_id, &current_chunk, chunk_index));
            current_chunk.clear();
            chunk_index += 1;
        }

        if !current_chunk.is_empty() {
            current_chunk.push_str("\n\n");
        }
        current_chunk.push_str(paragraph);
    }

    if !current_chunk.is_empty() {
        chunks.push(DocumentChunk::new(document_id, current_chunk, chunk_index));
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_content_single_chunk() {
        let doc_id = Uuid::new_v4();
        let content = "Hello world.\n\nThis is a test.";
        let chunks = chunk_content(doc_id, content, 100);

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].content, "Hello world.\n\nThis is a test.");
        assert_eq!(chunks[0].chunk_index, 0);
    }

    #[test]
    fn test_chunk_content_multiple_chunks() {
        let doc_id = Uuid::new_v4();
        let content = "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.";
        let chunks = chunk_content(doc_id, content, 30);

        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].chunk_index, 0);
        assert_eq!(chunks[1].chunk_index, 1);
        assert_eq!(chunks[2].chunk_index, 2);
    }

    #[test]
    fn test_chunk_content_empty() {
        let doc_id = Uuid::new_v4();
        let chunks = chunk_content(doc_id, "", 100);
        assert!(chunks.is_empty());
    }
}
