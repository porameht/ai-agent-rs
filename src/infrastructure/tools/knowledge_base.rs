use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

use crate::application::RagService;

#[derive(Debug, thiserror::Error)]
#[error("Knowledge base error: {0}")]
pub struct KnowledgeBaseError(pub String);

#[derive(Debug, Deserialize, Serialize)]
pub struct KnowledgeBaseArgs {
    pub query: String,
}

pub struct KnowledgeBaseTool {
    rag: Arc<RagService>,
    top_k: usize,
}

impl KnowledgeBaseTool {
    pub fn new(rag: Arc<RagService>, top_k: usize) -> Self {
        Self { rag, top_k }
    }

    pub fn with_rag(rag: Arc<RagService>) -> Self {
        Self::new(rag, 5)
    }
}

impl Tool for KnowledgeBaseTool {
    const NAME: &'static str = "knowledge_base";

    type Error = KnowledgeBaseError;
    type Args = KnowledgeBaseArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "knowledge_base".to_string(),
            description: "Search the knowledge base for relevant information.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The search query"
                    }
                },
                "required": ["query"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let results = self
            .rag
            .retrieve_top_k(&args.query, self.top_k)
            .await
            .map_err(|e| KnowledgeBaseError(e.to_string()))?;

        let output = results
            .iter()
            .enumerate()
            .map(|(i, r)| format!("[{}] {}", i + 1, r.chunk.content))
            .collect::<Vec<_>>()
            .join("\n\n");

        Ok(if output.is_empty() {
            "No relevant documents found.".to_string()
        } else {
            output
        })
    }
}
