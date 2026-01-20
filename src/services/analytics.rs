use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::db::SchemaManager;
use crate::error::Result;
use crate::models::gold::LabelCount;

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyticsResult {
    pub customer_id: String,
    pub label_counts: Vec<LabelCount>,
    pub sentiment_breakdown: SentimentBreakdown,
    pub avg_text_length: f64,
    pub avg_word_count: f64,
    pub total_records: i64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SentimentBreakdown {
    pub positive: i64,
    pub neutral: i64,
    pub negative: i64,
}

pub struct AnalyticsService;

impl AnalyticsService {
    pub async fn get_label_counts(pool: &PgPool, customer_id: &str) -> Result<Vec<LabelCount>> {
        let schema = SchemaManager::schema_name(customer_id);

        let query = format!(
            "SELECT label, COUNT(*) as count
             FROM {}.gold
             GROUP BY label
             ORDER BY count DESC",
            schema
        );

        let results: Vec<LabelCount> = sqlx::query_as(&query).fetch_all(pool).await?;

        Ok(results)
    }

    pub async fn get_full_analytics(pool: &PgPool, customer_id: &str) -> Result<AnalyticsResult> {
        let schema = SchemaManager::schema_name(customer_id);

        // Get label counts
        let label_counts = Self::get_label_counts(pool, customer_id).await?;

        // Get sentiment breakdown using pdb.agg()
        let sentiment_breakdown = Self::get_sentiment_breakdown(pool, &schema).await?;

        // Get average text length using pdb.agg()
        let avg_text_length = Self::get_avg_text_length(pool, &schema).await?;

        // Get average word count
        let avg_word_count = Self::get_avg_word_count(pool, &schema).await?;

        // Get total records
        let total_records = Self::get_total_records(pool, &schema).await?;

        Ok(AnalyticsResult {
            customer_id: customer_id.to_string(),
            label_counts,
            sentiment_breakdown,
            avg_text_length,
            avg_word_count,
            total_records,
        })
    }

    async fn get_sentiment_breakdown(pool: &PgPool, schema: &str) -> Result<SentimentBreakdown> {
        // Try using pdb.agg() for sentiment aggregation
        let agg_query = format!(
            r#"SELECT pdb.agg('{{"aggs": {{"sentiment_counts": {{"terms": {{"field": "sentiment"}}}}}}}}') FROM {}.gold"#,
            schema
        );

        // Try the aggregation query, fall back to simple SQL if it fails
        match sqlx::query_scalar::<_, serde_json::Value>(&agg_query)
            .fetch_optional(pool)
            .await
        {
            Ok(Some(result)) => {
                // Parse the pdb.agg() result
                if let Some(buckets) = result
                    .get("sentiment_counts")
                    .and_then(|v| v.get("buckets"))
                    .and_then(|v| v.as_array())
                {
                    let mut breakdown = SentimentBreakdown::default();
                    for bucket in buckets {
                        let key = bucket.get("key").and_then(|k| k.as_str()).unwrap_or("");
                        let count = bucket
                            .get("doc_count")
                            .and_then(|c| c.as_i64())
                            .unwrap_or(0);
                        match key {
                            "positive" => breakdown.positive = count,
                            "neutral" => breakdown.neutral = count,
                            "negative" => breakdown.negative = count,
                            _ => {}
                        }
                    }
                    return Ok(breakdown);
                }
                // Fall through to SQL approach
                Self::get_sentiment_breakdown_sql(pool, schema).await
            }
            _ => Self::get_sentiment_breakdown_sql(pool, schema).await,
        }
    }

    async fn get_sentiment_breakdown_sql(
        pool: &PgPool,
        schema: &str,
    ) -> Result<SentimentBreakdown> {
        let query = format!(
            "SELECT sentiment, COUNT(*) as count
             FROM {}.gold
             WHERE sentiment IS NOT NULL
             GROUP BY sentiment",
            schema
        );

        let rows: Vec<(String, i64)> = sqlx::query_as(&query).fetch_all(pool).await?;

        let mut breakdown = SentimentBreakdown::default();
        for (sentiment, count) in rows {
            match sentiment.as_str() {
                "positive" => breakdown.positive = count,
                "neutral" => breakdown.neutral = count,
                "negative" => breakdown.negative = count,
                _ => {}
            }
        }

        Ok(breakdown)
    }

    async fn get_avg_text_length(pool: &PgPool, schema: &str) -> Result<f64> {
        // Try using pdb.agg() for average calculation
        let agg_query = format!(
            r#"SELECT pdb.agg('{{"aggs": {{"avg_length": {{"avg": {{"field": "text_length"}}}}}}}}') FROM {}.gold"#,
            schema
        );

        match sqlx::query_scalar::<_, serde_json::Value>(&agg_query)
            .fetch_optional(pool)
            .await
        {
            Ok(Some(result)) => {
                if let Some(avg) = result
                    .get("avg_length")
                    .and_then(|v| v.get("value"))
                    .and_then(|v| v.as_f64())
                {
                    return Ok(avg);
                }
                Self::get_avg_text_length_sql(pool, schema).await
            }
            _ => Self::get_avg_text_length_sql(pool, schema).await,
        }
    }

    async fn get_avg_text_length_sql(pool: &PgPool, schema: &str) -> Result<f64> {
        let query = format!(
            "SELECT COALESCE(AVG(text_length), 0) FROM {}.gold",
            schema
        );

        let avg: f64 = sqlx::query_scalar(&query)
            .fetch_one(pool)
            .await
            .unwrap_or(0.0);

        Ok(avg)
    }

    async fn get_avg_word_count(pool: &PgPool, schema: &str) -> Result<f64> {
        // Try using pdb.agg() for average calculation
        let agg_query = format!(
            r#"SELECT pdb.agg('{{"aggs": {{"avg_words": {{"avg": {{"field": "word_count"}}}}}}}}') FROM {}.gold"#,
            schema
        );

        match sqlx::query_scalar::<_, serde_json::Value>(&agg_query)
            .fetch_optional(pool)
            .await
        {
            Ok(Some(result)) => {
                if let Some(avg) = result
                    .get("avg_words")
                    .and_then(|v| v.get("value"))
                    .and_then(|v| v.as_f64())
                {
                    return Ok(avg);
                }
                Self::get_avg_word_count_sql(pool, schema).await
            }
            _ => Self::get_avg_word_count_sql(pool, schema).await,
        }
    }

    async fn get_avg_word_count_sql(pool: &PgPool, schema: &str) -> Result<f64> {
        let query = format!(
            "SELECT COALESCE(AVG(word_count), 0) FROM {}.gold",
            schema
        );

        let avg: f64 = sqlx::query_scalar(&query)
            .fetch_one(pool)
            .await
            .unwrap_or(0.0);

        Ok(avg)
    }

    async fn get_total_records(pool: &PgPool, schema: &str) -> Result<i64> {
        let query = format!("SELECT COUNT(*) FROM {}.gold", schema);

        let count: i64 = sqlx::query_scalar(&query)
            .fetch_one(pool)
            .await
            .unwrap_or(0);

        Ok(count)
    }
}
