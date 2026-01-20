use sqlx::PgPool;

use crate::db::SchemaManager;
use crate::error::{AppError, Result};
use crate::models::BronzeRecord;

const TEXT_COLUMN_NAMES: &[&str] = &[
    "text",
    "content",
    "body",
    "message",
    "description",
    "notes",
];

pub struct TransformationService;

impl TransformationService {
    pub async fn transform_bronze_to_gold(pool: &PgPool, customer_id: &str) -> Result<u64> {
        let schema = SchemaManager::schema_name(customer_id);

        let query = format!(
            "SELECT id, source_file, raw_data, ingested_at FROM {}.bronze
             WHERE id NOT IN (SELECT bronze_id FROM {}.gold WHERE bronze_id IS NOT NULL)",
            schema, schema
        );

        let bronze_records: Vec<BronzeRecord> = sqlx::query_as(&query).fetch_all(pool).await?;

        let mut count = 0u64;

        for bronze in bronze_records {
            let raw_data = bronze.raw_data.0.as_object();

            if let Some(obj) = raw_data {
                let text = Self::extract_text(obj)?;
                let label = Self::derive_label(&text);

                let insert_query = format!(
                    "INSERT INTO {}.gold (bronze_id, text, label) VALUES ($1, $2, $3)",
                    schema
                );

                sqlx::query(&insert_query)
                    .bind(bronze.id)
                    .bind(&text)
                    .bind(&label)
                    .execute(pool)
                    .await?;

                count += 1;
            }
        }

        if count > 0 {
            SchemaManager::create_bm25_index(pool, customer_id).await?;
        }

        tracing::info!(
            "Transformed {} records to gold for customer {}",
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
        if text.to_lowercase().contains("important") {
            "high_priority".to_string()
        } else {
            "normal".to_string()
        }
    }
}
