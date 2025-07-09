//! Integration tests for CHIMERA Client
//! 
//! These tests verify the complete functionality of the CHIMERA Client
//! including error handling, retry logic, and fallback mechanisms.

use chimera_client::{
    ChimeraClient, ChimeraConfig, ChatCompletionRequest, ChatMessage,
    ExponentialBackoff, CircuitBreaker, CircuitState, FallbackEngine,
    MarketCondition, TradingAction,
};
use std::time::Duration;

/// Test basic client creation and configuration
#[tokio::test]
async fn test_client_creation() {
    let config = ChimeraConfig::new("test-api-key".to_string())
        .with_timeout(Duration::from_secs(10))
        .with_max_retries(3)
        .without_circuit_breaker()
        .without_fallback();
    
    let client = ChimeraClient::new(config);
    assert!(client.is_ok());
    
    let client = client.unwrap();
    let stats = client.stats().await;
    assert_eq!(stats.total_requests, 0);
    assert_eq!(stats.successful_requests, 0);
    assert_eq!(stats.failed_requests, 0);
}

/// Test configuration builder pattern
#[test]
fn test_config_builder() {
    let config = ChimeraConfig::new("test-key".to_string())
        .with_endpoint("https://custom.api.com".to_string())
        .with_timeout(Duration::from_secs(60))
        .with_max_retries(10)
        .without_circuit_breaker()
        .without_fallback();
    
    assert_eq!(config.api_key, "test-key");
    assert_eq!(config.api_endpoint, "https://custom.api.com");
    assert_eq!(config.timeout, Duration::from_secs(60));
    assert_eq!(config.max_retries, 10);
    assert!(!config.enable_circuit_breaker);
    assert!(!config.enable_fallback);
}

/// Test chat completion request creation
#[test]
fn test_request_creation() {
    let messages = vec![
        ChatMessage::system("You are a helpful assistant.".to_string()),
        ChatMessage::user("Hello!".to_string()),
    ];
    
    let request = ChatCompletionRequest::new("deepseek-chat".to_string(), messages)
        .with_temperature(0.7)
        .with_max_tokens(100)
        .with_json_output();
    
    assert_eq!(request.model, "deepseek-chat");
    assert_eq!(request.messages.len(), 2);
    assert_eq!(request.temperature, Some(0.7));
    assert_eq!(request.max_tokens, Some(100));
    assert!(request.response_format.is_some());
    
    if let Some(format) = request.response_format {
        assert_eq!(format.format_type, "json_object");
    }
}

/// Test exponential backoff functionality
#[tokio::test]
async fn test_exponential_backoff() {
    let mut backoff = ExponentialBackoff::new(10, 1000, 3, false);
    
    // Test initial state
    assert!(backoff.can_retry());
    assert_eq!(backoff.retry_count(), 0);
    
    // Test first backoff
    let start = std::time::Instant::now();
    let success = backoff.backoff().await;
    let elapsed = start.elapsed();
    
    assert!(success);
    assert!(elapsed >= Duration::from_millis(8)); // Allow some tolerance
    assert!(elapsed <= Duration::from_millis(20));
    assert_eq!(backoff.retry_count(), 1);
    
    // Test second backoff (should be longer)
    let start = std::time::Instant::now();
    let success = backoff.backoff().await;
    let elapsed = start.elapsed();
    
    assert!(success);
    assert!(elapsed >= Duration::from_millis(18)); // ~20ms expected
    assert!(elapsed <= Duration::from_millis(30));
    assert_eq!(backoff.retry_count(), 2);
    
    // Test third backoff
    let success = backoff.backoff().await;
    assert!(success);
    assert_eq!(backoff.retry_count(), 3);
    
    // Should not be able to retry anymore
    let success = backoff.backoff().await;
    assert!(!success);
    assert_eq!(backoff.retry_count(), 3);
}

/// Test circuit breaker functionality
#[test]
fn test_circuit_breaker() {
    let mut cb = CircuitBreaker::new(2, Duration::from_millis(100), 1);
    
    // Should start closed
    assert_eq!(cb.state(), &CircuitState::Closed);
    assert!(cb.can_execute());
    
    // First failure
    cb.record_failure();
    assert_eq!(cb.state(), &CircuitState::Closed);
    assert!(cb.can_execute());
    assert_eq!(cb.failure_count(), 1);
    
    // Second failure should open circuit
    cb.record_failure();
    assert_eq!(cb.state(), &CircuitState::Open);
    assert!(!cb.can_execute());
    assert_eq!(cb.failure_count(), 2);
    
    // Success should reset failure count when closed
    cb.force_close();
    cb.record_success();
    assert_eq!(cb.failure_count(), 0);
}

/// Test circuit breaker state transitions
#[tokio::test]
async fn test_circuit_breaker_transitions() {
    let mut cb = CircuitBreaker::new(1, Duration::from_millis(50), 1);
    
    // Open the circuit
    cb.record_failure();
    assert_eq!(cb.state(), &CircuitState::Open);
    assert!(!cb.can_execute());
    
    // Wait for timeout
    tokio::time::sleep(Duration::from_millis(60)).await;
    
    // Should transition to half-open
    assert!(cb.can_execute());
    assert_eq!(cb.state(), &CircuitState::HalfOpen);
    
    // Success should close the circuit
    cb.record_success();
    assert_eq!(cb.state(), &CircuitState::Closed);
    assert!(cb.can_execute());
}

