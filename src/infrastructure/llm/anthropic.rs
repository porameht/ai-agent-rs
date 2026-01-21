use async_trait::async_trait;
use rig::client::{CompletionClient, ProviderClient};
use rig::completion::Prompt;
use rig::providers::anthropic;

use crate::domain::{ports::LlmService, DomainError};

pub struct AnthropicLlm {
    model: String,
}

impl AnthropicLlm {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
        }
    }

    pub fn default_model() -> Self {
        Self::new("claude-3-opus-20240229")
    }
}

#[async_trait]
impl LlmService for AnthropicLlm {
    async fn complete(&self, prompt: &str) -> Result<String, DomainError> {
        let client = anthropic::Client::from_env();
        let agent = client.agent(&self.model).build();
        agent
            .prompt(prompt)
            .await
            .map_err(|e| DomainError::external(e.to_string()))
    }

    async fn complete_with_system(
        &self,
        system: &str,
        prompt: &str,
    ) -> Result<String, DomainError> {
        let client = anthropic::Client::from_env();
        let agent = client.agent(&self.model).preamble(system).build();
        agent
            .prompt(prompt)
            .await
            .map_err(|e| DomainError::external(e.to_string()))
    }
}
