use reqwest::Client;
use serde_json::{json, Value};
use anyhow::{Result, anyhow};
use std::time::Duration;
use tracing::{info, error};

pub struct ChimeraAI {
    client: Client,
    api_key: String,
    model: String,
}

impl ChimeraAI {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap(),
            api_key,
            model: "claude-3-5-sonnet-20240620".to_string(),
        }
    }

    pub async fn analyze_market(&self, data: &MarketData) -> Result<Decision> {
        let prompt = format!("Analyze Solana memecoin market: {:?}", data);
        
        info!("Sending market analysis request to Claude 3.5 Sonnet");
        
        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&json!({
                "model": self.model,
                "max_tokens": 1024,
                "messages": [{"role": "user", "content": prompt}]
            }))
            .send().await?
            .json::<Value>().await?;

        // Weryfikacja odpowiedzi
        if let Some(content) = response.get("content") {
            if let Some(text) = content[0].get("text") {
                let decision: Decision = serde_json::from_value(text.clone())?;
                return Ok(decision);
            }
        }
        
        error!("Failed to parse AI response: {:?}", response);
        Err(anyhow!("Invalid AI response format"))
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct MarketData {
    pub symbol: String,
    pub price: f64,
    pub volume: f64,
    pub market_cap: f64,
    pub change_24h: f64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Decision {
    pub action: Action,
    pub confidence: f64,
    pub reasoning: String,
    pub risk_assessment: f64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Action {
    Buy,
    Sell,
    Hold,
}