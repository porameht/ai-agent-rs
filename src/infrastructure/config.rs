use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub llm: LlmConfig,
    pub embedding: EmbeddingConfig,
    pub vector_store: VectorStoreConfig,
    pub rag: RagConfig,
    pub worker: WorkerConfig,
    pub tools: ToolsConfig,
    #[serde(default)]
    pub cors: CorsConfig,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CorsConfig {
    #[serde(default)]
    pub allowed_origins: Vec<String>,
}

impl CorsConfig {
    pub fn is_permissive(&self) -> bool {
        self.allowed_origins.is_empty() || self.allowed_origins.iter().any(|o| o == "*")
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct LlmConfig {
    pub model: String,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: usize,
}

fn default_max_tokens() -> usize {
    4096
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmbeddingConfig {
    pub model: String,
    pub dimension: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VectorStoreConfig {
    pub collection: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RagConfig {
    pub top_k: usize,
    pub chunk_size: usize,
    #[serde(default = "default_min_score")]
    pub min_score: f32,
}

fn default_min_score() -> f32 {
    0.7
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkerConfig {
    pub concurrency: usize,
    pub conversation_ttl_seconds: u64,
    pub result_ttl_seconds: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ToolsConfig {
    pub knowledge_base: KnowledgeBaseToolConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KnowledgeBaseToolConfig {
    pub name: String,
    pub description: String,
    pub no_results_message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PromptsConfig {
    pub agent: AgentPrompts,
    pub tools: ToolPrompts,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AgentPrompts {
    pub system: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ToolPrompts {
    pub knowledge_base: KnowledgeBasePrompts,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KnowledgeBasePrompts {
    pub description: String,
    pub query_description: String,
}

#[derive(Debug, Clone, Default)]
pub struct AppConfig {
    pub config: Config,
    pub prompts: PromptsConfig,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        Self::load_from_dir("config")
    }

    pub fn load_from_dir<P: AsRef<Path>>(dir: P) -> Result<Self, ConfigError> {
        let dir = dir.as_ref();

        let config_path = dir.join("agent.yaml");
        let prompts_path = dir.join("prompts.yaml");

        let config = Self::load_yaml(&config_path)?;
        let prompts = Self::load_yaml(&prompts_path)?;

        Ok(Self { config, prompts })
    }

    fn load_yaml<T: serde::de::DeserializeOwned, P: AsRef<Path>>(
        path: P,
    ) -> Result<T, ConfigError> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::Io(path.display().to_string(), e.to_string()))?;

        serde_yaml::from_str(&content)
            .map_err(|e| ConfigError::Parse(path.display().to_string(), e.to_string()))
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            llm: LlmConfig {
                model: "claude-sonnet-4-20250514".to_string(),
                max_tokens: 4096,
            },
            embedding: EmbeddingConfig {
                model: "text-embedding-3-small".to_string(),
                dimension: 1536,
            },
            vector_store: VectorStoreConfig {
                collection: "knowledge_base".to_string(),
            },
            rag: RagConfig {
                top_k: 5,
                chunk_size: 1000,
                min_score: 0.7,
            },
            worker: WorkerConfig {
                concurrency: 4,
                conversation_ttl_seconds: 3600,
                result_ttl_seconds: 86400,
            },
            tools: ToolsConfig {
                knowledge_base: KnowledgeBaseToolConfig {
                    name: "knowledge_base".to_string(),
                    description: "Search the knowledge base for relevant information.".to_string(),
                    no_results_message: "No relevant documents found.".to_string(),
                },
            },
            cors: CorsConfig::default(),
        }
    }
}

impl Default for PromptsConfig {
    fn default() -> Self {
        Self {
            agent: AgentPrompts {
                system: "You are a helpful assistant. Use the knowledge_base tool to search for relevant information when needed.".to_string(),
            },
            tools: ToolPrompts {
                knowledge_base: KnowledgeBasePrompts {
                    description: "Search the knowledge base for relevant information.".to_string(),
                    query_description: "The search query to find relevant documents".to_string(),
                },
            },
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to read config file '{0}': {1}")]
    Io(String, String),
    #[error("Failed to parse config file '{0}': {1}")]
    Parse(String, String),
}
