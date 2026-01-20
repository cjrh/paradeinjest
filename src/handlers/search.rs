use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::error::Result;
use crate::models::SearchResult;
use crate::services::SearchService;

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
    #[serde(default = "default_limit")]
    pub limit: i32,
}

fn default_limit() -> i32 {
    10
}

#[derive(Serialize)]
pub struct SearchResponse {
    pub query: String,
    pub results: Vec<SearchResult>,
    pub count: usize,
}

pub async fn search_handler(
    State(pool): State<PgPool>,
    Path(customer_id): Path<String>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<SearchResponse>> {
    let results = SearchService::search(&pool, &customer_id, &params.q, params.limit).await?;

    Ok(Json(SearchResponse {
        query: params.q,
        count: results.len(),
        results,
    }))
}

pub async fn semantic_search_handler(
    State(pool): State<PgPool>,
    Path(customer_id): Path<String>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<SearchResponse>> {
    let results =
        SearchService::semantic_search(&pool, &customer_id, &params.q, params.limit).await?;

    Ok(Json(SearchResponse {
        query: params.q,
        count: results.len(),
        results,
    }))
}
