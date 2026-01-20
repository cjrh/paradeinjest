use csv::ReaderBuilder;
use serde_json::{Map, Value};
use sqlx::PgPool;

use crate::db::SchemaManager;
use crate::error::Result;

pub struct IngestionService;

impl IngestionService {
    pub async fn ingest_csv(
        pool: &PgPool,
        customer_id: &str,
        filename: &str,
        csv_data: &[u8],
    ) -> Result<u64> {
        SchemaManager::ensure_customer_schema(pool, customer_id).await?;

        let schema = SchemaManager::schema_name(customer_id);
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(csv_data);

        let headers: Vec<String> = reader
            .headers()?
            .iter()
            .map(|h| h.to_string())
            .collect();

        let mut count = 0u64;

        for result in reader.records() {
            let record = result?;
            let mut json_obj = Map::new();

            for (i, value) in record.iter().enumerate() {
                if let Some(header) = headers.get(i) {
                    json_obj.insert(header.clone(), Value::String(value.to_string()));
                }
            }

            let raw_data = Value::Object(json_obj);

            let query = format!(
                "INSERT INTO {}.bronze (source_file, raw_data) VALUES ($1, $2)",
                schema
            );

            sqlx::query(&query)
                .bind(filename)
                .bind(&raw_data)
                .execute(pool)
                .await?;

            count += 1;
        }

        tracing::info!(
            "Ingested {} records from {} for customer {}",
            count,
            filename,
            customer_id
        );

        Ok(count)
    }
}
