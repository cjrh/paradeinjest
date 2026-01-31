use diesel::sql_types::{Integer, Jsonb, Text, Timestamptz};
use diesel::QueryableByName;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, QueryableByName)]
pub struct BronzeRecord {
    #[diesel(sql_type = Integer)]
    pub id: i32,
    #[diesel(sql_type = Text)]
    pub source_file: String,
    #[diesel(sql_type = Jsonb)]
    pub raw_data: serde_json::Value,
    #[diesel(sql_type = Timestamptz)]
    #[serde(with = "time::serde::rfc3339")]
    pub ingested_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BronzeInsert {
    pub source_file: String,
    pub raw_data: serde_json::Value,
}
