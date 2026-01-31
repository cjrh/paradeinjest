use diesel::sql_types::{Float4, Integer, Nullable, Text, Timestamptz};
use diesel::QueryableByName;
use pgvector::Vector;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, QueryableByName)]
pub struct GoldRecord {
    #[diesel(sql_type = Integer)]
    pub id: i32,
    #[diesel(sql_type = Nullable<Integer>)]
    pub silver_id: Option<i32>,
    #[diesel(sql_type = Nullable<Integer>)]
    pub bronze_id: Option<i32>,
    #[diesel(sql_type = Text)]
    pub text: String,
    #[diesel(sql_type = Text)]
    pub label: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub sentiment: Option<String>,
    #[diesel(sql_type = Nullable<Float4>)]
    pub sentiment_score: Option<f32>,
    #[diesel(sql_type = Nullable<Integer>)]
    pub text_length: Option<i32>,
    #[diesel(sql_type = Nullable<Integer>)]
    pub word_count: Option<i32>,
    #[diesel(sql_type = Nullable<pgvector::sql_types::Vector>)]
    #[serde(skip)]
    pub embedding: Option<Vector>,
    #[diesel(sql_type = Timestamptz)]
    #[serde(with = "time::serde::rfc3339")]
    pub processed_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, QueryableByName)]
pub struct SearchResult {
    #[diesel(sql_type = Integer)]
    pub id: i32,
    #[diesel(sql_type = Text)]
    pub text: String,
    #[diesel(sql_type = Text)]
    pub label: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub sentiment: Option<String>,
    #[diesel(sql_type = Float4)]
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, QueryableByName)]
pub struct LabelCount {
    #[diesel(sql_type = Text)]
    pub label: String,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
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