/// Test fallback engine decisions
#[test]
fn test_fallback_engine() {
    let engine = FallbackEngine::new();
    
    // Test overbought condition
    let market = MarketCondition {
        price: 100.0,
        price_change_24h: 2.0,
        volume: 1000000.0,
        volatility: 0.3,
        rsi: Some(75.0), // Overbought
        ma_short: Some(98.0),
        ma_long: Some(95.0),
    };
    
    let decision = engine.make_decision(&market);
    assert_eq!(decision.action, TradingAction::Sell);
    assert!(decision.confidence > 0.5);
    
    // Test oversold condition
    let market = MarketCondition {
        price: 100.0,
        price_change_24h: -1.0,
        volume: 1000000.0,
        volatility: 0.3,
        rsi: Some(25.0), // Oversold
        ma_short: Some(98.0),
        ma_long: Some(95.0),
    };
    
    let decision = engine.make_decision(&market);
    assert_eq!(decision.action, TradingAction::Buy);
    assert!(decision.confidence > 0.5);
    
    // Test high volatility condition
    let market = MarketCondition {
        price: 100.0,
        price_change_24h: 1.0,
        volume: 1000000.0,
        volatility: 0.8, // High volatility
        rsi: Some(50.0),
        ma_short: Some(98.0),
        ma_long: Some(95.0),
    };
    
    let decision = engine.make_decision(&market);
    assert_eq!(decision.action, TradingAction::Hold);
    assert!(decision.risk_level > 0.8);
}

/// Test different fallback engine configurations
#[test]
fn test_fallback_engine_configurations() {
    // Conservative engine
    let conservative = FallbackEngine::conservative();
    let settings = conservative.settings();
    assert!(settings.risk_tolerance <= 0.1);
    assert!(settings.max_position_size <= 0.05);
    
    // Aggressive engine
    let aggressive = FallbackEngine::aggressive();
    let settings = aggressive.settings();
    assert!(settings.risk_tolerance >= 0.7);
    assert!(settings.max_position_size >= 0.2);
    
    // Default engine
    let default = FallbackEngine::default();
    let settings = default.settings();
    assert!(settings.risk_tolerance > 0.1 && settings.risk_tolerance < 0.7);
}

/// Test backoff with jitter
#[test]
fn test_backoff_jitter() {
    let backoff = ExponentialBackoff::new(1000, 10000, 3, true);
    
    // Test that jitter produces different values
    let delay1 = backoff.next_delay();
    let delay2 = backoff.next_delay();
    
    // Both should be around 1000ms but with jitter
    assert!(delay1.as_millis() >= 1000);
    assert!(delay1.as_millis() <= 1100);
    assert!(delay2.as_millis() >= 1000);
    assert!(delay2.as_millis() <= 1100);
}

/// Test backoff statistics
#[tokio::test]
async fn test_backoff_stats() {
    let mut backoff = ExponentialBackoff::new(100, 5000, 3, false);

    // Initial stats
    let stats = backoff.stats();
    assert_eq!(stats.current_retries, 0);
    assert_eq!(stats.max_retries, 3);
    assert_eq!(stats.base_delay_ms, 100);
    assert_eq!(stats.max_delay_ms, 5000);
    assert_eq!(stats.next_delay_ms, 100); // Base delay
    assert!(stats.can_retry);

    // After one backoff
    backoff.backoff().await;
    let stats = backoff.stats();
    assert_eq!(stats.current_retries, 1);
    assert_eq!(stats.next_delay_ms, 200); // 100 * 2^1
    assert!(stats.can_retry);
}

/// Test circuit breaker statistics
#[test]
fn test_circuit_breaker_stats() {
    let mut cb = CircuitBreaker::new(3, Duration::from_secs(30), 2);
    cb.record_failure();
    cb.record_failure();
    
    let stats = cb.stats();
    assert_eq!(stats.state, CircuitState::Closed);
    assert_eq!(stats.failure_count, 2);
    assert_eq!(stats.failure_threshold, 3);
    assert_eq!(stats.success_threshold, 2);
    assert!(stats.time_until_half_open.is_none());
}

/// Test message creation helpers
#[test]
fn test_message_helpers() {
    let system_msg = ChatMessage::system("System prompt".to_string());
    assert_eq!(system_msg.role, "system");
    assert_eq!(system_msg.content, "System prompt");
    
    let user_msg = ChatMessage::user("User message".to_string());
    assert_eq!(user_msg.role, "user");
    assert_eq!(user_msg.content, "User message");
    
    let assistant_msg = ChatMessage::assistant("Assistant response".to_string());
    assert_eq!(assistant_msg.role, "assistant");
    assert_eq!(assistant_msg.content, "Assistant response");
}
