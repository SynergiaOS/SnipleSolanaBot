//! Circuit breaker implementation for CHIMERA Client
//! 
//! Provides protection against cascading failures by temporarily disabling
//! requests when the service is experiencing issues.

use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    /// Circuit is closed - requests flow normally
    Closed,
    /// Circuit is open - requests are blocked
    Open,
    /// Circuit is half-open - testing if service recovered
    HalfOpen,
}

/// Circuit breaker for protecting against service failures
#[derive(Debug)]
pub struct CircuitBreaker {
    /// Current state of the circuit
    state: CircuitState,
    
    /// Number of consecutive failures
    failure_count: u32,
    
    /// Threshold for opening the circuit
    failure_threshold: u32,
    
    /// Duration to keep circuit open
    timeout_duration: Duration,
    
    /// Time when circuit was opened
    last_failure_time: Option<Instant>,
    
    /// Number of successful requests in half-open state
    success_count: u32,
    
    /// Required successes to close circuit from half-open
    success_threshold: u32,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(
        failure_threshold: u32,
        timeout_duration: Duration,
        success_threshold: u32,
    ) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            failure_threshold,
            timeout_duration,
            last_failure_time: None,
            success_count: 0,
            success_threshold,
        }
    }
    
    /// Create default circuit breaker for API calls
    pub fn default_api() -> Self {
        Self::new(
            5,                              // 5 failures to open
            Duration::from_secs(30),        // 30s timeout
            3,                              // 3 successes to close
        )
    }
    
    /// Create aggressive circuit breaker for critical services
    pub fn critical_service() -> Self {
        Self::new(
            3,                              // 3 failures to open
            Duration::from_secs(60),        // 60s timeout
            5,                              // 5 successes to close
        )
    }
    
    /// Check if requests are allowed through the circuit
    pub fn can_execute(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed() >= self.timeout_duration {
                        debug!("Circuit breaker transitioning to half-open");
                        self.state = CircuitState::HalfOpen;
                        self.success_count = 0;
                        true
                    } else {
                        debug!("Circuit breaker is open - blocking request");
                        false
                    }
                } else {
                    // This shouldn't happen, but handle gracefully
                    warn!("Circuit breaker in open state without failure time");
                    self.state = CircuitState::Closed;
                    true
                }
            }
            CircuitState::HalfOpen => true,
        }
    }
    
    /// Record a successful operation
    pub fn record_success(&mut self) {
        match self.state {
            CircuitState::Closed => {
                // Reset failure count on success
                if self.failure_count > 0 {
                    debug!("Resetting failure count after success");
                    self.failure_count = 0;
                }
            }
            CircuitState::HalfOpen => {
                self.success_count += 1;
                debug!(
                    "Half-open success {}/{}", 
                    self.success_count, 
                    self.success_threshold
                );
                
                if self.success_count >= self.success_threshold {
                    info!("Circuit breaker closing after successful recovery");
                    self.state = CircuitState::Closed;
                    self.failure_count = 0;
                    self.success_count = 0;
                    self.last_failure_time = None;
                }
            }
            CircuitState::Open => {
                // This shouldn't happen
                warn!("Received success while circuit is open");
            }
        }
    }
    
    /// Record a failed operation
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(Instant::now());
        
        match self.state {
            CircuitState::Closed => {
                debug!(
                    "Failure {}/{} in closed state", 
                    self.failure_count, 
                    self.failure_threshold
                );
                
                if self.failure_count >= self.failure_threshold {
                    error!(
                        "Circuit breaker opening after {} failures", 
                        self.failure_count
                    );
                    self.state = CircuitState::Open;
                }
            }
            CircuitState::HalfOpen => {
                error!("Circuit breaker reopening after failure in half-open state");
                self.state = CircuitState::Open;
                self.success_count = 0;
            }
            CircuitState::Open => {
                debug!("Additional failure while circuit is open");
            }
        }
    }
    
    /// Force the circuit to open (for testing or manual intervention)
    pub fn force_open(&mut self) {
        warn!("Circuit breaker manually forced open");
        self.state = CircuitState::Open;
        self.last_failure_time = Some(Instant::now());
    }
    
    /// Force the circuit to close (for testing or manual intervention)
    pub fn force_close(&mut self) {
        info!("Circuit breaker manually forced closed");
        self.state = CircuitState::Closed;
        self.failure_count = 0;
        self.success_count = 0;
        self.last_failure_time = None;
    }
    
    /// Get current circuit state
    pub fn state(&self) -> &CircuitState {
        &self.state
    }
    
    /// Get current failure count
    pub fn failure_count(&self) -> u32 {
        self.failure_count
    }
    
    /// Get time remaining until circuit can transition to half-open
    pub fn time_until_half_open(&self) -> Option<Duration> {
        if self.state == CircuitState::Open {
            if let Some(last_failure) = self.last_failure_time {
                let elapsed = last_failure.elapsed();
                if elapsed < self.timeout_duration {
                    Some(self.timeout_duration - elapsed)
                } else {
                    Some(Duration::ZERO)
                }
            } else {
                None
            }
        } else {
            None
        }
    }
    
    /// Get statistics about circuit breaker state
    pub fn stats(&self) -> CircuitBreakerStats {
        CircuitBreakerStats {
            state: self.state.clone(),
            failure_count: self.failure_count,
            failure_threshold: self.failure_threshold,
            success_count: self.success_count,
            success_threshold: self.success_threshold,
            time_until_half_open: self.time_until_half_open(),
        }
    }
}

