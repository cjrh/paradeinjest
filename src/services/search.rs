use sqlx::PgPool;

use crate::db::SchemaManager;
use crate::error::Result;
use crate::models::SearchResult;

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
            "SELECT id, text, label, pdb.score(id) as score
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
}
