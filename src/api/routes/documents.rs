use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::state::AppState;
use crate::domain::Document;

#[derive(Debug, Deserialize)]
pub struct CreateDocumentRequest {
    pub name: String,
    pub content: String,
    #[allow(dead_code)]
    pub content_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DocumentResponse {
    pub id: Uuid,
    pub name: String,
    pub content_type: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<Document> for DocumentResponse {
    fn from(doc: Document) -> Self {
        Self {
            id: doc.id,
            name: doc.name,
            content_type: doc.content_type,
            created_at: doc.created_at,
            updated_at: doc.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListDocumentsQuery {
    #[allow(dead_code)]
    pub limit: Option<i64>,
    #[allow(dead_code)]
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct SearchDocumentsRequest {
    pub query: String,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct SearchResultResponse {
    pub chunk_id: Uuid,
    pub document_id: Uuid,
    pub content: String,
    pub score: f32,
}

pub async fn create_document(
    State(state): State<AppState>,
    Json(request): Json<CreateDocumentRequest>,
) -> Result<Json<DocumentResponse>, StatusCode> {
    let Some(doc_service) = &state.document_service else {
        let doc = Document::new(&request.name);
        return Ok(Json(DocumentResponse::from(doc)));
    };

    doc_service
        .ingest(&request.name, &request.content)
        .await
        .map(|(doc, _)| Json(DocumentResponse::from(doc)))
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to create document");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

pub async fn get_document(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<DocumentResponse>, StatusCode> {
    let Some(doc_service) = &state.document_service else {
        return Err(StatusCode::NOT_FOUND);
    };

    match doc_service.get(id).await {
        Ok(Some(doc)) => Ok(Json(DocumentResponse::from(doc))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!(error = %e, "Failed to get document");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn list_documents(
    State(_state): State<AppState>,
    Query(_query): Query<ListDocumentsQuery>,
) -> Result<Json<Vec<DocumentResponse>>, StatusCode> {
    // TODO: Implement document listing with document store
    Ok(Json(vec![]))
}

pub async fn delete_document(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let Some(doc_service) = &state.document_service else {
        return Err(StatusCode::NOT_FOUND);
    };

    doc_service.delete(id).await.map_err(|e| {
        tracing::error!(error = %e, "Failed to delete document");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn search_documents(
    State(state): State<AppState>,
    Json(request): Json<SearchDocumentsRequest>,
) -> Result<Json<Vec<SearchResultResponse>>, StatusCode> {
    let Some(rag_service) = &state.rag_service else {
        return Ok(Json(vec![]));
    };

    let top_k = request.limit.unwrap_or(5);
    rag_service
        .retrieve_top_k(&request.query, top_k)
        .await
        .map(|results| {
            Json(
                results
                    .into_iter()
                    .map(|r| SearchResultResponse {
                        chunk_id: r.chunk.id,
                        document_id: r.chunk.document_id,
                        content: r.chunk.content,
                        score: r.score,
                    })
                    .collect(),
            )
        })
        .map_err(|e| {
            tracing::error!(error = %e, "Search failed");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}
