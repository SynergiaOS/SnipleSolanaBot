//! Main CHIMERA Client implementation
//! 
//! Provides the core ChimeraClient struct with comprehensive error handling,
//! retry logic, circuit breaker protection, and fallback capabilities.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn, instrument};

use crate::{
    ChimeraError, Result,
    types::{ChatCompletionRequest, ChatCompletionResponse, ApiErrorResponse, ChatMessage},
    backoff::ExponentialBackoff,
    circuit_breaker::CircuitBreaker,
    fallback::{FallbackEngine, MarketCondition, FallbackDecision},
};

/// Configuration for CHIMERA Client
#[derive(Debug, Clone)]
pub struct ChimeraConfig {
    /// DeepSeek API key
    pub api_key: String,
    
    /// API endpoint URL
    pub api_endpoint: String,
    
    /// Request timeout duration
    pub timeout: Duration,
    
    /// Maximum retries for failed requests
    pub max_retries: u32,
    
    /// Enable circuit breaker protection
    pub enable_circuit_breaker: bool,
    
    /// Enable fallback to static rules
    pub enable_fallback: bool,
    
    /// User agent string
    pub user_agent: String,
}

impl ChimeraConfig {
    /// Create new configuration with API key
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            api_endpoint: "https://api.deepseek.com".to_string(),
            timeout: Duration::from_secs(30),
            max_retries: 5,
            enable_circuit_breaker: true,
            enable_fallback: true,
            user_agent: "CHIMERA-Client/1.0 (THE-OVERMIND-PROTOCOL)".to_string(),
        }
    }
    
    /// Set custom API endpoint
    pub fn with_endpoint(mut self, endpoint: String) -> Self {
        self.api_endpoint = endpoint;
        self
    }
    
    /// Set request timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    
    /// Set maximum retries
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }
    
    /// Disable circuit breaker (not recommended for production)
    pub fn without_circuit_breaker(mut self) -> Self {
        self.enable_circuit_breaker = false;
        self
    }
    
    /// Disable fallback logic
    pub fn without_fallback(mut self) -> Self {
        self.enable_fallback = false;
        self
    }
}

/// Main CHIMERA Client for AI communication
pub struct ChimeraClient {
    /// Client configuration
    config: ChimeraConfig,
    
    /// Exponential backoff for retries
    backoff: Arc<Mutex<ExponentialBackoff>>,
    
    /// Circuit breaker for failure protection
    circuit_breaker: Arc<Mutex<CircuitBreaker>>,
    
    /// Fallback engine for static decisions
    fallback_engine: FallbackEngine,
    
    /// Request statistics
    stats: Arc<Mutex<ClientStats>>,
}

/// Client statistics
#[derive(Debug, Default, Clone)]
pub struct ClientStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub fallback_decisions: u64,
    pub circuit_breaker_trips: u64,
    pub total_retry_attempts: u64,
}

impl ChimeraClient {
    /// Create a new CHIMERA client
    pub fn new(config: ChimeraConfig) -> Result<Self> {
        let backoff = Arc::new(Mutex::new(ExponentialBackoff::default_api()));
        let circuit_breaker = Arc::new(Mutex::new(CircuitBreaker::default_api()));
        let fallback_engine = FallbackEngine::new();
        let stats = Arc::new(Mutex::new(ClientStats::default()));

        info!("CHIMERA Client initialized with endpoint: {}", config.api_endpoint);

        Ok(Self {
            config,
            backoff,
            circuit_breaker,
            fallback_engine,
            stats,
        })
    }
    
