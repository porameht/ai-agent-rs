use deadpool_redis::{redis::AsyncCommands, Config as RedisConfig, Connection, Pool, Runtime};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

use ai_agent::application::RagService;
use ai_agent::domain::{chunk_content, Conversation, Message, MessageRole};
use ai_agent::infrastructure::{
    keys, queues, AppConfig, ChatAgent, EmbedDocumentJob, IndexDocumentJob, JobResult,
    ProcessChatJob, QdrantVectorStore, TextEmbedding,
};

pub type RedisPool = Pool;

#[derive(Debug, thiserror::Error)]
pub enum WorkerError {
    #[error("Redis pool error: {0}")]
    Pool(String),
    #[error("Redis error: {0}")]
    Redis(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, WorkerError>;

pub fn create_pool(redis_url: &str) -> Result<RedisPool> {
    let cfg = RedisConfig::from_url(redis_url);
    cfg.create_pool(Some(Runtime::Tokio1))
        .map_err(|e| WorkerError::Pool(e.to_string()))
}

pub struct WorkerState {
    pub redis_pool: RedisPool,
    pub agent: Arc<ChatAgent>,
    pub rag: Arc<RagService>,
    pub config: Arc<AppConfig>,
}

impl WorkerState {
    pub async fn new(
        redis_pool: RedisPool,
        qdrant_url: &str,
        config: AppConfig,
    ) -> anyhow::Result<Self> {
        let config = Arc::new(config);

        let embedding = Arc::new(TextEmbedding::from_config(&config.config.embedding));
        let vector_store = Arc::new(
            QdrantVectorStore::new(
                qdrant_url,
                &config.config.vector_store.collection,
                config.config.embedding.dimension,
            )
            .await?,
        );
        let rag = Arc::new(RagService::new(
            embedding,
            vector_store,
            config.config.rag.top_k,
        ));
        let agent = Arc::new(ChatAgent::new(rag.clone(), &config));

        Ok(Self {
            redis_pool,
            agent,
            rag,
            config,
        })
    }

    async fn get_connection(&self) -> Result<Connection> {
        self.redis_pool
            .get()
            .await
            .map_err(|e| WorkerError::Pool(e.to_string()))
    }
}

pub struct JobConsumer {
    state: Arc<WorkerState>,
    concurrency: usize,
}

impl JobConsumer {
    pub fn new(state: WorkerState, concurrency: usize) -> Self {
        Self {
            state: Arc::new(state),
            concurrency,
        }
    }

    pub async fn start(&self) -> Result<()> {
        let semaphore = Arc::new(Semaphore::new(self.concurrency));
        tracing::info!(concurrency = self.concurrency, "consumer started");

        loop {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let state = self.state.clone();

            tokio::spawn(async move {
                let _permit = permit;
                if let Err(e) = process_next_job(&state).await {
                    tracing::error!(error = %e, "job failed");
                }
            });

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }
}

async fn set_job_status(
    conn: &mut Connection,
    job_id: Uuid,
    status: &JobResult,
    ttl: u64,
) -> Result<()> {
    let json = serde_json::to_string(status)?;
    conn.set_ex::<_, _, ()>(keys::job_status(&job_id), &json, ttl)
        .await
        .map_err(|e| WorkerError::Redis(e.to_string()))
}

async fn process_next_job(state: &WorkerState) -> Result<()> {
    let mut conn = state.get_connection().await?;

    let result: Option<(String, String)> = conn
        .brpop(
            &[queues::CHAT_QUEUE, queues::EMBED_QUEUE, queues::INDEX_QUEUE],
            1.0,
        )
        .await
        .map_err(|e| WorkerError::Redis(e.to_string()))?;

    if let Some((queue, job_json)) = result {
        match queue.as_str() {
            queues::CHAT_QUEUE => {
                process_chat_job(state, serde_json::from_str(&job_json)?).await?;
            }
            queues::EMBED_QUEUE => {
                process_embed_job(state, serde_json::from_str(&job_json)?).await?;
            }
            queues::INDEX_QUEUE => {
                process_index_job(state, serde_json::from_str(&job_json)?).await?;
            }
            _ => tracing::warn!(queue, "unknown queue"),
        }
    }
    Ok(())
}

async fn process_chat_job(state: &WorkerState, job: ProcessChatJob) -> Result<()> {
    tracing::info!(job_id = %job.job_id, conversation_id = ?job.conversation_id, "processing chat");
    let mut conn = state.get_connection().await?;
    let result_ttl = state.config.config.worker.result_ttl_seconds;
    let conv_ttl = state.config.config.worker.conversation_ttl_seconds;

    set_job_status(
        &mut conn,
        job.job_id,
        &JobResult::processing(job.job_id),
        result_ttl,
    )
    .await?;

    let conversation_id = job.conversation_id.unwrap_or_else(Uuid::new_v4);
    let mut conversation = load_conversation(&mut conn, &conversation_id).await?;

    conversation.add_message(MessageRole::User, &job.message);

    // Get history excluding the message we just added
    let history: Vec<Message> = conversation
        .messages
        .iter()
        .take(conversation.messages.len().saturating_sub(1))
        .cloned()
        .collect();

    let response = state.agent.chat_with_history(&job.message, &history).await;

    match response {
        Ok(result) => {
            conversation.add_message(MessageRole::Assistant, &result);
            save_conversation(&mut conn, &conversation_id, &conversation, conv_ttl).await?;

            set_job_status(
                &mut conn,
                job.job_id,
                &JobResult::completed(
                    job.job_id,
                    serde_json::json!({
                        "response": result,
                        "conversation_id": conversation_id,
                    }),
                ),
                result_ttl,
            )
            .await?;
        }
        Err(e) => {
            set_job_status(
                &mut conn,
                job.job_id,
                &JobResult::failed(job.job_id, e.to_string()),
                result_ttl,
            )
            .await?;
        }
    }

    tracing::info!(job_id = %job.job_id, "chat completed");
    Ok(())
}

async fn load_conversation(conn: &mut Connection, id: &Uuid) -> Result<Conversation> {
    let key = keys::conversation(id);
    let data: Option<String> = conn
        .get(&key)
        .await
        .map_err(|e| WorkerError::Redis(e.to_string()))?;

    match data {
        Some(json) => serde_json::from_str(&json).map_err(WorkerError::from),
        None => Ok(Conversation::new()),
    }
}

async fn save_conversation(
    conn: &mut Connection,
    id: &Uuid,
    conv: &Conversation,
    ttl: u64,
) -> Result<()> {
    let key = keys::conversation(id);
    let json = serde_json::to_string(conv)?;
    conn.set_ex::<_, _, ()>(&key, &json, ttl)
        .await
        .map_err(|e| WorkerError::Redis(e.to_string()))
}

async fn process_embed_job(state: &WorkerState, job: EmbedDocumentJob) -> Result<()> {
    tracing::info!(job_id = %job.job_id, document_id = %job.document_id, "processing embed");
    let mut conn = state.get_connection().await?;
    let result_ttl = state.config.config.worker.result_ttl_seconds;
    let chunk_size = state.config.config.rag.chunk_size;

    set_job_status(
        &mut conn,
        job.job_id,
        &JobResult::processing(job.job_id),
        result_ttl,
    )
    .await?;

    let chunks = chunk_content(job.document_id, &job.content, chunk_size);

    let result = if chunks.is_empty() {
        JobResult::completed(
            job.job_id,
            serde_json::json!({ "document_id": job.document_id, "chunks_created": 0 }),
        )
    } else {
        match state.rag.index_chunks(&chunks).await {
            Ok(()) => JobResult::completed(
                job.job_id,
                serde_json::json!({
                    "document_id": job.document_id,
                    "chunks_created": chunks.len()
                }),
            ),
            Err(e) => JobResult::failed(job.job_id, e.to_string()),
        }
    };

    set_job_status(&mut conn, job.job_id, &result, result_ttl).await?;
    tracing::info!(job_id = %job.job_id, chunks = chunks.len(), "embed completed");
    Ok(())
}

async fn process_index_job(state: &WorkerState, job: IndexDocumentJob) -> Result<()> {
    tracing::info!(job_id = %job.job_id, document_id = %job.document_id, "processing index");
    let mut conn = state.get_connection().await?;
    let result_ttl = state.config.config.worker.result_ttl_seconds;

    set_job_status(
        &mut conn,
        job.job_id,
        &JobResult::processing(job.job_id),
        result_ttl,
    )
    .await?;

    let result = match state.rag.delete_document(job.document_id).await {
        Ok(()) => JobResult::completed(
            job.job_id,
            serde_json::json!({
                "document_id": job.document_id,
                "indexed": true,
                "action": "cleared_vectors"
            }),
        ),
        Err(e) => JobResult::failed(job.job_id, e.to_string()),
    };

    set_job_status(&mut conn, job.job_id, &result, result_ttl).await?;
    tracing::info!(job_id = %job.job_id, "index completed");
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "worker=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();

    let config = AppConfig::load().unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to load config, using defaults");
        AppConfig::default()
    });

    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".into());
    let qdrant_url = std::env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6334".into());

    let redis_pool = create_pool(&redis_url)?;
    info!("Redis connected");

    let concurrency = std::env::var("WORKER_CONCURRENCY")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(config.config.worker.concurrency);

    let state = WorkerState::new(redis_pool, &qdrant_url, config).await?;
    info!("Qdrant connected");

    let consumer = JobConsumer::new(state, concurrency);

    info!(concurrency, "worker started");
    consumer.start().await?;

    Ok(())
}
