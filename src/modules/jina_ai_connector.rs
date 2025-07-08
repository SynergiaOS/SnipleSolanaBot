// THE OVERMIND PROTOCOL - Jina AI Integration
// Advanced embeddings and reranking for enhanced AI decision making

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::time::{timeout, Duration};
use tracing::{debug, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JinaEmbeddingRequest {
    pub input: Vec<String>,
    pub model: String,
    pub task: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JinaEmbeddingResponse {
    pub object: String,
    pub data: Vec<EmbeddingData>,
    pub model: String,
    pub usage: Usage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingData {
    pub object: String,
    pub embedding: Vec<f32>,
    pub index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: usize,
    pub total_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JinaRerankRequest {
    pub model: String,
    pub query: String,
    pub documents: Vec<String>,
    pub top_n: Option<usize>,
    pub return_documents: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JinaRerankResponse {
    pub model: String,
    pub results: Vec<RerankResult>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankResult {
    pub index: usize,
    pub relevance_score: f64,
    pub document: Option<RerankDocument>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RerankDocument {
    pub text: String,
}

pub struct JinaAIConnector {
    client: Client,
    api_key: String,
    embedding_model: String,
    reranker_model: String,
    base_url: String,
}

impl JinaAIConnector {
    pub fn new(
        api_key: String,
        embedding_model: Option<String>,
        reranker_model: Option<String>,
    ) -> Self {
        Self {
            client: Client::new(),
            api_key,
            embedding_model: embedding_model.unwrap_or_else(|| "jina-embeddings-v2-base-en".to_string()),
            reranker_model: reranker_model.unwrap_or_else(|| "jina-reranker-v1-base-en".to_string()),
            base_url: "https://api.jina.ai/v1".to_string(),
        }
    }

    /// Generate embeddings for trading-related text
    pub async fn generate_embeddings(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        let request = JinaEmbeddingRequest {
            input: texts,
            model: self.embedding_model.clone(),
            task: Some("retrieval.passage".to_string()),
        };

        let response = timeout(
            Duration::from_secs(30),
            self.client
                .post(&format!("{}/embeddings", self.base_url))
                .header("Authorization", &format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&request)
                .send(),
        )
        .await
        .context("Timeout waiting for Jina AI embeddings response")?
        .context("Failed to send request to Jina AI")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Jina AI embeddings error: {}", error_text);
            return Err(anyhow::anyhow!("Jina AI embeddings failed: {}", error_text));
        }

        let embedding_response: JinaEmbeddingResponse = response
            .json()
            .await
            .context("Failed to parse Jina AI embeddings response")?;

        let embeddings: Vec<Vec<f32>> = embedding_response
            .data
            .into_iter()
            .map(|data| data.embedding)
            .collect();

        debug!("Generated {} embeddings using Jina AI", embeddings.len());
        Ok(embeddings)
    }

    /// Rerank documents based on relevance to query
    pub async fn rerank_documents(
        &self,
        query: String,
        documents: Vec<String>,
        top_n: Option<usize>,
    ) -> Result<Vec<RerankResult>> {
        let request = JinaRerankRequest {
            model: self.reranker_model.clone(),
            query,
            documents,
            top_n,
            return_documents: Some(true),
        };

        let response = timeout(
            Duration::from_secs(30),
            self.client
                .post(&format!("{}/rerank", self.base_url))
                .header("Authorization", &format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&request)
                .send(),
        )
        .await
        .context("Timeout waiting for Jina AI rerank response")?
        .context("Failed to send request to Jina AI")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Jina AI rerank error: {}", error_text);
            return Err(anyhow::anyhow!("Jina AI rerank failed: {}", error_text));
        }

        let rerank_response: JinaRerankResponse = response
            .json()
            .await
            .context("Failed to parse Jina AI rerank response")?;

        debug!("Reranked {} documents using Jina AI", rerank_response.results.len());
        Ok(rerank_response.results)
    }

    /// Generate embeddings for market analysis
    pub async fn embed_market_data(&self, market_data: &str) -> Result<Vec<f32>> {
        let embeddings = self.generate_embeddings(vec![market_data.to_string()]).await?;
        embeddings.into_iter().next()
            .ok_or_else(|| anyhow::anyhow!("No embeddings generated"))
    }

    /// Find most relevant trading signals
    pub async fn find_relevant_signals(
        &self,
        query: &str,
        signals: Vec<String>,
        top_k: usize,
    ) -> Result<Vec<(String, f64)>> {
        let results = self.rerank_documents(
            query.to_string(),
            signals,
            Some(top_k),
        ).await?;

        Ok(results.into_iter()
            .filter_map(|result| {
                result.document.map(|doc| (doc.text, result.relevance_score))
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_jina_connector_creation() {
        let connector = JinaAIConnector::new(
            "test-key".to_string(),
            None,
            None,
        );
        
        assert_eq!(connector.embedding_model, "jina-embeddings-v2-base-en");
        assert_eq!(connector.reranker_model, "jina-reranker-v1-base-en");
    }
}
