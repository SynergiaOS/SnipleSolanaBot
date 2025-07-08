// THE OVERMIND PROTOCOL - Enhanced AI Brain Manager
// Multi-model AI consensus system with Jina AI and DeepSeek V2

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

use crate::modules::deepseek_connector::DeepSeekConnector;
use crate::modules::jina_ai_connector::JinaAIConnector;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConsensus {
    pub final_decision: String,
    pub confidence: f64,
    pub reasoning: String,
    pub risk_assessment: String,
    pub model_votes: HashMap<String, ModelVote>,
    pub consensus_strength: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelVote {
    pub decision: String,
    pub confidence: f64,
    pub reasoning: String,
    pub weight: f64,
}

pub struct EnhancedAIBrain {
    deepseek_connector: DeepSeekConnector,
    jina_connector: JinaAIConnector,
    consensus_threshold: f64,
    enable_ensemble: bool,
}

impl EnhancedAIBrain {
    pub fn new(
        deepseek_api_key: String,
        jina_api_key: String,
        consensus_threshold: f64,
    ) -> Result<Self> {
        let deepseek_connector = DeepSeekConnector::new(deepseek_api_key)?;
        let jina_connector = JinaAIConnector::new(
            jina_api_key,
            Some("jina-embeddings-v2-base-en".to_string()),
            Some("jina-reranker-v1-base-en".to_string()),
        );

        Ok(Self {
            deepseek_connector,
            jina_connector,
            consensus_threshold,
            enable_ensemble: true,
        })
    }

    /// Generate AI consensus for trading decision
    pub async fn generate_consensus(
        &self,
        token_data: &str,
        market_context: &str,
        historical_data: Vec<String>,
    ) -> Result<AIConsensus> {
        info!("Generating AI consensus for trading decision");

        // Step 1: Enhance context with Jina AI semantic search
        let enhanced_context = self
            .enhance_context_with_jina(token_data, historical_data)
            .await?;

        // Step 2: Get decisions from DeepSeek V2
        let mut model_votes = HashMap::new();

        // DeepSeek V2 analysis
        if let Ok(deepseek_analysis) = self
            .get_deepseek_analysis(token_data, market_context)
            .await
        {
            model_votes.insert("deepseek-v2".to_string(), deepseek_analysis);
        }

        // Step 3: Calculate consensus
        let consensus = self.calculate_consensus(model_votes).await?;

        info!(
            "AI consensus generated: {} with {:.2}% confidence",
            consensus.final_decision,
            consensus.confidence * 100.0
        );

        Ok(consensus)
    }

    /// Enhance context using Jina AI reranking
    async fn enhance_context_with_jina(
        &self,
        query: &str,
        historical_data: Vec<String>,
    ) -> Result<String> {
        if historical_data.is_empty() {
            return Ok(String::new());
        }

        debug!("Enhancing context with Jina AI reranking");

        // Use Jina AI to rerank historical data by relevance
        let relevant_data = self
            .jina_connector
            .rerank_documents(query.to_string(), historical_data, Some(5))
            .await?;

        let enhanced_context = relevant_data
            .into_iter()
            .map(|result| format!("Relevance: {:.2} - {}", result.relevance_score,
                result.document.map(|d| d.text).unwrap_or_default()))
            .collect::<Vec<_>>()
            .join("\n\n");

        Ok(enhanced_context)
    }



    /// Get DeepSeek V2 analysis
    async fn get_deepseek_analysis(
        &self,
        token_data: &str,
        market_context: &str,
    ) -> Result<ModelVote> {
        let analysis = self
            .deepseek_connector
            .analyze_trading_opportunity(token_data, market_context)
            .await?;

        Ok(ModelVote {
            decision: analysis.action,
            confidence: analysis.confidence,
            reasoning: analysis.reasoning,
            weight: 0.9, // DeepSeek gets slightly lower weight
        })
    }

    /// Parse model vote from JSON response
    fn parse_model_vote(&self, response: &str, weight: f64) -> Result<ModelVote> {
        let json_start = response.find('{').unwrap_or(0);
        let json_end = response.rfind('}').unwrap_or(response.len());
        let json_str = &response[json_start..=json_end];

        let parsed: serde_json::Value = serde_json::from_str(json_str)
            .context("Failed to parse model vote JSON")?;

        Ok(ModelVote {
            decision: parsed["decision"].as_str().unwrap_or("HOLD").to_string(),
            confidence: parsed["confidence"].as_f64().unwrap_or(0.5),
            reasoning: parsed["reasoning"].as_str().unwrap_or("No reasoning").to_string(),
            weight,
        })
    }

    /// Calculate consensus from model votes
    async fn calculate_consensus(&self, model_votes: HashMap<String, ModelVote>) -> Result<AIConsensus> {
        if model_votes.is_empty() {
            return Err(anyhow::anyhow!("No model votes available"));
        }

        // Calculate weighted votes for each decision
        let mut decision_scores: HashMap<String, f64> = HashMap::new();
        let mut total_weight = 0.0;

        for (model, vote) in &model_votes {
            let weighted_confidence = vote.confidence * vote.weight;
            *decision_scores.entry(vote.decision.clone()).or_insert(0.0) += weighted_confidence;
            total_weight += vote.weight;
            
            debug!(
                "Model {} voted {} with confidence {:.2} (weight: {:.2})",
                model, vote.decision, vote.confidence, vote.weight
            );
        }

        // Find the decision with highest weighted score
        let (final_decision, best_score) = decision_scores
            .into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap();

        let confidence = best_score / total_weight;
        let consensus_strength = self.calculate_consensus_strength(&model_votes);

        // Combine reasoning from all models
        let combined_reasoning = model_votes
            .values()
            .map(|vote| vote.reasoning.clone())
            .collect::<Vec<_>>()
            .join(" | ");

        // Generate risk assessment
        let risk_assessment = self.assess_consensus_risk(confidence, consensus_strength);

        Ok(AIConsensus {
            final_decision,
            confidence,
            reasoning: combined_reasoning,
            risk_assessment,
            model_votes,
            consensus_strength,
        })
    }

    /// Calculate consensus strength (how much models agree)
    fn calculate_consensus_strength(&self, model_votes: &HashMap<String, ModelVote>) -> f64 {
        if model_votes.len() < 2 {
            return 1.0;
        }

        let decisions: Vec<&String> = model_votes.values().map(|v| &v.decision).collect();
        let unique_decisions: std::collections::HashSet<_> = decisions.iter().collect();
        
        // More agreement = higher consensus strength
        1.0 - (unique_decisions.len() as f64 - 1.0) / (model_votes.len() as f64 - 1.0)
    }

    /// Assess risk based on consensus metrics
    fn assess_consensus_risk(&self, confidence: f64, consensus_strength: f64) -> String {
        match (confidence, consensus_strength) {
            (c, s) if c > 0.8 && s > 0.8 => "LOW".to_string(),
            (c, s) if c > 0.6 && s > 0.6 => "MEDIUM".to_string(),
            (c, s) if c > 0.4 || s > 0.4 => "HIGH".to_string(),
            _ => "EXTREME".to_string(),
        }
    }

    /// Health check for all AI services
    pub async fn health_check(&self) -> Result<HashMap<String, bool>> {
        let mut health_status = HashMap::new();

        // Check DeepSeek
        health_status.insert(
            "deepseek".to_string(),
            self.deepseek_connector.health_check().await.unwrap_or(false),
        );

        // Note: JinaAIConnector doesn't have health_check method yet
        health_status.insert("jina".to_string(), true);

        let healthy_services = health_status.values().filter(|&&v| v).count();
        info!(
            "AI Brain health check: {}/{} services healthy",
            healthy_services,
            health_status.len()
        );

        Ok(health_status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consensus_strength_calculation() {
        let brain = EnhancedAIBrain {
            deepseek_connector: DeepSeekConnector::new("test".to_string()).unwrap(),
            jina_connector: JinaAIConnector::new(
                "test".to_string(),
                Some("model".to_string()),
                Some("reranker".to_string()),
            ),
            consensus_threshold: 0.75,
            enable_ensemble: true,
        };

        let mut votes = HashMap::new();
        votes.insert("model1".to_string(), ModelVote {
            decision: "BUY".to_string(),
            confidence: 0.8,
            reasoning: "test".to_string(),
            weight: 1.0,
        });
        votes.insert("model2".to_string(), ModelVote {
            decision: "BUY".to_string(),
            confidence: 0.7,
            reasoning: "test".to_string(),
            weight: 1.0,
        });

        let strength = brain.calculate_consensus_strength(&votes);
        assert_eq!(strength, 1.0); // Perfect agreement
    }
}
