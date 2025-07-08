// THE OVERMIND PROTOCOL - DeepSeek V2 Connector
// Advanced reasoning and analysis for trading decisions

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone)]
pub struct DeepSeekConnector {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: u32,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
    usage: Usage,
    model: String,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ResponseMessage,
    finish_reason: String,
    index: u32,
}

#[derive(Debug, Deserialize)]
struct ResponseMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Clone)]
pub struct TradingAnalysis {
    pub confidence: f64,
    pub action: String,
    pub reasoning: String,
    pub risk_level: String,
    pub expected_return: f64,
}

impl DeepSeekConnector {
    pub fn new(api_key: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            api_key,
            base_url: "https://api.deepseek.com/v1".to_string(),
            model: "deepseek-chat".to_string(),
        })
    }

    /// Analyze trading opportunity using DeepSeek V2
    pub async fn analyze_trading_opportunity(
        &self,
        token_data: &str,
        market_context: &str,
    ) -> Result<TradingAnalysis> {
        let prompt = format!(
            r#"You are an expert crypto trading analyst. Analyze this trading opportunity:

TOKEN DATA:
{}

MARKET CONTEXT:
{}

Provide analysis in this exact JSON format:
{{
    "confidence": 0.85,
    "action": "BUY|SELL|HOLD",
    "reasoning": "Detailed reasoning for the decision",
    "risk_level": "LOW|MEDIUM|HIGH|EXTREME",
    "expected_return": 0.15
}}

Focus on:
1. Technical indicators
2. Market sentiment
3. Risk/reward ratio
4. Entry/exit timing
5. Position sizing recommendations

Be precise and data-driven."#,
            token_data, market_context
        );

        let response = self.chat_completion(prompt).await?;
        self.parse_trading_analysis(&response)
    }

    /// Generate market sentiment analysis
    pub async fn analyze_market_sentiment(
        &self,
        social_data: &str,
        news_data: &str,
    ) -> Result<String> {
        let prompt = format!(
            r#"Analyze market sentiment from this data:

SOCIAL MEDIA DATA:
{}

NEWS DATA:
{}

Provide a concise sentiment analysis (1-2 paragraphs) focusing on:
1. Overall market mood (bullish/bearish/neutral)
2. Key sentiment drivers
3. Potential market moving events
4. Confidence level in sentiment assessment"#,
            social_data, news_data
        );

        self.chat_completion(prompt).await
    }

    /// Analyze token fundamentals
    pub async fn analyze_token_fundamentals(
        &self,
        token_address: &str,
        on_chain_data: &str,
    ) -> Result<String> {
        let prompt = format!(
            r#"Analyze token fundamentals for address: {}

ON-CHAIN DATA:
{}

Provide fundamental analysis covering:
1. Tokenomics (supply, distribution, inflation)
2. Utility and use cases
3. Team and development activity
4. Community strength
5. Competitive positioning
6. Long-term viability

Rate each aspect 1-10 and provide overall score."#,
            token_address, on_chain_data
        );

        self.chat_completion(prompt).await
    }

    /// Generate risk assessment
    pub async fn assess_risk(
        &self,
        position_data: &str,
        market_conditions: &str,
    ) -> Result<String> {
        let prompt = format!(
            r#"Assess trading risk for this position:

POSITION DATA:
{}

MARKET CONDITIONS:
{}

Provide comprehensive risk assessment:
1. Market risk (volatility, liquidity)
2. Technical risk (support/resistance levels)
3. Fundamental risk (token-specific risks)
4. Systemic risk (broader market factors)
5. Risk mitigation strategies
6. Recommended position size
7. Stop-loss levels

Rate overall risk: LOW/MEDIUM/HIGH/EXTREME"#,
            position_data, market_conditions
        );

        self.chat_completion(prompt).await
    }

    /// Private method for chat completion
    async fn chat_completion(&self, prompt: String) -> Result<String> {
        let request = ChatRequest {
            model: self.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt,
            }],
            temperature: 0.1, // Low temperature for consistent analysis
            max_tokens: 2000,
            stream: false,
        };

        debug!("Sending request to DeepSeek V2");

        let response = timeout(
            Duration::from_secs(60),
            self.client
                .post(&format!("{}/chat/completions", self.base_url))
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&request)
                .send(),
        )
        .await
        .context("Request timeout")?
        .context("Failed to send request")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("DeepSeek API error: {}", error_text);
            return Err(anyhow::anyhow!("DeepSeek API error: {}", error_text));
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .context("Failed to parse response")?;

        if chat_response.choices.is_empty() {
            return Err(anyhow::anyhow!("No response choices received"));
        }

        let content = chat_response.choices[0].message.content.clone();
        
        info!(
            "DeepSeek analysis completed. Tokens used: {}",
            chat_response.usage.total_tokens
        );

        Ok(content)
    }

    /// Parse trading analysis from JSON response
    fn parse_trading_analysis(&self, response: &str) -> Result<TradingAnalysis> {
        // Try to extract JSON from response
        let json_start = response.find('{').unwrap_or(0);
        let json_end = response.rfind('}').unwrap_or(response.len());
        let json_str = &response[json_start..=json_end];

        let parsed: serde_json::Value = serde_json::from_str(json_str)
            .context("Failed to parse trading analysis JSON")?;

        Ok(TradingAnalysis {
            confidence: parsed["confidence"].as_f64().unwrap_or(0.5),
            action: parsed["action"].as_str().unwrap_or("HOLD").to_string(),
            reasoning: parsed["reasoning"].as_str().unwrap_or("No reasoning provided").to_string(),
            risk_level: parsed["risk_level"].as_str().unwrap_or("MEDIUM").to_string(),
            expected_return: parsed["expected_return"].as_f64().unwrap_or(0.0),
        })
    }

    /// Health check for DeepSeek service
    pub async fn health_check(&self) -> Result<bool> {
        let test_prompt = "Respond with 'OK' if you can process this message.".to_string();
        
        match self.chat_completion(test_prompt).await {
            Ok(response) => {
                let is_healthy = response.to_lowercase().contains("ok");
                if is_healthy {
                    info!("DeepSeek health check passed");
                } else {
                    warn!("DeepSeek health check failed: unexpected response");
                }
                Ok(is_healthy)
            }
            Err(e) => {
                warn!("DeepSeek health check failed: {}", e);
                Ok(false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_deepseek_connector_creation() {
        let connector = DeepSeekConnector::new("test-key".to_string());
        assert!(connector.is_ok());
    }

    #[test]
    fn test_parse_trading_analysis() {
        let connector = DeepSeekConnector::new("test".to_string()).unwrap();
        let json_response = r#"{"confidence": 0.85, "action": "BUY", "reasoning": "Strong fundamentals", "risk_level": "MEDIUM", "expected_return": 0.15}"#;
        
        let analysis = connector.parse_trading_analysis(json_response);
        assert!(analysis.is_ok());
        
        let analysis = analysis.unwrap();
        assert_eq!(analysis.confidence, 0.85);
        assert_eq!(analysis.action, "BUY");
    }
}
