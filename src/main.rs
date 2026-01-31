mod config;
mod db;
mod error;
mod handlers;
mod models;
mod routes;
mod services;

use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;
use crate::db::{create_pool, run_migrations};
use crate::routes::create_router;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "paradeinjest=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env();

    tracing::info!("Running database migrations...");
    run_migrations(&config.database_url).map_err(|e| anyhow::anyhow!("{}", e))?;

    tracing::info!("Connecting to database...");
    let pool = create_pool(&config.database_url).await.map_err(|e| anyhow::anyhow!("{}", e))?;
    tracing::info!("Database connected");

    let app = create_router(pool).layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(config.bind_addr()).await?;
    tracing::info!("Server listening on {}", config.bind_addr());

    axum::serve(listener, app).await?;

    Ok(())
}
