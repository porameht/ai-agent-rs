use deadpool_redis::{redis::AsyncCommands, Config, Connection, Pool, Runtime};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use ai_agent::application::RagService;
use ai_agent::infrastructure::{
    keys, queues, ChatAgent, EmbedDocumentJob, IndexDocumentJob, JobResult, ProcessChatJob,
    QdrantVectorStore, QueueJobStatus, TextEmbedding, RESULT_TTL_SECONDS,
};

const EMBEDDING_DIMENSION: usize = 1536;
const COLLECTION_NAME: &str = "knowledge_base";

pub type RedisPool = Pool;

#[derive(Debug, thiserror::Error)]
pub enum WorkerError {
    #[error("Redis pool error: {0}")]
    Pool(String),
    #[error("Redis error: {0}")]
    Redis(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Processing error: {0}")]
    Processing(String),
}

pub type Result<T> = std::result::Result<T, WorkerError>;

pub fn create_pool(redis_url: &str) -> Result<RedisPool> {
    let cfg = Config::from_url(redis_url);
    cfg.create_pool(Some(Runtime::Tokio1))
        .map_err(|e| WorkerError::Pool(e.to_string()))
}

pub struct WorkerState {
    pub redis_pool: RedisPool,
    pub agent: Arc<ChatAgent>,
    pub rag: Arc<RagService>,
}

impl WorkerState {
    pub async fn new(redis_pool: RedisPool, qdrant_url: &str) -> anyhow::Result<Self> {
        let embedding = Arc::new(TextEmbedding::new());
        let vector_store = Arc::new(
            QdrantVectorStore::new(qdrant_url, COLLECTION_NAME, EMBEDDING_DIMENSION).await?,
        );
        let rag = Arc::new(RagService::new(embedding, vector_store, 5));
        let agent = Arc::new(ChatAgent::new(rag.clone()));

        Ok(Self {
            redis_pool,
            agent,
            rag,
        })
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

async fn conn(state: &WorkerState) -> Result<Connection> {
    state
        .redis_pool
        .get()
        .await
        .map_err(|e| WorkerError::Pool(e.to_string()))
}

async fn set_status(conn: &mut Connection, job_id: uuid::Uuid, status: &JobResult) -> Result<()> {
    let json = serde_json::to_string(status)?;
    conn.set_ex::<_, _, ()>(keys::job_status(&job_id), &json, RESULT_TTL_SECONDS)
        .await
        .map_err(|e| WorkerError::Redis(e.to_string()))
}

async fn process_next_job(state: &WorkerState) -> Result<()> {
    let mut c = conn(state).await?;

    let result: Option<(String, String)> = c
        .brpop(
            &[queues::CHAT_QUEUE, queues::EMBED_QUEUE, queues::INDEX_QUEUE],
            1.0,
        )
        .await
        .map_err(|e| WorkerError::Redis(e.to_string()))?;

    if let Some((queue, job_json)) = result {
        match queue.as_str() {
            q if q == queues::CHAT_QUEUE => {
                process_chat_job(state, serde_json::from_str(&job_json)?).await?;
            }
            q if q == queues::EMBED_QUEUE => {
                process_embed_job(state, serde_json::from_str(&job_json)?).await?;
            }
            q if q == queues::INDEX_QUEUE => {
                process_index_job(state, serde_json::from_str(&job_json)?).await?;
            }
            _ => tracing::warn!(queue, "unknown queue"),
        }
    }
    Ok(())
}

async fn process_chat_job(state: &WorkerState, job: ProcessChatJob) -> Result<()> {
    tracing::info!(job_id = %job.job_id, "processing chat");
    let mut c = conn(state).await?;

    set_status(
        &mut c,
        job.job_id,
        &JobResult {
            job_id: job.job_id,
            status: QueueJobStatus::Processing,
            result: None,
            error: None,
            completed_at: None,
        },
    )
    .await?;

    let response = state.agent.chat(&job.message).await;

    match response {
        Ok(result) => {
            set_status(
                &mut c,
                job.job_id,
                &JobResult::completed(
                    job.job_id,
                    serde_json::json!({
                        "response": result,
                        "conversation_id": job.conversation_id,
                    }),
                ),
            )
            .await?;
        }
        Err(e) => {
            set_status(
                &mut c,
                job.job_id,
                &JobResult::failed(job.job_id, e.to_string()),
            )
            .await?;
        }
    }

    tracing::info!(job_id = %job.job_id, "chat completed");
    Ok(())
}

async fn process_embed_job(state: &WorkerState, job: EmbedDocumentJob) -> Result<()> {
    tracing::info!(job_id = %job.job_id, document_id = %job.document_id, "processing embed");
    let mut c = conn(state).await?;

    // TODO: Implement embedding pipeline
    set_status(
        &mut c,
        job.job_id,
        &JobResult::completed(
            job.job_id,
            serde_json::json!({ "document_id": job.document_id, "chunks_created": 0 }),
        ),
    )
    .await?;

    tracing::info!(job_id = %job.job_id, "embed completed");
    Ok(())
}

async fn process_index_job(state: &WorkerState, job: IndexDocumentJob) -> Result<()> {
    tracing::info!(job_id = %job.job_id, document_id = %job.document_id, "processing index");
    let mut c = conn(state).await?;

    // TODO: Implement indexing pipeline
    set_status(
        &mut c,
        job.job_id,
        &JobResult::completed(
            job.job_id,
            serde_json::json!({ "document_id": job.document_id, "indexed": true }),
        ),
    )
    .await?;

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

    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".into());
    let qdrant_url = std::env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6334".into());

    let redis_pool = create_pool(&redis_url)?;
    info!("Redis connected");

    let concurrency: usize = std::env::var("WORKER_CONCURRENCY")
        .unwrap_or_else(|_| "4".into())
        .parse()
        .unwrap_or(4);

    let state = WorkerState::new(redis_pool, &qdrant_url).await?;
    info!("Qdrant connected");

    let consumer = JobConsumer::new(state, concurrency);

    info!(concurrency, "worker started");
    consumer.start().await?;

    Ok(())
}
