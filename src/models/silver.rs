use pgvector::Vector;
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SilverRecord {
    pub id: i32,
    pub bronze_id: Option<i32>,
    pub normalized_data: Json<serde_json::Value>,
    pub primary_text: Option<String>,
    pub label: Option<String>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub source_date: Option<OffsetDateTime>,
    pub data_quality_score: Option<f32>,
    #[sqlx(skip)]
    #[serde(skip)]
    pub embedding: Option<Vector>,
    pub sentiment: Option<String>,
    pub sentiment_score: Option<f32>,
    pub field_mapping: Option<Json<serde_json::Value>>,
    #[serde(with = "time::serde::rfc3339")]
    pub processed_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SilverInsert {
    pub bronze_id: i32,
    pub normalized_data: serde_json::Value,
    pub primary_text: String,
    pub label: String,
    pub data_quality_score: f32,
    pub embedding: Vec<f32>,
    pub sentiment: String,
    pub sentiment_score: f32,
    pub field_mapping: serde_json::Value,
}
