use diesel::sql_query;
use diesel::sql_types::{Integer, Text};
use diesel_async::RunQueryDsl;
use pgvector::Vector;

use crate::db::{DbPool, SchemaManager};
use crate::error::Result;
use crate::models::SearchResult;
use crate::services::embedding::{EmbeddingProvider, MockEmbeddingProvider};

pub struct SearchService;

impl SearchService {
    pub async fn search(
        pool: &DbPool,
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

        let mut conn = pool.get().await?;
        let results: Vec<SearchResult> = sql_query(sql)
            .bind::<Text, _>(query)
            .bind::<Integer, _>(limit)
            .load(&mut conn)
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
        pool: &DbPool,
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

        let mut conn = pool.get().await?;
        let results: Vec<SearchResult> = sql_query(sql)
            .bind::<pgvector::sql_types::Vector, _>(&query_vector)
            .bind::<Integer, _>(limit)
            .load(&mut conn)
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
