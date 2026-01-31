use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] diesel::result::Error),

    #[error("Connection pool error: {0}")]
    Pool(String),

    #[error("CSV parsing error: {0}")]
    CsvParse(#[from] csv::Error),

    #[error("Invalid customer ID: {0}")]
    InvalidCustomerId(String),

    #[error("No text column found in CSV")]
    NoTextColumn,

    #[error("Multipart error: {0}")]
    Multipart(#[from] axum::extract::multipart::MultipartError),

    #[error("No file uploaded")]
    NoFileUploaded,

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<diesel_async::pooled_connection::bb8::RunError> for AppError {
    fn from(err: diesel_async::pooled_connection::bb8::RunError) -> Self {
        AppError::Pool(err.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Database(e) => {
                tracing::error!("Database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            AppError::Pool(e) => {
                tracing::error!("Pool error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            AppError::CsvParse(e) => {
                tracing::error!("CSV parse error: {:?}", e);
                (StatusCode::BAD_REQUEST, self.to_string())
            }
            AppError::InvalidCustomerId(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::NoTextColumn => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::Multipart(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::NoFileUploaded => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
