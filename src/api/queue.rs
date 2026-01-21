use deadpool_redis::{redis::AsyncCommands, Config, Pool, Runtime};
use uuid::Uuid;

use crate::infrastructure::{
    keys, queues, EmbedDocumentJob, IndexDocumentJob, JobResult, ProcessChatJob, RESULT_TTL_SECONDS,
};

pub type RedisPool = Pool;

#[derive(Debug, thiserror::Error)]
pub enum QueueError {
    #[error("Redis pool error: {0}")]
    Pool(String),
    #[error("Redis error: {0}")]
    Redis(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, QueueError>;

pub fn create_pool(redis_url: &str) -> Result<RedisPool> {
    let cfg = Config::from_url(redis_url);
    cfg.create_pool(Some(Runtime::Tokio1))
        .map_err(|e| QueueError::Pool(e.to_string()))
}

#[derive(Clone)]
pub struct JobProducer {
    pool: RedisPool,
}

impl JobProducer {
    pub fn new(pool: RedisPool) -> Self {
        Self { pool }
    }

    async fn conn(&self) -> Result<deadpool_redis::Connection> {
        self.pool
            .get()
            .await
            .map_err(|e| QueueError::Pool(e.to_string()))
    }

    async fn push_job(&self, queue: &str, job_id: Uuid, payload: &str) -> Result<Uuid> {
        let mut conn = self.conn().await?;

        conn.lpush::<_, _, ()>(queue, payload)
            .await
            .map_err(|e| QueueError::Redis(e.to_string()))?;

        let status = serde_json::to_string(&JobResult::pending(job_id))?;
        conn.set_ex::<_, _, ()>(keys::job_status(&job_id), &status, RESULT_TTL_SECONDS)
            .await
            .map_err(|e| QueueError::Redis(e.to_string()))?;

        tracing::info!(job_id = %job_id, queue, "job queued");
        Ok(job_id)
    }

    pub async fn push_chat_job(&self, job: &ProcessChatJob) -> Result<Uuid> {
        self.push_job(queues::CHAT_QUEUE, job.job_id, &serde_json::to_string(job)?)
            .await
    }

    pub async fn push_embed_job(&self, job: &EmbedDocumentJob) -> Result<Uuid> {
        self.push_job(
            queues::EMBED_QUEUE,
            job.job_id,
            &serde_json::to_string(job)?,
        )
        .await
    }

    pub async fn push_index_job(&self, job: &IndexDocumentJob) -> Result<Uuid> {
        self.push_job(
            queues::INDEX_QUEUE,
            job.job_id,
            &serde_json::to_string(job)?,
        )
        .await
    }

    pub async fn get_job_status(&self, job_id: &Uuid) -> Result<Option<JobResult>> {
        let mut conn = self.conn().await?;
        let result: Option<String> = conn
            .get(keys::job_status(job_id))
            .await
            .map_err(|e| QueueError::Redis(e.to_string()))?;

        result
            .map(|json| serde_json::from_str(&json).map_err(Into::into))
            .transpose()
    }
}
