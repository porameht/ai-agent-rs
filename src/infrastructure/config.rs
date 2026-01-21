use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub llm: LlmConfig,
    pub embedding: EmbeddingConfig,
    pub database_url: String,
    pub redis_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LlmConfig {
    pub provider: String,
    pub model: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmbeddingConfig {
    pub model: String,
    pub dimension: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            llm: LlmConfig {
                provider: "openai".to_string(),
                model: "gpt-4".to_string(),
            },
            embedding: EmbeddingConfig {
                model: "text-embedding-ada-002".to_string(),
                dimension: 1536,
            },
            database_url: "postgres://localhost/ai_agent".to_string(),
            redis_url: "redis://localhost".to_string(),
        }
    }
}
