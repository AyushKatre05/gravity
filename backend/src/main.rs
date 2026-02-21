mod api;
mod complexity;
mod db;
mod graph;
mod models;
mod parser;

use std::net::SocketAddr;
use anyhow::{Context, Result};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use api::{AppState, build_router};

#[tokio::main]
async fn main() -> Result<()> {
    // ── 1. Tracing / logging ─────────────────────────────────────────────────
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    // ── 2. Environment variables ─────────────────────────────────────────────
    // Load .env in development; in production env vars come from the platform.
    let _ = dotenvy::dotenv();

    let database_url = std::env::var("DATABASE_URL")
        .context("DATABASE_URL must be set")?;

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .context("PORT must be a valid u16")?;

    let analyze_path = std::env::var("ANALYZE_PATH")
        .unwrap_or_else(|_| "/analyze".to_string());

    // ── 3. Database ──────────────────────────────────────────────────────────
    info!("Connecting to database…");
    let pool = db::init_pool(&database_url).await?;

    info!("Running migrations…");
    db::run_migrations(&pool).await?;

    // ── 4. Application state ─────────────────────────────────────────────────
    let state = AppState {
        pool,
        analyze_path,
    };

    // ── 5. CORS ──────────────────────────────────────────────────────────────
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // ── 6. Router ─────────────────────────────────────────────────────────────
    let app = build_router(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    // ── 7. Bind & serve ───────────────────────────────────────────────────────
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Gravity backend listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("Failed to bind to {addr}"))?;

    axum::serve(listener, app)
        .await
        .context("Server error")?;

    Ok(())
}