/// Statistics about circuit breaker state
#[derive(Debug, Clone)]
pub struct CircuitBreakerStats {
    pub state: CircuitState,
    pub failure_count: u32,
    pub failure_threshold: u32,
    pub success_count: u32,
    pub success_threshold: u32,
    pub time_until_half_open: Option<Duration>,
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::default_api()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[test]
    fn test_circuit_breaker_creation() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(10), 2);
        assert_eq!(cb.state(), &CircuitState::Closed);
        assert_eq!(cb.failure_count(), 0);
        assert!(cb.time_until_half_open().is_none());
    }
    
    #[test]
    fn test_closed_to_open_transition() {
        let mut cb = CircuitBreaker::new(2, Duration::from_secs(10), 1);
        
        // Should start closed and allow requests
        assert!(cb.can_execute());
        assert_eq!(cb.state(), &CircuitState::Closed);
        
        // First failure
        cb.record_failure();
        assert!(cb.can_execute());
        assert_eq!(cb.state(), &CircuitState::Closed);
        assert_eq!(cb.failure_count(), 1);
        
        // Second failure should open circuit
        cb.record_failure();
        assert!(!cb.can_execute());
        assert_eq!(cb.state(), &CircuitState::Open);
        assert_eq!(cb.failure_count(), 2);
    }
    
    #[test]
    fn test_success_resets_failures() {
        let mut cb = CircuitBreaker::new(3, Duration::from_secs(10), 1);
        
        // Record some failures
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.failure_count(), 2);
        assert_eq!(cb.state(), &CircuitState::Closed);
        
        // Success should reset failure count
        cb.record_success();
        assert_eq!(cb.failure_count(), 0);
        assert_eq!(cb.state(), &CircuitState::Closed);
    }
    
    #[tokio::test]
    async fn test_open_to_half_open_transition() {
        let mut cb = CircuitBreaker::new(1, Duration::from_millis(50), 1);
        
        // Force circuit open
        cb.record_failure();
        assert_eq!(cb.state(), &CircuitState::Open);
        assert!(!cb.can_execute());
        
        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(60)).await;
        
        // Should transition to half-open
        assert!(cb.can_execute());
        assert_eq!(cb.state(), &CircuitState::HalfOpen);
    }
    
    #[test]
    fn test_half_open_to_closed() {
        let mut cb = CircuitBreaker::new(1, Duration::from_secs(10), 2);
        
        // Force to half-open state
        cb.state = CircuitState::HalfOpen;
        
        // First success
        cb.record_success();
        assert_eq!(cb.state(), &CircuitState::HalfOpen);
        
        // Second success should close circuit
        cb.record_success();
        assert_eq!(cb.state(), &CircuitState::Closed);
        assert_eq!(cb.failure_count(), 0);
    }
    
    #[test]
    fn test_half_open_to_open() {
        let mut cb = CircuitBreaker::new(1, Duration::from_secs(10), 2);
        
        // Force to half-open state
        cb.state = CircuitState::HalfOpen;
        
        // Failure should reopen circuit
        cb.record_failure();
        assert_eq!(cb.state(), &CircuitState::Open);
    }
    
    #[test]
    fn test_force_operations() {
        let mut cb = CircuitBreaker::new(1, Duration::from_secs(10), 1);
        
        // Force open
        cb.force_open();
        assert_eq!(cb.state(), &CircuitState::Open);
        assert!(!cb.can_execute());
        
        // Force close
        cb.force_close();
        assert_eq!(cb.state(), &CircuitState::Closed);
        assert!(cb.can_execute());
        assert_eq!(cb.failure_count(), 0);
    }
    
    #[test]
    fn test_stats() {
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
}
