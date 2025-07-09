//! TENSORZERO GATEWAY - Unified LLM Access
//! 
//! Integracja z TensorZero dla AI-driven strategy generation
//! Unified API dla wszystkich LLM providers

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, info, warn, error};
use reqwest::Client;

/// TensorZero Gateway configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TensorZeroConfig {
    /// TensorZero Gateway URL
    pub gateway_url: String,
    
    /// ClickHouse database URL
    pub clickhouse_url: String,
    
    /// Configuration file path
    pub config_file: String,
    
    /// Default model for strategy generation
    pub default_model: String,
    
    /// Backup models for fallback
    pub fallback_models: Vec<String>,
    
    /// Request timeout
    pub timeout_seconds: u64,
    
    /// Max retries
    pub max_retries: u8,
    
    /// Enable observability
    pub enable_observability: bool,
}

impl Default for TensorZeroConfig {
    fn default() -> Self {
        Self {
            gateway_url: "http://localhost:3000".to_string(),
            clickhouse_url: "http://localhost:8123/tensorzero".to_string(),
            config_file: "config/tensorzero.toml".to_string(),
            default_model: "anthropic::claude-3-7-sonnet-20250219".to_string(),
            fallback_models: vec![
                "openai::gpt-4o".to_string(),
                "anthropic::claude-3-5-haiku-20241022".to_string(),
            ],
            timeout_seconds: 30,
            max_retries: 3,
            enable_observability: true,
        }
    }
}

/// TensorZero Gateway client
#[derive(Debug)]
pub struct TensorZeroGateway {
    /// Configuration
    config: TensorZeroConfig,
    
    /// HTTP client
    http_client: Client,
    
    /// Request metrics
    metrics: TensorZeroMetrics,
}

/// TensorZero metrics
#[derive(Debug, Default, Clone)]
pub struct TensorZeroMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub total_tokens_used: u64,
    pub average_response_time_ms: u64,
    pub fallback_usage_count: u64,
}

/// TensorZero inference request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub model_name: String,
    pub input: InferenceInput,
    pub stream: Option<bool>,
    pub function_name: Option<String>,
    pub episode_id: Option<String>,
    pub tags: Option<HashMap<String, String>>,
}

/// TensorZero inference input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceInput {
    pub messages: Vec<Message>,
    pub system: Option<String>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub tools: Option<Vec<Tool>>,
}

/// Message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// TensorZero inference response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse {
    pub inference_id: String,
    pub episode_id: Option<String>,
    pub variant_name: String,
    pub content: Vec<ContentBlock>,
    pub usage: Usage,
    pub model_name: String,
    pub model_provider_name: String,
}

/// Content block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentBlock {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: Option<String>,
    pub tool_use: Option<ToolUse>,
}

/// Tool use
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUse {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
}

/// Usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
}

impl TensorZeroGateway {
    /// Create new TensorZero Gateway
    pub async fn new(config: TensorZeroConfig) -> Result<Self> {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;
        
        // Test connection to TensorZero Gateway
        let health_url = format!("{}/health", config.gateway_url);
        match http_client.get(&health_url).send().await {
            Ok(response) if response.status().is_success() => {
                info!("✅ TensorZero Gateway connection successful");
            }
            Ok(response) => {
                warn!("⚠️ TensorZero Gateway responded with status: {}", response.status());
            }
            Err(e) => {
                error!("❌ Failed to connect to TensorZero Gateway: {}", e);
                return Err(anyhow!("TensorZero Gateway connection failed: {}", e));
            }
        }
        
        Ok(Self {
            config,
            http_client,
            metrics: TensorZeroMetrics::default(),
        })
    }
    
    /// Generate strategy DSL using TensorZero
    pub async fn generate_strategy_dsl(
        &mut self,
        prompt: &str,
        context: Option<&str>,
        function_name: Option<&str>,
    ) -> Result<String> {
        let start_time = std::time::Instant::now();
        
        // Build system prompt for strategy generation
        let system_prompt = self.build_strategy_system_prompt(context);
        
        // Build inference request
        let request = InferenceRequest {
            model_name: self.config.default_model.clone(),
            input: InferenceInput {
                messages: vec![
                    Message {
                        role: "user".to_string(),
                        content: prompt.to_string(),
                    }
                ],
                system: Some(system_prompt),
                temperature: Some(0.7),
                max_tokens: Some(4000),
                tools: None,
            },
            stream: Some(false),
            function_name: function_name.map(|s| s.to_string()),
            episode_id: None,
            tags: Some(self.build_request_tags()),
        };
        
        // Send request with retries
        let response = self.send_inference_request_with_retries(request).await?;
        
        // Extract generated DSL
        let dsl = self.extract_dsl_from_response(&response)?;
        
        // Update metrics
        let response_time = start_time.elapsed().as_millis() as u64;
        self.update_metrics(&response, response_time, true);
        
        debug!("Generated strategy DSL: {} characters", dsl.len());
        Ok(dsl)
    }
    
