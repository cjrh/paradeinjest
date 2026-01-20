use pgvector::Vector;
use sqlx::PgPool;

use crate::db::SchemaManager;
use crate::error::Result;

pub struct TransformationService;

impl TransformationService {
    pub async fn transform_silver_to_gold(pool: &PgPool, customer_id: &str) -> Result<u64> {
        let schema = SchemaManager::schema_name(customer_id);

        // Fetch unprocessed silver records
        let query = format!(
            "SELECT id, bronze_id, primary_text, label, sentiment, sentiment_score, embedding
             FROM {}.silver
             WHERE id NOT IN (SELECT silver_id FROM {}.gold WHERE silver_id IS NOT NULL)",
            schema, schema
        );

        #[derive(sqlx::FromRow)]
        struct SilverRow {
            id: i32,
            bronze_id: Option<i32>,
            primary_text: Option<String>,
            label: Option<String>,
            sentiment: Option<String>,
            sentiment_score: Option<f32>,
            embedding: Option<Vector>,
        }

        let silver_records: Vec<SilverRow> = sqlx::query_as(&query).fetch_all(pool).await?;

        let mut count = 0u64;

        for silver in silver_records {
            let text = silver.primary_text.unwrap_or_default();
            let label = silver.label.unwrap_or_else(|| "general".to_string());
            let sentiment = silver.sentiment.unwrap_or_else(|| "neutral".to_string());
            let sentiment_score = silver.sentiment_score.unwrap_or(0.0);

            let text_length = text.len() as i32;
            let word_count = text.split_whitespace().count() as i32;

            let insert_query = format!(
                "INSERT INTO {}.gold (
                    silver_id, bronze_id, text, label, sentiment, sentiment_score,
                    text_length, word_count, embedding
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                schema
            );

            sqlx::query(&insert_query)
                .bind(silver.id)
                .bind(silver.bronze_id)
                .bind(&text)
                .bind(&label)
                .bind(&sentiment)
                .bind(sentiment_score)
                .bind(text_length)
                .bind(word_count)
                .bind(&silver.embedding)
                .execute(pool)
                .await?;

            count += 1;
        }

        if count > 0 {
            SchemaManager::create_bm25_index(pool, customer_id).await?;
            SchemaManager::create_gold_embedding_index(pool, customer_id).await?;
        }

        tracing::info!(
            "Transformed {} records to gold for customer {}",
            count,
            customer_id
        );

        Ok(count)
    }

    pub async fn transform_full_pipeline(pool: &PgPool, customer_id: &str) -> Result<(u64, u64)> {
        use crate::services::SilverService;

        let silver_count = SilverService::transform_bronze_to_silver(pool, customer_id).await?;
        let gold_count = Self::transform_silver_to_gold(pool, customer_id).await?;

        tracing::info!(
            "Full pipeline for customer {}: {} bronze→silver, {} silver→gold",
            customer_id,
            silver_count,
            gold_count
        );

        Ok((silver_count, gold_count))
    }
}
