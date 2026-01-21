pub mod chat;
pub mod documents;
pub mod health;

use axum::http::{header, Method};
use axum::{routing::get, routing::post, Router};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::warn;

use crate::api::state::AppState;

pub fn create_router(state: AppState) -> Router {
    let cors = build_cors(&state);

    Router::new()
        .route("/health", get(health::health_check))
        .route("/ready", get(health::readiness_check))
        .nest("/api/v1", api_v1_routes())
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

fn build_cors(state: &AppState) -> CorsLayer {
    let cors_config = &state.config.config.cors;

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    if cors_config.is_permissive() {
        warn!("CORS is configured to allow all origins - not recommended for production");
        cors.allow_origin(Any)
    } else {
        let origins: Vec<_> = cors_config
            .allowed_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        cors.allow_origin(origins)
    }
}

fn api_v1_routes() -> Router<AppState> {
    Router::new()
        .route("/chat", post(chat::chat_handler))
        .route("/chat/jobs/{job_id}", get(chat::get_job_status))
        .route("/documents", post(documents::create_document))
        .route("/documents", get(documents::list_documents))
        .route("/documents/{id}", get(documents::get_document))
        .route(
            "/documents/{id}",
            axum::routing::delete(documents::delete_document),
        )
        .route("/documents/search", post(documents::search_documents))
}