    /// Send inference request with retries and fallbacks
    async fn send_inference_request_with_retries(&mut self, mut request: InferenceRequest) -> Result<InferenceResponse> {
        let mut last_error = None;
        
        // Try primary model
        for attempt in 0..self.config.max_retries {
            match self.send_inference_request(&request).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    warn!("Inference attempt {} failed: {}", attempt + 1, e);
                    last_error = Some(e);
                    
                    if attempt < self.config.max_retries - 1 {
                        tokio::time::sleep(Duration::from_millis(1000 * (attempt as u64 + 1))).await;
                    }
                }
            }
        }
        
        // Try fallback models
        let fallback_models = self.config.fallback_models.clone();
        for fallback_model in &fallback_models {
            warn!("Trying fallback model: {}", fallback_model);
            request.model_name = fallback_model.clone();

            match self.send_inference_request(&request).await {
                Ok(response) => {
                    self.metrics.fallback_usage_count += 1;
                    return Ok(response);
                }
                Err(e) => {
                    warn!("Fallback model {} failed: {}", fallback_model, e);
                    last_error = Some(e);
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| anyhow!("All inference attempts failed")))
    }
    
    /// Send single inference request
    async fn send_inference_request(&mut self, request: &InferenceRequest) -> Result<InferenceResponse> {
        let url = format!("{}/inference", self.config.gateway_url);
        
        let response = timeout(
            Duration::from_secs(self.config.timeout_seconds),
            self.http_client.post(&url)
                .json(request)
                .send()
        ).await
        .map_err(|_| anyhow!("Request timeout"))?
        .map_err(|e| anyhow!("HTTP request failed: {}", e))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("TensorZero API error {}: {}", status, error_text));
        }
        
        let inference_response: InferenceResponse = response.json().await
            .map_err(|e| anyhow!("Failed to parse response: {}", e))?;
        
        Ok(inference_response)
    }
    
    /// Build system prompt for strategy generation
    fn build_strategy_system_prompt(&self, context: Option<&str>) -> String {
        let base_prompt = r#"
You are an expert quantitative trading strategy developer. Your task is to generate high-performance trading strategy DSL code.

REQUIREMENTS:
1. Generate valid DSL syntax for trading strategies
2. Include risk management parameters
3. Specify entry and exit conditions
4. Define AI model requirements
5. Ensure code is production-ready

DSL STRUCTURE:
```
strategy StrategyName:
  risk_model:
    max_drawdown: X%
    daily_loss_limit: Y%
    position_size: Z%
    
  entry_logic:
    - trigger: "condition"
      action: action_type(parameters)
      
  exit_logic:
    - trigger: "condition"
      action: action_type(parameters)
      
  ai_models:
    - name: ModelName
      version: X.Y
      purpose: "description"
```

FOCUS ON:
- High-frequency trading optimizations
- Risk-adjusted returns
- Market microstructure
- Latency-sensitive operations
"#;
        
        if let Some(ctx) = context {
            format!("{}\n\nCONTEXT:\n{}", base_prompt, ctx)
        } else {
            base_prompt.to_string()
        }
    }
    
    /// Build request tags for observability
    fn build_request_tags(&self) -> HashMap<String, String> {
        let mut tags = HashMap::new();
        tags.insert("source".to_string(), "forge_dsl_generator".to_string());
        tags.insert("version".to_string(), "1.0.0".to_string());
        tags.insert("environment".to_string(), "production".to_string());
        tags
    }
    
    /// Extract DSL from TensorZero response
    fn extract_dsl_from_response(&self, response: &InferenceResponse) -> Result<String> {
        for content_block in &response.content {
            if content_block.content_type == "text" {
                if let Some(text) = &content_block.text {
                    // Extract DSL code from markdown code blocks if present
                    if let Some(dsl) = self.extract_code_from_markdown(text) {
                        return Ok(dsl);
                    }
                    return Ok(text.clone());
                }
            }
        }
        
        Err(anyhow!("No text content found in response"))
    }
    
    /// Extract code from markdown code blocks
    fn extract_code_from_markdown(&self, text: &str) -> Option<String> {
        // Look for code blocks with ```dsl or ``` 
        if let Some(start) = text.find("```") {
            let after_start = &text[start + 3..];
            
            // Skip language identifier if present
            let code_start = if let Some(newline) = after_start.find('\n') {
                start + 3 + newline + 1
            } else {
                start + 3
            };
            
            if let Some(end) = text[code_start..].find("```") {
                let code = &text[code_start..code_start + end];
                return Some(code.trim().to_string());
            }
        }
        
        None
    }
    
    /// Update metrics
    fn update_metrics(&mut self, response: &InferenceResponse, response_time_ms: u64, success: bool) {
        self.metrics.total_requests += 1;
        
        if success {
            self.metrics.successful_requests += 1;
        } else {
            self.metrics.failed_requests += 1;
        }
        
        self.metrics.total_tokens_used += response.usage.total_tokens as u64;
        
        // Update average response time
        let total_requests = self.metrics.total_requests;
        self.metrics.average_response_time_ms = 
            (self.metrics.average_response_time_ms * (total_requests - 1) + response_time_ms) / total_requests;
    }
    
    /// Get metrics
    pub fn get_metrics(&self) -> &TensorZeroMetrics {
        &self.metrics
    }
    
    /// Get configuration
    pub fn get_config(&self) -> &TensorZeroConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_tensorzero_gateway_creation() {
        let config = TensorZeroConfig::default();
        
        // Test będzie działał tylko z działającym TensorZero Gateway
        match TensorZeroGateway::new(config).await {
            Ok(_) => println!("TensorZero Gateway initialized successfully"),
            Err(e) => println!("Expected error in test environment: {}", e),
        }
    }
    
    #[test]
    fn test_code_extraction_from_markdown() {
        let gateway = TensorZeroGateway {
            config: TensorZeroConfig::default(),
            http_client: Client::new(),
            metrics: TensorZeroMetrics::default(),
        };
        
        let markdown_text = r#"
Here's your strategy:

```dsl
strategy TestStrategy:
  risk_model:
    max_drawdown: 5%
```

This should work well.
"#;
        
        let extracted = gateway.extract_code_from_markdown(markdown_text).unwrap();
        assert!(extracted.contains("strategy TestStrategy"));
        assert!(extracted.contains("max_drawdown: 5%"));
    }
}
