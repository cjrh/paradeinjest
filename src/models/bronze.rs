use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BronzeRecord {
    pub id: i32,
    pub source_file: String,
    pub raw_data: Json<serde_json::Value>,
    #[serde(with = "time::serde::rfc3339")]
    pub ingested_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BronzeInsert {
    pub source_file: String,
    pub raw_data: serde_json::Value,
}
