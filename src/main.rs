mod ai;
mod cache;
mod config;
mod db;
mod external;
mod handlers;
mod models;
mod repositories;
mod services;

use std::net::SocketAddr;

use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing (structured JSON logging)
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "cardeal=info,tower_http=info".into()))
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    // Load configuration
    let cfg = config::Config::load()?;
    tracing::info!(port = cfg.server.port, "starting CarDeal API");

    // Connect to PostgreSQL (+ run migrations)
    let pool = db::connect(&cfg.database.url, cfg.database.max_connections).await?;

    // Connect to Redis
    let redis_cache = cache::RedisCache::new(&cfg.redis).await?;

    // Initialize AI client
    let llm = ai::LlmClient::new(cfg.ai.clone());

    // Initialize external data provider (Biluppgifter.se / Transportstyrelsen)
    let external = external::VehicleDataProvider::new(&cfg);

    // Build service & router
    let service = services::VehicleService::new(pool, redis_cache, llm, external);
    let app = handlers::build_router(service)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], cfg.server.port));
    tracing::info!(%addr, "server listening");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("server stopped");
    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C handler");
    tracing::info!("shutdown signal received");
}
