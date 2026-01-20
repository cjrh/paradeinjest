use axum::{
    routing::{get, post},
    Router,
};
use sqlx::PgPool;

use crate::handlers::{
    analytics_handler, bronze_to_silver_handler, process_handler, search_handler,
    semantic_search_handler, silver_to_gold_handler, upload_handler,
};

pub fn create_router(pool: PgPool) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/customers/:id/upload", post(upload_handler))
        // Full pipeline: bronze -> silver -> gold
        .route("/customers/:id/process", post(process_handler))
        // Step-by-step processing
        .route(
            "/customers/:id/process/bronze-to-silver",
            post(bronze_to_silver_handler),
        )
        .route(
            "/customers/:id/process/silver-to-gold",
            post(silver_to_gold_handler),
        )
        // Search endpoints
        .route("/customers/:id/search", get(search_handler))
        .route(
            "/customers/:id/search/semantic",
            get(semantic_search_handler),
        )
        // Analytics
        .route("/customers/:id/analytics", get(analytics_handler))
        .with_state(pool)
}

async fn health_check() -> &'static str {
    "OK"
}
