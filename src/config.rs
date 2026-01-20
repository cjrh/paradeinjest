use std::env;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub host: String,
    pub port: u16,
    pub embedding: EmbeddingConfig,
}

#[derive(Clone)]
pub struct EmbeddingConfig {
    pub provider: EmbeddingProvider,
    pub dimension: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EmbeddingProvider {
    Mock,
    OpenAI,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let embedding_provider = match env::var("EMBEDDING_PROVIDER")
            .unwrap_or_else(|_| "mock".into())
            .to_lowercase()
            .as_str()
        {
            "openai" => EmbeddingProvider::OpenAI,
            _ => EmbeddingProvider::Mock,
        };

        let embedding_dimension = env::var("EMBEDDING_DIMENSION")
            .ok()
            .and_then(|d| d.parse().ok())
            .unwrap_or(1536);

        Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://parade:parade@localhost:5432/paradeinjest".into()),
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
            embedding: EmbeddingConfig {
                provider: embedding_provider,
                dimension: embedding_dimension,
            },
        }
    }

    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
