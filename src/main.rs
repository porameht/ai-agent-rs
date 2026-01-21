use ai_agent::api::{create_router, queue, AppState};
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();

    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".into());
    let redis_pool = queue::create_pool(&redis_url)?;
    info!("Redis pool initialized");

    let state = AppState::new(redis_pool);
    let app = create_router(state);

    let host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port: u16 = std::env::var("SERVER_PORT")
        .unwrap_or_else(|_| "8080".into())
        .parse()?;
    let addr = SocketAddr::new(host.parse()?, port);

    info!("API server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