    /// Execute binary task for THE OVERMIND PROTOCOL v4.4 "GEOHOT CORE"
    #[instrument(skip(self, prompt_data))]
    pub async fn execute_binary(&self, prompt_data: &[u8]) -> Result<Vec<u8>> {
        // Convert binary prompt to text for API compatibility
        let prompt_text = String::from_utf8_lossy(prompt_data);

        let request = ChatCompletionRequest {
            model: "deepseek-chat".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt_text.to_string(),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            }],
            temperature: Some(0.1), // Low temperature for consistent results
            max_tokens: Some(100),   // Short responses for binary protocol
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stop: None,
            stream: Some(false),
            response_format: None,
            tools: None,
            tool_choice: None,
        };

        // Execute through standard reasoning task
        let response_text = self.execute_reasoning_task(request).await?;

        // Convert response back to binary
        Ok(response_text.into_bytes())
    }

    /// Execute a reasoning task with the AI
    #[instrument(skip(self, request))]
    pub async fn execute_reasoning_task(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<String> {
        let start_time = Instant::now();
        
        // Update stats
        {
            let mut stats = self.stats.lock().await;
            stats.total_requests += 1;
        }
        
        // Check circuit breaker
        if self.config.enable_circuit_breaker {
            let mut cb = self.circuit_breaker.lock().await;
            if !cb.can_execute() {
                warn!("Circuit breaker is open - request blocked");
                
                // Update stats
                {
                    let mut stats = self.stats.lock().await;
                    stats.circuit_breaker_trips += 1;
                }
                
                return if self.config.enable_fallback {
                    self.execute_fallback_logic().await
                } else {
                    Err(ChimeraError::CircuitBreakerOpen)
                };
            }
        }
        
        // Attempt the request with retries
        let result = self.send_request_with_retries(request).await;
        
        // Update circuit breaker based on result
        if self.config.enable_circuit_breaker {
            let mut cb = self.circuit_breaker.lock().await;
            match &result {
                Ok(_) => cb.record_success(),
                Err(ChimeraError::Api { status, .. }) if *status >= 500 => {
                    cb.record_failure();
                }
                Err(ChimeraError::Network(_)) => {
                    cb.record_failure();
                }
                Err(ChimeraError::Timeout(_)) => {
                    cb.record_failure();
                }
                _ => {} // Don't count client errors as circuit breaker failures
            }
        }
        
        // Handle result
        match result {
            Ok(response) => {
                let duration = start_time.elapsed();
                info!("AI request completed successfully in {:?}", duration);
                
                // Update stats
                {
                    let mut stats = self.stats.lock().await;
                    stats.successful_requests += 1;
                }
                
                Ok(response)
            }
            Err(error) => {
                let duration = start_time.elapsed();
                error!("AI request failed after {:?}: {}", duration, error);
                
                // Update stats
                {
                    let mut stats = self.stats.lock().await;
                    stats.failed_requests += 1;
                }
                
                // Try fallback if enabled and appropriate
                if self.config.enable_fallback && self.should_use_fallback(&error) {
                    warn!("Falling back to static rules due to: {}", error);
                    self.execute_fallback_logic().await
                } else {
                    Err(error)
                }
            }
        }
    }
    
    /// Send request with retry logic
    async fn send_request_with_retries(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<String> {
        let mut backoff = self.backoff.lock().await;
        backoff.reset();
        
        loop {
            match self.send_single_request(&request).await {
                Ok(response) => {
                    debug!("Request succeeded");
                    return Ok(response);
                }
                Err(error) => {
                    match &error {
                        ChimeraError::RateLimit { retry_after_seconds } => {
                            warn!("Rate limited, waiting {}s", retry_after_seconds);
                            tokio::time::sleep(Duration::from_secs(*retry_after_seconds)).await;
                            continue;
                        }
                        ChimeraError::Api { status, .. } if *status >= 500 => {
                            // Server error - retry with backoff
                            if backoff.can_retry() {
                                warn!("Server error {}, retrying...", status);
                                
                                // Update stats
                                {
                                    let mut stats = self.stats.lock().await;
                                    stats.total_retry_attempts += 1;
                                }
                                
                                backoff.backoff().await;
                                continue;
                            }
                        }
                        ChimeraError::Network(_) | ChimeraError::Timeout(_) => {
                            // Network/timeout error - retry with backoff
                            if backoff.can_retry() {
                                warn!("Network/timeout error, retrying...");
                                
                                // Update stats
                                {
                                    let mut stats = self.stats.lock().await;
                                    stats.total_retry_attempts += 1;
                                }
                                
                                backoff.backoff().await;
                                continue;
                            }
                        }
                        _ => {
                            // Client error or other - don't retry
                            debug!("Non-retryable error: {}", error);
                        }
                    }
                    
                    return Err(error);
                }
            }
        }
    }
    
    /// Send a single HTTP request to the API using minreq (HOTZ PHILOSOPHY)
    async fn send_single_request(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<String> {
        let url = format!("{}/chat/completions", self.config.api_endpoint);

        debug!("Sending request to: {}", url);

        // Serialize request using serde_json (will migrate to tinyjson later)
        let request_json = serde_json::to_string(request)
            .map_err(|e| ChimeraError::Serialization(format!("Request serialization error: {}", e)))?;

        // Use minreq for HTTP request (zero dependencies)
        let response = tokio::task::spawn_blocking({
            let url = url.clone();
            let api_key = self.config.api_key.clone();
            let user_agent = self.config.user_agent.clone();
            let timeout_secs = self.config.timeout.as_secs();

            move || {
                minreq::post(&url)
                    .with_header("Authorization", format!("Bearer {}", api_key))
                    .with_header("Content-Type", "application/json")
                    .with_header("User-Agent", user_agent)
                    .with_timeout(timeout_secs)
                    .with_body(request_json)
                    .send()
            }
        })
        .await
        .map_err(|e| ChimeraError::Network(format!("Task join error: {}", e)))?
        .map_err(|e| ChimeraError::Network(format!("HTTP request error: {}", e)))?;

        let status_code = response.status_code;
        debug!("Received response with status: {}", status_code);

        match status_code {
            200 => {
                let response_text = response.as_str()
                    .map_err(|e| ChimeraError::Network(format!("Response decode error: {}", e)))?;

                let response_body: ChatCompletionResponse = serde_json::from_str(response_text)
                    .map_err(|e| ChimeraError::Serialization(format!("JSON parse error: {}", e)))?;
                
                // Extract content from first choice
                if let Some(choice) = response_body.choices.first() {
                    Ok(choice.message.content.clone())
                } else {
                    Err(ChimeraError::Api {
                        status: 200,
                        message: "No choices in response".to_string(),
                    })
                }
            }
            429 => {
                // Parse retry-after header if present
                let retry_after = response.headers.get("retry-after")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(60); // Default to 60 seconds

                Err(ChimeraError::RateLimit {
                    retry_after_seconds: retry_after,
                })
            }
            401 => {
                Err(ChimeraError::Authentication(
                    "Invalid API key".to_string()
                ))
            }
            _ => {
                // Try to parse error response
                let default_error = format!("HTTP {} error", status_code);
                let error_text = response.as_str()
                    .unwrap_or(&default_error);

                // Try to parse as API error response
                if let Ok(api_error) = serde_json::from_str::<ApiErrorResponse>(error_text) {
                    Err(ChimeraError::Api {
                        status: status_code as u16,
                        message: api_error.error.message,
                    })
                } else {
                    Err(ChimeraError::Api {
                        status: status_code as u16,
                        message: error_text.to_string(),
                    })
                }
            }
        }
    }
    
    /// Execute fallback logic when AI is unavailable
    async fn execute_fallback_logic(&self) -> Result<String> {
        info!("Executing fallback logic - using static trading rules");
        
        // Update stats
        {
            let mut stats = self.stats.lock().await;
            stats.fallback_decisions += 1;
        }
        
        // Create sample market condition (in real implementation, this would come from market data)
        let market_condition = MarketCondition {
            price: 100.0,
            price_change_24h: 0.0,
            volume: 1000000.0,
            volatility: 0.3,
            rsi: Some(50.0),
            ma_short: Some(100.0),
            ma_long: Some(100.0),
        };
        
        let decision = self.fallback_engine.make_decision(&market_condition);
        
        let fallback_response = serde_json::to_string(&decision)
            .map_err(|e| ChimeraError::Serialization(format!("Fallback serialization error: {}", e)))?;
        
        info!("Fallback decision: {:?}", decision.action);
        Ok(fallback_response)
    }
    
    /// Determine if fallback should be used for this error
    fn should_use_fallback(&self, error: &ChimeraError) -> bool {
        match error {
            ChimeraError::Network(_) => true,
            ChimeraError::Api { status, .. } if *status >= 500 => true,
            ChimeraError::Timeout(_) => true,
            ChimeraError::CircuitBreakerOpen => true,
            ChimeraError::Critical(_) => true,
            _ => false,
        }
    }
    
    /// Get client statistics
    pub async fn stats(&self) -> ClientStats {
        let stats = self.stats.lock().await;
        ClientStats {
            total_requests: stats.total_requests,
            successful_requests: stats.successful_requests,
            failed_requests: stats.failed_requests,
            fallback_decisions: stats.fallback_decisions,
            circuit_breaker_trips: stats.circuit_breaker_trips,
            total_retry_attempts: stats.total_retry_attempts,
        }
    }

    /// Create mock client for testing
    #[cfg(test)]
    pub fn mock() -> Self {
        let config = ChimeraConfig::new("mock_api_key".to_string());
        Self::new(config).expect("Failed to create mock client")
    }
    
    /// Reset client statistics
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.lock().await;
        *stats = ClientStats::default();
        info!("Client statistics reset");
    }
    
    /// Get circuit breaker status
    pub async fn circuit_breaker_status(&self) -> Option<crate::circuit_breaker::CircuitBreakerStats> {
        if self.config.enable_circuit_breaker {
            Some(self.circuit_breaker.lock().await.stats())
        } else {
            None
        }
    }
    
    /// Get backoff status
    pub async fn backoff_status(&self) -> crate::backoff::BackoffStats {
        self.backoff.lock().await.stats()
    }
}
