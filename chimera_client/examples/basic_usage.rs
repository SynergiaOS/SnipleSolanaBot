//! Basic usage example for CHIMERA Client
//! 
//! This example demonstrates how to use the CHIMERA Client to communicate
//! with the DeepSeek AI API for trading decisions.

use chimera_client::{
    ChimeraClient, ChimeraConfig, ChatCompletionRequest, ChatMessage,
    Result, ChimeraError,
};
use std::env;
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Get API key from environment
    let api_key = env::var("DEEPSEEK_API_KEY")
        .expect("DEEPSEEK_API_KEY environment variable must be set");
    
    info!("🚀 Starting CHIMERA Client example");
    
    // Create client configuration
    let config = ChimeraConfig::new(api_key)
        .with_timeout(std::time::Duration::from_secs(30))
        .with_max_retries(3);
    
    // Initialize CHIMERA client
    let client = ChimeraClient::new(config)?;
    
    // Create a trading analysis request
    let messages = vec![
        ChatMessage::system(
            "You are an expert cryptocurrency trading analyst. Analyze market conditions and provide trading recommendations in JSON format.".to_string()
        ),
        ChatMessage::user(
            "Analyze the current market conditions for SOL/USDT. The price is $95.50, up 3.2% in 24h, with RSI at 65. Volume is above average. Should I buy, sell, or hold? Provide your analysis in JSON format with action, confidence, and reasoning.".to_string()
        ),
    ];
    
    let request = ChatCompletionRequest::new("deepseek-chat".to_string(), messages)
        .with_temperature(0.3)
        .with_max_tokens(500)
        .with_json_output();
    
    info!("📡 Sending trading analysis request to AI...");
    
    // Execute the request
    match client.execute_reasoning_task(request).await {
        Ok(response) => {
            info!("✅ AI Response received:");
            println!("{}", response);
            
            // Display client statistics
            let stats = client.stats().await;
            info!("📊 Client Statistics:");
            info!("  Total requests: {}", stats.total_requests);
            info!("  Successful requests: {}", stats.successful_requests);
            info!("  Failed requests: {}", stats.failed_requests);
            info!("  Fallback decisions: {}", stats.fallback_decisions);
            info!("  Circuit breaker trips: {}", stats.circuit_breaker_trips);
            info!("  Total retry attempts: {}", stats.total_retry_attempts);
        }
        Err(ChimeraError::CircuitBreakerOpen) => {
            error!("🔴 Circuit breaker is open - service temporarily unavailable");
            
            // Show circuit breaker status
            if let Some(cb_stats) = client.circuit_breaker_status().await {
                error!("Circuit breaker state: {:?}", cb_stats.state);
                if let Some(time_until_half_open) = cb_stats.time_until_half_open {
                    error!("Time until retry: {:?}", time_until_half_open);
                }
            }
        }
        Err(ChimeraError::RateLimit { retry_after_seconds }) => {
            error!("🟡 Rate limited - retry after {} seconds", retry_after_seconds);
        }
        Err(ChimeraError::Authentication(msg)) => {
            error!("🔑 Authentication failed: {}", msg);
            error!("Please check your DEEPSEEK_API_KEY environment variable");
        }
        Err(error) => {
            error!("❌ Request failed: {}", error);
            
            // Show fallback decision if available
            let stats = client.stats().await;
            if stats.fallback_decisions > 0 {
                info!("🔄 Fallback logic was used {} times", stats.fallback_decisions);
            }
        }
    }
    
    info!("🏁 CHIMERA Client example completed");
    Ok(())
}
