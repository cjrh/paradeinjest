use diesel::sql_query;
use diesel::sql_types::{Float4, Integer, Jsonb, Text};
use diesel_async::RunQueryDsl;
use pgvector::Vector;

use crate::db::{DbPool, SchemaManager};
use crate::error::{AppError, Result};
use crate::models::BronzeRecord;
use crate::services::embedding::{EmbeddingProvider, MockEmbeddingProvider};
use crate::services::sentiment::SentimentAnalyzer;

const TEXT_COLUMN_NAMES: &[&str] = &[
    "text",
    "content",
    "body",
    "message",
    "description",
    "notes",
];

pub struct SilverService;

impl SilverService {
    pub async fn transform_bronze_to_silver(pool: &DbPool, customer_id: &str) -> Result<u64> {
        let schema = SchemaManager::schema_name(customer_id);
        let embedding_provider = MockEmbeddingProvider::new();

        let query = format!(
            "SELECT id, source_file, raw_data, ingested_at FROM {}.bronze
             WHERE id NOT IN (SELECT bronze_id FROM {}.silver WHERE bronze_id IS NOT NULL)",
            schema, schema
        );

        let mut conn = pool.get().await?;
        let bronze_records: Vec<BronzeRecord> = sql_query(query).load(&mut conn).await?;

        let mut count = 0u64;

        for bronze in bronze_records {
            let raw_data = bronze.raw_data.as_object();

            if let Some(obj) = raw_data {
                let text = Self::extract_text(obj)?;
                let label = Self::derive_label(&text);
                let quality_score = Self::calculate_quality_score(&text, obj);

                // Generate embedding
                let embedding_vec = embedding_provider.embed(&text).await?;
                let embedding = Vector::from(embedding_vec);

                // Analyze sentiment
                let sentiment_result = SentimentAnalyzer::analyze(&text);

                // Create field mapping
                let field_mapping = Self::create_field_mapping(obj);

                let insert_query = format!(
                    "INSERT INTO {}.silver (
                        bronze_id, normalized_data, primary_text, label,
                        data_quality_score, embedding, sentiment, sentiment_score, field_mapping
                    ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                    schema
                );

                sql_query(insert_query)
                    .bind::<Integer, _>(bronze.id)
                    .bind::<Jsonb, _>(&bronze.raw_data)
                    .bind::<Text, _>(&text)
                    .bind::<Text, _>(&label)
                    .bind::<Float4, _>(quality_score)
                    .bind::<pgvector::sql_types::Vector, _>(&embedding)
                    .bind::<Text, _>(&sentiment_result.label)
                    .bind::<Float4, _>(sentiment_result.score)
                    .bind::<Jsonb, _>(serde_json::to_value(&field_mapping).unwrap_or_default())
                    .execute(&mut conn)
                    .await?;

                count += 1;
            }
        }

        if count > 0 {
            SchemaManager::create_silver_embedding_index(pool, customer_id).await?;
        }

        tracing::info!(
            "Transformed {} records to silver for customer {}",
            count,
            customer_id
        );

        Ok(count)
    }

    fn extract_text(obj: &serde_json::Map<String, serde_json::Value>) -> Result<String> {
        for col_name in TEXT_COLUMN_NAMES {
            if let Some(value) = obj.get(*col_name) {
                if let Some(text) = value.as_str() {
                    return Ok(text.to_string());
                }
            }
        }

        for (_key, value) in obj {
            if let Some(text) = value.as_str() {
                if text.len() > 10 {
                    return Ok(text.to_string());
                }
            }
        }

        Err(AppError::NoTextColumn)
    }

    fn derive_label(text: &str) -> String {
        let text_lower = text.to_lowercase();
        if text_lower.contains("important")
            || text_lower.contains("urgent")
            || text_lower.contains("critical")
        {
            "high_priority".to_string()
        } else if text_lower.contains("review") || text_lower.contains("pending") {
            "needs_review".to_string()
        } else {
            "general".to_string()
        }
    }

    fn calculate_quality_score(text: &str, obj: &serde_json::Map<String, serde_json::Value>) -> f32 {
        let mut score = 0.0f32;

        // Text length factor (longer is better up to a point)
        let len = text.len();
        if len > 10 {
            score += 0.2;
        }
        if len > 50 {
            score += 0.2;
        }
        if len > 100 {
            score += 0.1;
        }

        // Number of fields factor
        let field_count = obj.len();
        if field_count > 1 {
            score += 0.1;
        }
        if field_count > 3 {
            score += 0.1;
        }

        // Non-empty fields factor
        let non_empty_count = obj
            .values()
            .filter(|v| {
                v.as_str().map_or(false, |s| !s.trim().is_empty())
                    || v.as_i64().is_some()
                    || v.as_f64().is_some()
            })
            .count();

        score += (non_empty_count as f32 / field_count.max(1) as f32) * 0.3;

        score.min(1.0)
    }

    fn create_field_mapping(
        obj: &serde_json::Map<String, serde_json::Value>,
    ) -> serde_json::Map<String, serde_json::Value> {
        let mut mapping = serde_json::Map::new();

        for (key, value) in obj {
            let field_type = if value.is_string() {
                "string"
            } else if value.is_i64() || value.is_f64() {
                "number"
            } else if value.is_boolean() {
                "boolean"
            } else if value.is_null() {
                "null"
            } else if value.is_array() {
                "array"
            } else {
                "object"
            };

            mapping.insert(key.clone(), serde_json::Value::String(field_type.to_string()));
        }

        mapping
    }
}
