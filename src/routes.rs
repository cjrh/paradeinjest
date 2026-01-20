use axum::{
    routing::{get, post},
    Router,
};
use sqlx::PgPool;

use crate::handlers::{analytics_handler, process_handler, search_handler, upload_handler};

pub fn create_router(pool: PgPool) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/customers/:id/upload", post(upload_handler))
        .route("/customers/:id/process", post(process_handler))
        .route("/customers/:id/search", get(search_handler))
        .route("/customers/:id/analytics", get(analytics_handler))
        .with_state(pool)
}

async fn health_check() -> &'static str {
    "OK"
}
