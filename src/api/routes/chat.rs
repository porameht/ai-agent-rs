use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::state::AppState;
use crate::infrastructure::ProcessChatJob;

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub conversation_id: Option<Uuid>,
    pub agent_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub job_id: Uuid,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct JobStatusResponse {
    pub job_id: Uuid,
    pub status: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

pub async fn chat_handler(
    State(state): State<AppState>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, StatusCode> {
    let mut job = ProcessChatJob::new(&request.message);
    if let Some(conv_id) = request.conversation_id {
        job = job.with_conversation(conv_id);
    }
    if let Some(agent_id) = request.agent_id {
        job = job.with_agent(agent_id);
    }

    let job_id = state.job_producer.push_chat_job(&job).await.map_err(|e| {
        tracing::error!(error = %e, "Failed to queue chat job");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(ChatResponse {
        job_id,
        status: "queued".to_string(),
    }))
}

pub async fn get_job_status(
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
) -> Result<Json<JobStatusResponse>, StatusCode> {
    let result = state
        .job_producer
        .get_job_status(&job_id)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to get job status");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    match result {
        Some(job_result) => Ok(Json(JobStatusResponse {
            job_id: job_result.job_id,
            status: format!("{:?}", job_result.status).to_lowercase(),
            result: job_result.result,
            error: job_result.error,
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}
