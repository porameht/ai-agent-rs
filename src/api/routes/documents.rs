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
    pub limit: Option<i64>,
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
    // Use document service if available
    if let Some(doc_service) = &state.document_service {
        match doc_service.ingest(&request.name, &request.content).await {
            Ok((doc, _chunks)) => {
                return Ok(Json(DocumentResponse::from(doc)));
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to create document");
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }

    // Fallback: create document without persistence
    let doc = Document::new(&request.name);
    Ok(Json(DocumentResponse::from(doc)))
}

pub async fn get_document(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<DocumentResponse>, StatusCode> {
    if let Some(doc_service) = &state.document_service {
        match doc_service.get(id).await {
            Ok(Some(doc)) => return Ok(Json(DocumentResponse::from(doc))),
            Ok(None) => return Err(StatusCode::NOT_FOUND),
            Err(e) => {
                tracing::error!(error = %e, "Failed to get document");
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }

    Err(StatusCode::NOT_FOUND)
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
    if let Some(doc_service) = &state.document_service {
        match doc_service.delete(id).await {
            Ok(()) => return Ok(StatusCode::NO_CONTENT),
            Err(e) => {
                tracing::error!(error = %e, "Failed to delete document");
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }

    Err(StatusCode::NOT_FOUND)
}

pub async fn search_documents(
    State(state): State<AppState>,
    Json(request): Json<SearchDocumentsRequest>,
) -> Result<Json<Vec<SearchResultResponse>>, StatusCode> {
    if let Some(rag_service) = &state.rag_service {
        let top_k = request.limit.unwrap_or(5);
        match rag_service.retrieve_top_k(&request.query, top_k).await {
            Ok(results) => {
                return Ok(Json(
                    results
                        .into_iter()
                        .map(|r| SearchResultResponse {
                            chunk_id: r.chunk.id,
                            document_id: r.chunk.document_id,
                            content: r.chunk.content,
                            score: r.score,
                        })
                        .collect(),
                ));
            }
            Err(e) => {
                tracing::error!(error = %e, "Search failed");
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }

    Ok(Json(vec![]))
}
