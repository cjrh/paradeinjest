use axum::{
    extract::{Path, State},
    Json,
};
use serde::Serialize;
use sqlx::PgPool;

use crate::error::Result;
use crate::models::gold::LabelCount;
use crate::services::AnalyticsService;

#[derive(Serialize)]
pub struct AnalyticsResponse {
    pub customer_id: String,
    pub label_counts: Vec<LabelCount>,
    pub total_records: i64,
}

pub async fn analytics_handler(
    State(pool): State<PgPool>,
    Path(customer_id): Path<String>,
) -> Result<Json<AnalyticsResponse>> {
    let label_counts = AnalyticsService::get_label_counts(&pool, &customer_id).await?;
    let total: i64 = label_counts.iter().map(|lc| lc.count).sum();

    Ok(Json(AnalyticsResponse {
        customer_id,
        label_counts,
        total_records: total,
    }))
}
