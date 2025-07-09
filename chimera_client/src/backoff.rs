//! Exponential backoff implementation with jitter for CHIMERA Client
//! 
//! Provides intelligent retry logic with exponential delays and random jitter
//! to avoid thundering herd problems when multiple clients retry simultaneously.

use std::time::Duration;
use nanorand::{Rng, WyRand};
use tracing::{debug, warn};

/// Exponential backoff strategy with configurable jitter
#[derive(Debug, Clone)]
pub struct ExponentialBackoff {
    /// Base delay in milliseconds
    base_delay_ms: u64,
    
    /// Maximum delay in milliseconds
    max_delay_ms: u64,
    
    /// Current retry attempt count
    current_retries: u32,
    
    /// Maximum number of retries
    max_retries: u32,
    
    /// Whether to use jitter
    use_jitter: bool,
    
    /// Multiplier for exponential growth
    multiplier: f64,
}

impl ExponentialBackoff {
    /// Create a new exponential backoff strategy
    pub fn new(
        base_delay_ms: u64,
        max_delay_ms: u64,
        max_retries: u32,
        use_jitter: bool,
    ) -> Self {
        Self {
            base_delay_ms,
            max_delay_ms,
            current_retries: 0,
            max_retries,
            use_jitter,
            multiplier: 2.0,
        }
    }
    
    /// Create default backoff strategy for API calls
    pub fn default_api() -> Self {
        Self::new(
            100,    // 100ms base delay
            30000,  // 30s max delay
            5,      // 5 max retries
            true,   // use jitter
        )
    }
    
    /// Create aggressive backoff for rate limiting
    pub fn rate_limit() -> Self {
        Self::new(
            1000,   // 1s base delay
            60000,  // 60s max delay
            3,      // 3 max retries
            true,   // use jitter
        )
    }
    
    /// Reset the backoff state
    pub fn reset(&mut self) {
        self.current_retries = 0;
        debug!("Exponential backoff reset");
    }
    
    /// Check if we can retry
    pub fn can_retry(&self) -> bool {
        self.current_retries < self.max_retries
    }
    
    /// Get current retry count
    pub fn retry_count(&self) -> u32 {
        self.current_retries
    }
    
    /// Calculate next delay duration
    pub fn next_delay(&self) -> Duration {
        let exponential_delay = self.base_delay_ms as f64 * self.multiplier.powi(self.current_retries as i32);
        let capped_delay = exponential_delay.min(self.max_delay_ms as f64) as u64;
        
        let final_delay = if self.use_jitter {
            self.add_jitter(capped_delay)
        } else {
            capped_delay
        };
        
        debug!(
            "Calculated backoff delay: {}ms (attempt {}/{})",
            final_delay,
            self.current_retries + 1,
            self.max_retries
        );
        
        Duration::from_millis(final_delay)
    }
    
    /// Perform backoff delay and increment retry count
    pub async fn backoff(&mut self) -> bool {
        if !self.can_retry() {
            warn!("Maximum retries ({}) exceeded", self.max_retries);
            return false;
        }
        
        let delay = self.next_delay();
        self.current_retries += 1;
        
        debug!(
            "Backing off for {:?} (attempt {}/{})",
            delay,
            self.current_retries,
            self.max_retries
        );
        
        tokio::time::sleep(delay).await;
        true
    }
    
    /// Add random jitter to delay to prevent thundering herd
    fn add_jitter(&self, delay_ms: u64) -> u64 {
        let jitter_range = (delay_ms as f64 * 0.1) as u64; // 10% jitter
        let mut rng = WyRand::new();
        let jitter = rng.generate_range(0..=jitter_range);
        delay_ms + jitter
    }
    
    /// Set custom multiplier for exponential growth
    pub fn with_multiplier(mut self, multiplier: f64) -> Self {
        self.multiplier = multiplier.max(1.0); // Ensure multiplier is at least 1.0
        self
    }
    
