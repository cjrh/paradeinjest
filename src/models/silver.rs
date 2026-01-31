use diesel::sql_types::{Float4, Integer, Jsonb, Nullable, Text, Timestamptz};
use diesel::QueryableByName;
use pgvector::Vector;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, QueryableByName)]
pub struct SilverRecord {
    #[diesel(sql_type = Integer)]
    pub id: i32,
    #[diesel(sql_type = Nullable<Integer>)]
    pub bronze_id: Option<i32>,
    #[diesel(sql_type = Jsonb)]
    pub normalized_data: serde_json::Value,
    #[diesel(sql_type = Nullable<Text>)]
    pub primary_text: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub label: Option<String>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    #[serde(with = "time::serde::rfc3339::option")]
    pub source_date: Option<OffsetDateTime>,
    #[diesel(sql_type = Nullable<Float4>)]
    pub data_quality_score: Option<f32>,
    #[diesel(sql_type = Nullable<pgvector::sql_types::Vector>)]
    #[serde(skip)]
    pub embedding: Option<Vector>,
    #[diesel(sql_type = Nullable<Text>)]
    pub sentiment: Option<String>,
    #[diesel(sql_type = Nullable<Float4>)]
    pub sentiment_score: Option<f32>,
    #[diesel(sql_type = Nullable<Jsonb>)]
    pub field_mapping: Option<serde_json::Value>,
    #[diesel(sql_type = Timestamptz)]
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
