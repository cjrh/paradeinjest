use async_trait::async_trait;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::error::Result;

pub const EMBEDDING_DIMENSION: usize = 1536;

#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
}

pub struct MockEmbeddingProvider;

impl MockEmbeddingProvider {
    pub fn new() -> Self {
        Self
    }

    fn hash_text(text: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        hasher.finish()
    }

    fn seeded_random(seed: u64, index: usize) -> f32 {
        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);
        index.hash(&mut hasher);
        let hash = hasher.finish();
        // Convert to float between -1 and 1
        ((hash % 10000) as f32 / 10000.0) * 2.0 - 1.0
    }

    fn normalize(vec: &mut [f32]) {
        let magnitude: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for v in vec.iter_mut() {
                *v /= magnitude;
            }
        }
    }

    fn generate_embedding(text: &str) -> Vec<f32> {
        let hash = Self::hash_text(text);
        let mut vec: Vec<f32> = (0..EMBEDDING_DIMENSION)
            .map(|i| Self::seeded_random(hash, i))
            .collect();
        Self::normalize(&mut vec);
        vec
    }
}

impl Default for MockEmbeddingProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EmbeddingProvider for MockEmbeddingProvider {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        Ok(Self::generate_embedding(text))
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        Ok(texts.iter().map(|t| Self::generate_embedding(t)).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_embedding_dimension() {
        let provider = MockEmbeddingProvider::new();
        let embedding = provider.embed("test text").await.unwrap();
        assert_eq!(embedding.len(), EMBEDDING_DIMENSION);
    }

    #[tokio::test]
    async fn test_mock_embedding_deterministic() {
        let provider = MockEmbeddingProvider::new();
        let embedding1 = provider.embed("test text").await.unwrap();
        let embedding2 = provider.embed("test text").await.unwrap();
        assert_eq!(embedding1, embedding2);
    }

    #[tokio::test]
    async fn test_mock_embedding_normalized() {
        let provider = MockEmbeddingProvider::new();
        let embedding = provider.embed("test text").await.unwrap();
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((magnitude - 1.0).abs() < 0.001);
    }
}
