use sqlx::PgPool;

use crate::db::SchemaManager;
use crate::error::Result;
use crate::models::gold::LabelCount;

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
}
