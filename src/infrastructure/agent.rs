use rig::client::{CompletionClient, ProviderClient};
use rig::completion::Prompt;
use rig::providers::gemini;
use std::sync::Arc;

use crate::application::RagService;
use crate::domain::{DomainError, Message};
use crate::infrastructure::config::{AppConfig, KnowledgeBaseToolConfig};
use crate::infrastructure::tools::KnowledgeBaseTool;

pub struct ChatAgent {
    model: String,
    system_prompt: String,
    rag: Arc<RagService>,
    top_k: usize,
    tool_config: KnowledgeBaseToolConfig,
}

impl ChatAgent {
    pub fn new(rag: Arc<RagService>, config: &AppConfig) -> Self {
        Self {
            model: config.config.llm.model.clone(),
            system_prompt: config.prompts.agent.system.clone(),
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

    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = prompt.into();
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
        let client = gemini::Client::from_env();
        let tool = KnowledgeBaseTool::new(self.rag.clone(), self.top_k, self.tool_config.clone());

        let agent = client
            .agent(&self.model)
            .preamble(&self.system_prompt)
            .tool(tool)
            .build();

        let prompt = self.build_prompt(message, history);

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
        let client = gemini::Client::from_env();
        let tool = KnowledgeBaseTool::new(self.rag.clone(), self.top_k, self.tool_config.clone());

        let agent = client
            .agent(&self.model)
            .preamble(&self.system_prompt)
            .tool(tool)
            .build();

        agent
            .prompt(message)
            .multi_turn(max_turns)
            .await
            .map_err(|e| DomainError::external(e.to_string()))
    }

    fn build_prompt(&self, message: &str, history: &[Message]) -> String {
        if history.is_empty() {
            return message.to_string();
        }

        let context = history
            .iter()
            .map(|m| format!("{}: {}", m.role.as_str(), m.content))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "Previous conversation:\n{}\n\nCurrent message from user: {}",
            context, message
        )
    }
}
