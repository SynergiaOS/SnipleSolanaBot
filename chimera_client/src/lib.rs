//! # CHIMERA Client - AI Communication Bridge for THE OVERMIND PROTOCOL
//!
//! This crate provides a robust, production-ready client for communicating with DeepSeek AI API.
//! Features include:
//! - Exponential backoff with jitter
//! - Circuit breaker pattern
//! - Comprehensive error handling
//! - Fallback to static rules
//! - Rate limiting protection

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

pub mod types;
pub mod client;
pub mod backoff;
pub mod circuit_breaker;
pub mod fallback;

pub use types::*;
pub use client::{ChimeraClient, ChimeraConfig, ClientStats};
pub use backoff::{ExponentialBackoff, BackoffStats};
pub use circuit_breaker::{CircuitBreaker, CircuitState, CircuitBreakerStats};
pub use fallback::{FallbackEngine, FallbackDecision, TradingAction, MarketCondition};

/// Main error type for CHIMERA Client operations
#[derive(Error, Debug)]
pub enum ChimeraError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("API error (status={status}): {message}")]
    Api {
        status: u16,
        message: String,
    },

    #[error("Rate limit exceeded. Retry after: {retry_after_seconds}s")]
    RateLimit {
        retry_after_seconds: u64,
    },

    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Circuit breaker is open - service temporarily unavailable")]
    CircuitBreakerOpen,

    #[error("Critical system failure: {0}")]
    Critical(String),

    #[error("Timeout error: {0}")]
    Timeout(String),
}

/// Result type alias for CHIMERA operations
pub type Result<T> = std::result::Result<T, ChimeraError>;