    /// Get statistics about current backoff state
    pub fn stats(&self) -> BackoffStats {
        BackoffStats {
            current_retries: self.current_retries,
            max_retries: self.max_retries,
            base_delay_ms: self.base_delay_ms,
            max_delay_ms: self.max_delay_ms,
            next_delay_ms: if self.can_retry() {
                self.next_delay().as_millis() as u64
            } else {
                0
            },
            can_retry: self.can_retry(),
        }
    }
}

/// Statistics about backoff state
#[derive(Debug, Clone)]
pub struct BackoffStats {
    pub current_retries: u32,
    pub max_retries: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub next_delay_ms: u64,
    pub can_retry: bool,
}

impl Default for ExponentialBackoff {
    fn default() -> Self {
        Self::default_api()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;
    
    #[test]
    fn test_backoff_creation() {
        let backoff = ExponentialBackoff::new(100, 5000, 3, true);
        assert_eq!(backoff.base_delay_ms, 100);
        assert_eq!(backoff.max_delay_ms, 5000);
        assert_eq!(backoff.max_retries, 3);
        assert!(backoff.use_jitter);
        assert!(backoff.can_retry());
    }
    
    #[test]
    fn test_delay_calculation() {
        let backoff = ExponentialBackoff::new(100, 5000, 3, false);
        
        // First delay should be base delay
        let delay1 = backoff.next_delay();
        assert_eq!(delay1, Duration::from_millis(100));
        
        // Create backoff with one retry already done
        let mut backoff = ExponentialBackoff::new(100, 5000, 3, false);
        backoff.current_retries = 1;
        let delay2 = backoff.next_delay();
        assert_eq!(delay2, Duration::from_millis(200));
        
        // Test max delay capping
        backoff.current_retries = 10; // Very high retry count
        let delay_max = backoff.next_delay();
        assert_eq!(delay_max, Duration::from_millis(5000));
    }
    
    #[test]
    fn test_retry_limits() {
        let mut backoff = ExponentialBackoff::new(100, 5000, 2, false);
        
        assert!(backoff.can_retry());
        assert_eq!(backoff.retry_count(), 0);
        
        // Simulate first retry
        backoff.current_retries = 1;
        assert!(backoff.can_retry());
        assert_eq!(backoff.retry_count(), 1);
        
        // Simulate second retry
        backoff.current_retries = 2;
        assert!(!backoff.can_retry());
        assert_eq!(backoff.retry_count(), 2);
    }
    
    #[test]
    fn test_reset() {
        let mut backoff = ExponentialBackoff::new(100, 5000, 3, false);
        backoff.current_retries = 2;
        
        assert_eq!(backoff.retry_count(), 2);
        backoff.reset();
        assert_eq!(backoff.retry_count(), 0);
        assert!(backoff.can_retry());
    }
    
    #[test]
    fn test_jitter() {
        let backoff = ExponentialBackoff::new(1000, 10000, 3, true);
        
        // Test that jitter produces different values
        let delay1 = backoff.add_jitter(1000);
        let delay2 = backoff.add_jitter(1000);
        
        // Both should be >= 1000 (original delay)
        assert!(delay1 >= 1000);
        assert!(delay2 >= 1000);
        
        // Both should be <= 1100 (original + 10% jitter)
        assert!(delay1 <= 1100);
        assert!(delay2 <= 1100);
    }
    
    #[tokio::test]
    async fn test_backoff_timing() {
        let mut backoff = ExponentialBackoff::new(50, 1000, 2, false);
        
        let start = Instant::now();
        let success = backoff.backoff().await;
        let elapsed = start.elapsed();
        
        assert!(success);
        assert!(elapsed >= Duration::from_millis(45)); // Allow some tolerance
        assert!(elapsed <= Duration::from_millis(100));
        assert_eq!(backoff.retry_count(), 1);
    }
    
    #[test]
    fn test_stats() {
        let mut backoff = ExponentialBackoff::new(100, 5000, 3, false);
        backoff.current_retries = 1;
        
        let stats = backoff.stats();
        assert_eq!(stats.current_retries, 1);
        assert_eq!(stats.max_retries, 3);
        assert_eq!(stats.base_delay_ms, 100);
        assert_eq!(stats.max_delay_ms, 5000);
        assert_eq!(stats.next_delay_ms, 200);
        assert!(stats.can_retry);
    }
}
