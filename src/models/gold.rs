use pgvector::Vector;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GoldRecord {
    pub id: i32,
    pub silver_id: Option<i32>,
    pub bronze_id: Option<i32>,
    pub text: String,
    pub label: String,
    pub sentiment: Option<String>,
    pub sentiment_score: Option<f32>,
    pub text_length: Option<i32>,
    pub word_count: Option<i32>,
    #[sqlx(skip)]
    #[serde(skip)]
    pub embedding: Option<Vector>,
    #[serde(with = "time::serde::rfc3339")]
    pub processed_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SearchResult {
    pub id: i32,
    pub text: String,
    pub label: String,
    pub sentiment: Option<String>,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LabelCount {
    pub label: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoldInsert {
    pub silver_id: i32,
    pub bronze_id: i32,
    pub text: String,
    pub label: String,
    pub sentiment: String,
    pub sentiment_score: f32,
    pub text_length: i32,
    pub word_count: i32,
    pub embedding: Vec<f32>,
}
