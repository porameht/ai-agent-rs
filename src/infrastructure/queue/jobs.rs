use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod queues {
    pub const CHAT_QUEUE: &str = "jobs:chat";
    pub const EMBED_QUEUE: &str = "jobs:embed";
    pub const INDEX_QUEUE: &str = "jobs:index";
}

pub mod keys {
    use uuid::Uuid;

    pub fn job_status(job_id: &Uuid) -> String {
        format!("job:status:{}", job_id)
    }

    pub fn conversation(conversation_id: &Uuid) -> String {
        format!("conversation:{}", conversation_id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueueJobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResult {
    pub job_id: Uuid,
    pub status: QueueJobStatus,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl JobResult {
    pub fn pending(job_id: Uuid) -> Self {
        Self {
            job_id,
            status: QueueJobStatus::Pending,
            result: None,
            error: None,
            completed_at: None,
        }
    }

    pub fn processing(job_id: Uuid) -> Self {
        Self {
            job_id,
            status: QueueJobStatus::Processing,
            result: None,
            error: None,
            completed_at: None,
        }
    }

    pub fn completed(job_id: Uuid, result: serde_json::Value) -> Self {
        Self {
            job_id,
            status: QueueJobStatus::Completed,
            result: Some(result),
            error: None,
            completed_at: Some(Utc::now()),
        }
    }

    pub fn failed(job_id: Uuid, error: impl Into<String>) -> Self {
        Self {
            job_id,
            status: QueueJobStatus::Failed,
            result: None,
            error: Some(error.into()),
            completed_at: Some(Utc::now()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessChatJob {
    pub job_id: Uuid,
    pub message: String,
    pub conversation_id: Option<Uuid>,
    pub agent_id: Option<String>,
}

impl ProcessChatJob {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            job_id: Uuid::new_v4(),
            message: message.into(),
            conversation_id: None,
            agent_id: None,
        }
    }

    pub fn with_conversation(mut self, conversation_id: Uuid) -> Self {
        self.conversation_id = Some(conversation_id);
        self
    }

    pub fn with_agent(mut self, agent_id: impl Into<String>) -> Self {
        self.agent_id = Some(agent_id.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedDocumentJob {
    pub job_id: Uuid,
    pub document_id: Uuid,
    pub content: String,
    pub metadata: serde_json::Value,
}

impl EmbedDocumentJob {
    pub fn new(document_id: Uuid, content: impl Into<String>) -> Self {
        Self {
            job_id: Uuid::new_v4(),
            document_id,
            content: content.into(),
            metadata: serde_json::json!({}),
        }
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexDocumentJob {
    pub job_id: Uuid,
    pub document_id: Uuid,
}

impl IndexDocumentJob {
    pub fn new(document_id: Uuid) -> Self {
        Self {
            job_id: Uuid::new_v4(),
            document_id,
        }
    }
}
