use pgvector::Vector;
use sqlx::PgPool;

use crate::db::SchemaManager;
use crate::error::Result;
use crate::models::SearchResult;
use crate::services::embedding::{EmbeddingProvider, MockEmbeddingProvider};

pub struct SearchService;

impl SearchService {
    pub async fn search(
        pool: &PgPool,
        customer_id: &str,
        query: &str,
        limit: i32,
    ) -> Result<Vec<SearchResult>> {
        let schema = SchemaManager::schema_name(customer_id);

        let sql = format!(
            "SELECT id, text, label, sentiment, pdb.score(id) as score
             FROM {}.gold
             WHERE text ||| $1
             ORDER BY pdb.score(id) DESC
             LIMIT $2",
            schema
        );

        let results: Vec<SearchResult> = sqlx::query_as(&sql)
            .bind(query)
            .bind(limit)
            .fetch_all(pool)
            .await?;

        tracing::info!(
            "Search for '{}' in customer {} returned {} results",
            query,
            customer_id,
            results.len()
        );

        Ok(results)
    }

    pub async fn semantic_search(
        pool: &PgPool,
        customer_id: &str,
        query: &str,
        limit: i32,
    ) -> Result<Vec<SearchResult>> {
        let schema = SchemaManager::schema_name(customer_id);
        let embedding_provider = MockEmbeddingProvider::new();

        // Generate query embedding
        let query_embedding = embedding_provider.embed(query).await?;
        let query_vector = Vector::from(query_embedding);

        let sql = format!(
            "SELECT id, text, label, sentiment, (1 - (embedding <=> $1))::FLOAT4 as score
             FROM {}.gold
             WHERE embedding IS NOT NULL
             ORDER BY embedding <=> $1
             LIMIT $2",
            schema
        );

        let results: Vec<SearchResult> = sqlx::query_as(&sql)
            .bind(&query_vector)
            .bind(limit)
            .fetch_all(pool)
            .await?;

        tracing::info!(
            "Semantic search for '{}' in customer {} returned {} results",
            query,
            customer_id,
            results.len()
        );

        Ok(results)
    }
}
