use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};

pub async fn api_key_auth(request: Request, next: Next) -> Result<Response, StatusCode> {
    let api_key = request
        .headers()
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok());

    // TODO: Validate API key against stored keys
    if api_key.is_some() {
        // Validate key here
    }

    Ok(next.run(request).await)
}
