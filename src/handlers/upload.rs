use axum::{
    extract::{Multipart, Path, State},
    Json,
};
use serde::Serialize;
use sqlx::PgPool;

use crate::error::{AppError, Result};
use crate::services::IngestionService;

#[derive(Serialize)]
pub struct UploadResponse {
    pub message: String,
    pub records_ingested: u64,
    pub filename: String,
}

pub async fn upload_handler(
    State(pool): State<PgPool>,
    Path(customer_id): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>> {
    let mut filename = String::new();
    let mut data = Vec::new();

    while let Some(field) = multipart.next_field().await? {
        if field.name() == Some("file") {
            filename = field
                .file_name()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "unknown.csv".to_string());
            data = field.bytes().await?.to_vec();
        }
    }

    if data.is_empty() {
        return Err(AppError::NoFileUploaded);
    }

    let count = IngestionService::ingest_csv(&pool, &customer_id, &filename, &data).await?;

    Ok(Json(UploadResponse {
        message: format!("Successfully ingested {} records", count),
        records_ingested: count,
        filename,
    }))
}
