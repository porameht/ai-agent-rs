use rig::client::{CompletionClient, ProviderClient};
use rig::completion::Prompt;
use rig::providers::anthropic;
use std::sync::Arc;

use crate::application::RagService;
use crate::domain::{DomainError, Message, MessageRole};
use crate::infrastructure::config::{AppConfig, KnowledgeBaseToolConfig};

use super::tools::KnowledgeBaseTool;

pub struct ChatAgent {
    model: String,
    preamble: String,
    rag: Arc<RagService>,
    top_k: usize,
    tool_config: KnowledgeBaseToolConfig,
}

impl ChatAgent {
    pub fn new(rag: Arc<RagService>, config: &AppConfig) -> Self {
        Self {
            model: config.config.llm.model.clone(),
            preamble: config.prompts.agent.system.clone(),
            rag,
            top_k: config.config.rag.top_k,
            tool_config: config.config.tools.knowledge_base.clone(),
        }
    }

    pub fn with_defaults(rag: Arc<RagService>) -> Self {
        Self::new(rag, &AppConfig::default())
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
        self.chat_with_history(message, &[]).await
    }

    pub async fn chat_with_history(
        &self,
        message: &str,
        history: &[Message],
    ) -> Result<String, DomainError> {
        let client = anthropic::Client::from_env();
        let tool = KnowledgeBaseTool::new(self.rag.clone(), self.top_k, self.tool_config.clone());

        let agent = client
            .agent(&self.model)
            .preamble(&self.preamble)
            .tool(tool)
            .build();

        let prompt = if history.is_empty() {
            message.to_string()
        } else {
            let context = history
                .iter()
                .map(|m| {
                    let role = match m.role {
                        MessageRole::User => "User",
                        MessageRole::Assistant => "Assistant",
                        MessageRole::System => "System",
                    };
                    format!("{}: {}", role, m.content)
                })
                .collect::<Vec<_>>()
                .join("\n");
            format!(
                "Previous conversation:\n{}\n\nCurrent message from user: {}",
                context, message
            )
        };

        agent
            .prompt(&prompt)
            .await
            .map_err(|e| DomainError::external(e.to_string()))
    }

    pub async fn chat_multi_turn(
        &self,
        message: &str,
        max_turns: usize,
    ) -> Result<String, DomainError> {
        let client = anthropic::Client::from_env();
        let tool = KnowledgeBaseTool::new(self.rag.clone(), self.top_k, self.tool_config.clone());

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
