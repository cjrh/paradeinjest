use axum::{
    extract::{Path, State},
    Json,
};
use sqlx::PgPool;

use crate::error::Result;
use crate::services::{AnalyticsResult, AnalyticsService};

pub async fn analytics_handler(
    State(pool): State<PgPool>,
    Path(customer_id): Path<String>,
) -> Result<Json<AnalyticsResult>> {
    let analytics = AnalyticsService::get_full_analytics(&pool, &customer_id).await?;

    Ok(Json(analytics))
}
