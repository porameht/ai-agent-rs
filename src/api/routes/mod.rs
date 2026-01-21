pub mod chat;
pub mod documents;
pub mod health;

use axum::{routing::get, routing::post, Router};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::api::state::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::health_check))
        .route("/ready", get(health::readiness_check))
        .nest("/api/v1", api_v1_routes())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

fn api_v1_routes() -> Router<AppState> {
    Router::new()
        .route("/chat", post(chat::chat_handler))
        .route("/chat/jobs/{job_id}", get(chat::get_job_status))
        .route("/documents", post(documents::create_document))
        .route("/documents", get(documents::list_documents))
        .route("/documents/{id}", get(documents::get_document))
        .route("/documents/{id}", axum::routing::delete(documents::delete_document))
        .route("/documents/search", post(documents::search_documents))
}
