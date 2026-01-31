use diesel::sql_query;
use diesel::sql_types::Bool;
use diesel::QueryableByName;
use diesel_async::RunQueryDsl;

use crate::db::pool::DbPool;
use crate::error::{AppError, Result};

#[derive(QueryableByName)]
struct ExistsResult {
    #[diesel(sql_type = Bool)]
    exists: bool,
}

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

    pub async fn ensure_customer_schema(pool: &DbPool, customer_id: &str) -> Result<()> {
        Self::validate_customer_id(customer_id)?;

        let mut conn = pool.get().await?;
        let query = format!("SELECT create_customer_schema('{}')", customer_id);
        sql_query(query).execute(&mut conn).await?;

        Ok(())
    }

    pub fn schema_name(customer_id: &str) -> String {
        format!("customer_{}", customer_id)
    }

    pub async fn create_bm25_index(pool: &DbPool, customer_id: &str) -> Result<()> {
        let schema = Self::schema_name(customer_id);

        let mut conn = pool.get().await?;

        let check_query = format!(
            "SELECT EXISTS (
                SELECT 1 FROM pg_indexes
                WHERE schemaname = '{}' AND indexname = 'gold_search_idx'
            ) as exists",
            schema
        );

        let result: ExistsResult = sql_query(check_query).get_result(&mut conn).await?;

        if !result.exists {
            let create_query = format!(
                r#"CREATE INDEX gold_search_idx ON {schema}.gold
                USING bm25 (id, text, label, sentiment, text_length, word_count)
                WITH (
                    key_field = 'id',
                    text_fields = '{{"text": {{}}, "label": {{"tokenizer": {{"type": "keyword"}}}}, "sentiment": {{"tokenizer": {{"type": "keyword"}}}}}}',
                    numeric_fields = '{{"text_length": {{"fast": true}}, "word_count": {{"fast": true}}}}'
                )"#,
                schema = schema
            );
            sql_query(create_query).execute(&mut conn).await?;
            tracing::info!("Created BM25 index for customer {}", customer_id);
        }

        Ok(())
    }

    pub async fn create_silver_embedding_index(pool: &DbPool, customer_id: &str) -> Result<()> {
        let schema = Self::schema_name(customer_id);

        let mut conn = pool.get().await?;

        let check_query = format!(
            "SELECT EXISTS (
                SELECT 1 FROM pg_indexes
                WHERE schemaname = '{}' AND indexname = 'silver_embedding_idx'
            ) as exists",
            schema
        );

        let result: ExistsResult = sql_query(check_query).get_result(&mut conn).await?;

        if !result.exists {
            let create_query = format!(
                "CREATE INDEX silver_embedding_idx ON {}.silver
                 USING ivfflat (embedding vector_cosine_ops) WITH (lists = 100)",
                schema
            );
            sql_query(create_query).execute(&mut conn).await?;
            tracing::info!(
                "Created silver embedding index for customer {}",
                customer_id
            );
        }

        Ok(())
    }

    pub async fn create_gold_embedding_index(pool: &DbPool, customer_id: &str) -> Result<()> {
        let schema = Self::schema_name(customer_id);

        let mut conn = pool.get().await?;

        let check_query = format!(
            "SELECT EXISTS (
                SELECT 1 FROM pg_indexes
                WHERE schemaname = '{}' AND indexname = 'gold_embedding_idx'
            ) as exists",
            schema
        );

        let result: ExistsResult = sql_query(check_query).get_result(&mut conn).await?;

        if !result.exists {
            let create_query = format!(
                "CREATE INDEX gold_embedding_idx ON {}.gold
                 USING ivfflat (embedding vector_cosine_ops) WITH (lists = 100)",
                schema
            );
            sql_query(create_query).execute(&mut conn).await?;
            tracing::info!("Created gold embedding index for customer {}", customer_id);
        }

        Ok(())
    }
}
