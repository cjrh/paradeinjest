use axum::{
    extract::{Path, State},
    Json,
};
use serde::Serialize;

use crate::db::DbPool;
use crate::error::Result;
use crate::services::{SilverService, TransformationService};

#[derive(Serialize)]
pub struct ProcessResponse {
    pub message: String,
    pub records_processed: u64,
}

#[derive(Serialize)]
pub struct FullPipelineResponse {
    pub message: String,
    pub bronze_to_silver: u64,
    pub silver_to_gold: u64,
}

pub async fn bronze_to_silver_handler(
    State(pool): State<DbPool>,
    Path(customer_id): Path<String>,
) -> Result<Json<ProcessResponse>> {
    let count = SilverService::transform_bronze_to_silver(&pool, &customer_id).await?;

    Ok(Json(ProcessResponse {
        message: format!("Successfully processed {} records to silver layer", count),
        records_processed: count,
    }))
}

pub async fn silver_to_gold_handler(
    State(pool): State<DbPool>,
    Path(customer_id): Path<String>,
) -> Result<Json<ProcessResponse>> {
    let count = TransformationService::transform_silver_to_gold(&pool, &customer_id).await?;

    Ok(Json(ProcessResponse {
        message: format!("Successfully processed {} records to gold layer", count),
        records_processed: count,
    }))
}

pub async fn process_handler(
    State(pool): State<DbPool>,
    Path(customer_id): Path<String>,
) -> Result<Json<FullPipelineResponse>> {
    let (silver_count, gold_count) =
        TransformationService::transform_full_pipeline(&pool, &customer_id).await?;

    Ok(Json(FullPipelineResponse {
        message: format!(
            "Successfully processed {} records to silver and {} records to gold",
            silver_count, gold_count
        ),
        bronze_to_silver: silver_count,
        silver_to_gold: gold_count,
    }))
}
