use axum::{
    extract::{Path, State},
    Json,
};
use serde::Serialize;
use sqlx::PgPool;

use crate::error::Result;
use crate::services::TransformationService;

#[derive(Serialize)]
pub struct ProcessResponse {
    pub message: String,
    pub records_processed: u64,
}

pub async fn process_handler(
    State(pool): State<PgPool>,
    Path(customer_id): Path<String>,
) -> Result<Json<ProcessResponse>> {
    let count = TransformationService::transform_bronze_to_gold(&pool, &customer_id).await?;

    Ok(Json(ProcessResponse {
        message: format!("Successfully processed {} records to gold layer", count),
        records_processed: count,
    }))
}
