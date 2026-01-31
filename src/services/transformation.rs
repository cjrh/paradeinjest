use diesel::sql_query;
use diesel::sql_types::{Float4, Integer, Nullable, Text};
use diesel::QueryableByName;
use diesel_async::RunQueryDsl;
use pgvector::Vector;

use crate::db::{DbPool, SchemaManager};
use crate::error::Result;

pub struct TransformationService;

impl TransformationService {
    pub async fn transform_silver_to_gold(pool: &DbPool, customer_id: &str) -> Result<u64> {
        let schema = SchemaManager::schema_name(customer_id);

        // Fetch unprocessed silver records
        let query = format!(
            "SELECT id, bronze_id, primary_text, label, sentiment, sentiment_score, embedding
             FROM {}.silver
             WHERE id NOT IN (SELECT silver_id FROM {}.gold WHERE silver_id IS NOT NULL)",
            schema, schema
        );

        #[derive(QueryableByName)]
        struct SilverRow {
            #[diesel(sql_type = Integer)]
            id: i32,
            #[diesel(sql_type = Nullable<Integer>)]
            bronze_id: Option<i32>,
            #[diesel(sql_type = Nullable<Text>)]
            primary_text: Option<String>,
            #[diesel(sql_type = Nullable<Text>)]
            label: Option<String>,
            #[diesel(sql_type = Nullable<Text>)]
            sentiment: Option<String>,
            #[diesel(sql_type = Nullable<Float4>)]
            sentiment_score: Option<f32>,
            #[diesel(sql_type = Nullable<pgvector::sql_types::Vector>)]
            embedding: Option<Vector>,
        }

        let mut conn = pool.get().await?;
        let silver_records: Vec<SilverRow> = sql_query(query).load(&mut conn).await?;

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

            sql_query(insert_query)
                .bind::<Integer, _>(silver.id)
                .bind::<Nullable<Integer>, _>(silver.bronze_id)
                .bind::<Text, _>(&text)
                .bind::<Text, _>(&label)
                .bind::<Text, _>(&sentiment)
                .bind::<Float4, _>(sentiment_score)
                .bind::<Integer, _>(text_length)
                .bind::<Integer, _>(word_count)
                .bind::<Nullable<pgvector::sql_types::Vector>, _>(&silver.embedding)
                .execute(&mut conn)
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

    pub async fn transform_full_pipeline(pool: &DbPool, customer_id: &str) -> Result<(u64, u64)> {
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
