use sqlx::PgPool;

use crate::error::{AppError, Result};

pub struct SchemaManager;

impl SchemaManager {
    pub fn validate_customer_id(customer_id: &str) -> Result<()> {
        if customer_id.is_empty() {
            return Err(AppError::InvalidCustomerId(
                "Customer ID cannot be empty".into(),
            ));
        }

        if !customer_id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_')
        {
            return Err(AppError::InvalidCustomerId(
                "Customer ID must contain only alphanumeric characters and underscores".into(),
            ));
        }

        Ok(())
    }

    pub async fn ensure_customer_schema(pool: &PgPool, customer_id: &str) -> Result<()> {
        Self::validate_customer_id(customer_id)?;

        sqlx::query("SELECT create_customer_schema($1)")
            .bind(customer_id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub fn schema_name(customer_id: &str) -> String {
        format!("customer_{}", customer_id)
    }

    pub async fn create_bm25_index(pool: &PgPool, customer_id: &str) -> Result<()> {
        let schema = Self::schema_name(customer_id);

        let index_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (
                SELECT 1 FROM pg_indexes
                WHERE schemaname = $1 AND indexname = 'gold_search_idx'
            )",
        )
        .bind(&schema)
        .fetch_one(pool)
        .await?;

        if !index_exists {
            let query = format!(
                "CREATE INDEX gold_search_idx ON {}.gold USING bm25 (id, text, label) WITH (key_field='id')",
                schema
            );
            sqlx::query(&query).execute(pool).await?;
            tracing::info!("Created BM25 index for customer {}", customer_id);
        }

        Ok(())
    }
}
