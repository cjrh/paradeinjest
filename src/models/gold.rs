use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GoldRecord {
    pub id: i32,
    pub bronze_id: Option<i32>,
    pub text: String,
    pub label: String,
    #[serde(with = "time::serde::rfc3339")]
    pub processed_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SearchResult {
    pub id: i32,
    pub text: String,
    pub label: String,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LabelCount {
    pub label: String,
    pub count: i64,
}
