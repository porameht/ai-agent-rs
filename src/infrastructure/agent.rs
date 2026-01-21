use rig::client::{CompletionClient, ProviderClient};
use rig::completion::Prompt;
use rig::providers::anthropic;
use std::sync::Arc;

use crate::application::RagService;
use crate::domain::DomainError;

use super::tools::KnowledgeBaseTool;

const DEFAULT_MODEL: &str = "claude-sonnet-4-20250514";
const DEFAULT_PREAMBLE: &str = "You are a helpful assistant. Use the knowledge_base tool to search for relevant information when needed.";

pub struct ChatAgent {
    model: String,
    preamble: String,
    rag: Arc<RagService>,
    top_k: usize,
}

impl ChatAgent {
    pub fn new(rag: Arc<RagService>) -> Self {
        Self {
            model: DEFAULT_MODEL.to_string(),
            preamble: DEFAULT_PREAMBLE.to_string(),
            rag,
            top_k: 5,
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub fn with_preamble(mut self, preamble: impl Into<String>) -> Self {
        self.preamble = preamble.into();
        self
    }

    pub fn with_top_k(mut self, top_k: usize) -> Self {
        self.top_k = top_k;
        self
    }

    pub async fn chat(&self, message: &str) -> Result<String, DomainError> {
        let client = anthropic::Client::from_env();
        let tool = KnowledgeBaseTool::new(self.rag.clone(), self.top_k);

        let agent = client
            .agent(&self.model)
            .preamble(&self.preamble)
            .tool(tool)
            .build();

        agent
            .prompt(message)
            .await
            .map_err(|e| DomainError::external(e.to_string()))
    }

    pub async fn chat_multi_turn(&self, message: &str, max_turns: usize) -> Result<String, DomainError> {
        let client = anthropic::Client::from_env();
        let tool = KnowledgeBaseTool::new(self.rag.clone(), self.top_k);

        let agent = client
            .agent(&self.model)
            .preamble(&self.preamble)
            .tool(tool)
            .build();

        agent
            .prompt(message)
            .multi_turn(max_turns)
            .await
            .map_err(|e| DomainError::external(e.to_string()))
    }
}
