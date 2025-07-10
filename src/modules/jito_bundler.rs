// THE OVERMIND PROTOCOL - Jito Bundler Module
// Advanced Jito bundle handling with exponential backoff and error recovery

use anyhow::Result;
use backoff::ExponentialBackoff;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone)]
pub struct JitoBundler {
    pub auth_key: String,
    pub endpoint: String,
    pub backoff_strategy: ExponentialBackoff,
    pub max_retries: usize,
    pub timeout: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JitoBundle {
    pub transactions: Vec<String>,
    pub tip_account: String,
    pub tip_amount: u64,
    pub bundle_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleResponse {
    pub signature: String,
    pub bundle_id: String,
    pub status: String,
    pub slot: Option<u64>,
    pub confirmation_time_ms: Option<u64>,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum JitoError {
    #[error("Slot skew detected - bundle timing issue")]
    SlotSkew,
    #[error("Invalid fee account - incorrect tip account")]
    InvalidFeeAccount,
    #[error("Bundle timeout - execution took too long")]
    BundleTimeout,
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl JitoBundler {
    /// Create new Jito bundler with advanced configuration
    pub fn new(key: String, endpoint: String) -> Self {
        let mut backoff = ExponentialBackoff::default();
        backoff.initial_interval = Duration::from_millis(100);
        backoff.multiplier = 2.0;
        backoff.max_interval = Duration::from_secs(5);
        backoff.max_elapsed_time = Some(Duration::from_secs(30));

        Self {
            auth_key: key,
            endpoint,
            backoff_strategy: backoff,
            max_retries: 5,
            timeout: Duration::from_secs(10),
        }
    }

    /// Create bundler with custom configuration
    pub fn with_config(
        key: String,
        endpoint: String,
        max_retries: usize,
        timeout: Duration,
    ) -> Self {
        let mut bundler = Self::new(key, endpoint);
        bundler.max_retries = max_retries;
        bundler.timeout = timeout;
        bundler
    }

    /// Send bundle with advanced retry logic and error handling
    pub async fn send_bundle(&self, bundle: JitoBundle) -> Result<BundleResponse, JitoError> {
        info!("🚀 Sending Jito bundle: {}", bundle.bundle_id);
        
        let mut retries = 0;
        let client = reqwest::Client::builder()
            .timeout(self.timeout)
            .build()
            .map_err(|e| JitoError::NetworkError(format!("Failed to create HTTP client: {}", e)))?;
        
        let start_time = Instant::now();
        let mut last_error = None;

        while retries < self.max_retries {
            let attempt_start = Instant::now();
            
            debug!("📤 Bundle attempt {} for {}", retries + 1, bundle.bundle_id);

            match self.send_bundle_attempt(&client, &bundle).await {
                Ok(response) => {
                    let total_time = start_time.elapsed();
                    info!("✅ Bundle {} sent successfully in {:?} (attempt {})", 
                          bundle.bundle_id, total_time, retries + 1);
                    return Ok(response);
                }
                Err(e) => {
                    last_error = Some(e.clone());
                    
                    match &e {
                        JitoError::SlotSkew => {
                            retries += 1;
                            if retries >= self.max_retries {
                                error!("❌ Max retries reached for slot skew on bundle {}", bundle.bundle_id);
                                return Err(e);
                            }
                            
                            let delay = Duration::from_millis(100 * retries as u64);
                            warn!("⏳ Slot skew detected, retrying in {:?} (attempt {})", delay, retries + 1);
                            sleep(delay).await;
                        }
                        JitoError::InvalidFeeAccount => {
                            error!("❌ Invalid fee account for bundle {}: {}", bundle.bundle_id, e);
                            return Err(e);
                        }
                        JitoError::BundleTimeout => {
                            retries += 1;
                            if retries >= self.max_retries {
                                error!("❌ Bundle {} timed out after {} attempts", bundle.bundle_id, retries);
                                return Err(e);
                            }
                            
                            warn!("⏰ Bundle timeout, retrying (attempt {})", retries + 1);
                            sleep(Duration::from_millis(200)).await;
                        }
                        JitoError::NetworkError(_) => {
                            retries += 1;
                            if retries >= self.max_retries {
                                error!("❌ Network error persists for bundle {}: {}", bundle.bundle_id, e);
                                return Err(e);
                            }
                            
                            let delay = self.calculate_backoff_delay(retries);
                            warn!("🌐 Network error, retrying in {:?} (attempt {}): {}", delay, retries + 1, e);
                            sleep(delay).await;
                        }
                        JitoError::ApiError(_) => {
                            retries += 1;
                            if retries >= self.max_retries {
                                error!("❌ API error persists for bundle {}: {}", bundle.bundle_id, e);
                                return Err(e);
                            }
                            
                            let delay = self.calculate_backoff_delay(retries);
                            warn!("🔌 API error, retrying in {:?} (attempt {}): {}", delay, retries + 1, e);
                            sleep(delay).await;
                        }
                        JitoError::SerializationError(_) => {
                            error!("❌ Serialization error for bundle {}: {}", bundle.bundle_id, e);
                            return Err(e);
                        }
                    }
                }
            }

            // Check total elapsed time
            if start_time.elapsed() > Duration::from_secs(30) {
                error!("❌ Total timeout exceeded for bundle {}", bundle.bundle_id);
                return Err(JitoError::BundleTimeout);
            }
        }

        Err(last_error.unwrap_or(JitoError::BundleTimeout))
    }

    /// Single bundle send attempt
    async fn send_bundle_attempt(&self, client: &reqwest::Client, bundle: &JitoBundle) -> Result<BundleResponse, JitoError> {
        let request_payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "sendBundle",
            "params": {
                "transactions": bundle.transactions,
                "tip_account": bundle.tip_account,
                "tip_amount": bundle.tip_amount
            }
        });

        debug!("📡 Sending bundle request to: {}", self.endpoint);

        let response = client
            .post(&self.endpoint)
            .header("Authorization", &self.auth_key)
            .header("Content-Type", "application/json")
            .json(&request_payload)
            .send()
            .await
            .map_err(|e| JitoError::NetworkError(format!("Request failed: {}", e)))?;

        let status = response.status();
        let response_text = response.text().await
            .map_err(|e| JitoError::NetworkError(format!("Failed to read response: {}", e)))?;

        if status.is_success() {
            self.parse_success_response(&response_text, bundle)
        } else {
            self.handle_error_response(status.as_u16(), &response_text)
        }
    }

    /// Parse successful response
    fn parse_success_response(&self, response_text: &str, bundle: &JitoBundle) -> Result<BundleResponse, JitoError> {
        let json_response: serde_json::Value = serde_json::from_str(response_text)
            .map_err(|e| JitoError::SerializationError(format!("Failed to parse JSON: {}", e)))?;

        if let Some(result) = json_response.get("result") {
            let signature = result.as_str()
                .ok_or_else(|| JitoError::SerializationError("Missing signature in response".to_string()))?;

            Ok(BundleResponse {
                signature: signature.to_string(),
                bundle_id: bundle.bundle_id.clone(),
                status: "submitted".to_string(),
                slot: None,
                confirmation_time_ms: None,
            })
        } else if let Some(error) = json_response.get("error") {
            let error_msg = error.get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            
            Err(JitoError::ApiError(error_msg.to_string()))
        } else {
            Err(JitoError::SerializationError("Invalid response format".to_string()))
        }
    }

    /// Handle error responses with specific error type detection
    fn handle_error_response(&self, status_code: u16, response_text: &str) -> Result<BundleResponse, JitoError> {
        let error_msg = response_text.to_lowercase();

        if error_msg.contains("fee account") || error_msg.contains("tip account") {
            Err(JitoError::InvalidFeeAccount)
        } else if error_msg.contains("slot skew") || error_msg.contains("slot") {
            Err(JitoError::SlotSkew)
        } else if status_code == 408 || error_msg.contains("timeout") {
            Err(JitoError::BundleTimeout)
        } else {
            Err(JitoError::ApiError(format!("HTTP {}: {}", status_code, response_text)))
        }
    }

    /// Calculate exponential backoff delay
    fn calculate_backoff_delay(&self, retry_count: usize) -> Duration {
        let base_delay = Duration::from_millis(100);
        let multiplier = 2_u64.pow(retry_count as u32);
        let delay = Duration::from_millis(base_delay.as_millis() as u64 * multiplier);
        
        // Cap at max interval
        if delay > Duration::from_secs(5) {
            Duration::from_secs(5)
        } else {
            delay
        }
    }

    /// Create bundle from transactions
    pub fn create_bundle(
        &self,
        transactions: Vec<String>,
        tip_account: String,
        tip_amount: u64,
    ) -> JitoBundle {
        let bundle_id = format!("bundle_{}", uuid::Uuid::new_v4());
        
        JitoBundle {
            transactions,
            tip_account,
            tip_amount,
            bundle_id,
        }
    }

    /// Validate bundle before sending
    pub fn validate_bundle(&self, bundle: &JitoBundle) -> Result<(), JitoError> {
        if bundle.transactions.is_empty() {
            return Err(JitoError::SerializationError("Bundle cannot be empty".to_string()));
        }

        if bundle.transactions.len() > 5 {
            return Err(JitoError::SerializationError("Bundle too large (max 5 transactions)".to_string()));
        }

        // Validate tip account format
        if bundle.tip_account.len() != 44 {
            return Err(JitoError::InvalidFeeAccount);
        }

        // Validate tip amount
        if bundle.tip_amount == 0 {
            warn!("⚠️ Bundle has zero tip amount");
        }

        Ok(())
    }

    /// Get bundle status
    pub async fn get_bundle_status(&self, bundle_id: &str) -> Result<BundleResponse, JitoError> {
        let client = reqwest::Client::new();
        
        let request_payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getBundleStatus",
            "params": {
                "bundle_id": bundle_id
            }
        });

        let response = client
            .post(&self.endpoint)
            .header("Authorization", &self.auth_key)
            .header("Content-Type", "application/json")
            .json(&request_payload)
            .send()
            .await
            .map_err(|e| JitoError::NetworkError(format!("Status request failed: {}", e)))?;

        let response_text = response.text().await
            .map_err(|e| JitoError::NetworkError(format!("Failed to read status response: {}", e)))?;

        let json_response: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| JitoError::SerializationError(format!("Failed to parse status JSON: {}", e)))?;

        if let Some(result) = json_response.get("result") {
            Ok(BundleResponse {
                signature: result.get("signature").and_then(|s| s.as_str()).unwrap_or("").to_string(),
                bundle_id: bundle_id.to_string(),
                status: result.get("status").and_then(|s| s.as_str()).unwrap_or("unknown").to_string(),
                slot: result.get("slot").and_then(|s| s.as_u64()),
                confirmation_time_ms: result.get("confirmation_time_ms").and_then(|s| s.as_u64()),
            })
        } else {
            Err(JitoError::ApiError("Failed to get bundle status".to_string()))
        }
    }
}
