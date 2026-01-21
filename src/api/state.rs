use std::sync::Arc;

use crate::api::queue::{JobProducer, RedisPool};
use crate::application::{DocumentService, RagService};
use crate::infrastructure::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub redis_pool: RedisPool,
    pub job_producer: JobProducer,
    pub document_service: Option<Arc<DocumentService>>,
    pub rag_service: Option<Arc<RagService>>,
    pub config: Arc<AppConfig>,
}

impl AppState {
    pub fn new(redis_pool: RedisPool, config: AppConfig) -> Self {
        let config = Arc::new(config);
        let job_producer =
            JobProducer::new(redis_pool.clone(), config.config.worker.result_ttl_seconds);
        Self {
            redis_pool,
            job_producer,
            document_service: None,
            rag_service: None,
            config,
        }
    }

    pub fn with_document_service(mut self, service: Arc<DocumentService>) -> Self {
        self.document_service = Some(service);
        self
    }

    pub fn with_rag_service(mut self, service: Arc<RagService>) -> Self {
        self.rag_service = Some(service);
        self
    }
}
