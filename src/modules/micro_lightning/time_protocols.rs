//! TIME PROTOCOLS MODULE
//! 
//! Time-based trading rules and limits for micro-lightning operations
//! Implements golden window, decay window, and hard expiry logic

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, Duration};
use tracing::{debug, info, warn};

/// Exit percentage wrapper for type safety
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ExitPercentage(pub f64);

impl ExitPercentage {
    /// Create new exit percentage (0.0 to 1.0)
    pub fn new(percentage: f64) -> Self {
        Self(percentage.clamp(0.0, 1.0))
    }

    /// Get percentage as decimal (0.0 to 1.0)
    pub fn as_decimal(&self) -> f64 {
        self.0
    }

    /// Get percentage as whole number (0 to 100)
    pub fn as_percentage(&self) -> f64 {
        self.0 * 100.0
    }

    /// Check if full exit is required
    pub fn is_full_exit(&self) -> bool {
        self.0 >= 1.0
    }

    /// Check if no exit is required
    pub fn is_no_exit(&self) -> bool {
        self.0 <= 0.0
    }
}

/// Time window types for different trading phases
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TimeWindow {
    Golden {
        start_minutes: u16,
        end_minutes: u16,
    },
    Decay {
        start_minutes: u16,
        end_minutes: u16,
        decay_rate: f64,
    },
    HardExpiry {
        expiry_minutes: u16,
    },
}

/// Time protocol configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeProtocolConfig {
    pub golden_window_end: u16,      // End of golden window (minutes)
    pub decay_window_end: u16,       // End of decay window (minutes)
    pub hard_expiry: u16,            // Hard expiry time (minutes)
    pub decay_interval: u16,         // Decay check interval (minutes)
    pub decay_percentage: f64,       // Percentage to exit per decay interval
    pub emergency_exit_buffer: u16,  // Buffer before hard expiry (minutes)
}

impl Default for TimeProtocolConfig {
    fn default() -> Self {
        Self {
            golden_window_end: 15,      // 15 minutes golden window
            decay_window_end: 45,       // 45 minutes decay window
            hard_expiry: 55,            // 55 minutes hard expiry
            decay_interval: 5,          // Check every 5 minutes
            decay_percentage: 0.33,     // Exit 33% per interval
            emergency_exit_buffer: 5,   // 5 minutes before hard expiry
        }
    }
}

/// Time protocol manager
pub struct TimeProtocol {
    position_start: SystemTime,
    config: TimeProtocolConfig,
    last_decay_check: Option<SystemTime>,
    cumulative_exit_percentage: f64,
}

impl TimeProtocol {
    /// Create new time protocol with current time as start
    pub fn new() -> Self {
        Self {
            position_start: SystemTime::now(),
            config: TimeProtocolConfig::default(),
            last_decay_check: None,
            cumulative_exit_percentage: 0.0,
        }
    }

    /// Create time protocol with custom configuration
    pub fn with_config(config: TimeProtocolConfig) -> Self {
        Self {
            position_start: SystemTime::now(),
            config,
            last_decay_check: None,
            cumulative_exit_percentage: 0.0,
        }
    }

    /// Create time protocol with specific start time
    pub fn with_start_time(start_time: SystemTime) -> Self {
        Self {
            position_start: start_time,
            config: TimeProtocolConfig::default(),
            last_decay_check: None,
            cumulative_exit_percentage: 0.0,
        }
    }

    /// Get elapsed time since position start
    pub fn elapsed_time(&self) -> Duration {
        SystemTime::now()
            .duration_since(self.position_start)
            .unwrap_or(Duration::ZERO)
    }

    /// Get elapsed time in minutes
    pub fn elapsed_minutes(&self) -> f64 {
        self.elapsed_time().as_secs_f64() / 60.0
    }

    /// Get remaining time until hard expiry
    pub fn time_remaining(&self) -> Duration {
        let elapsed = self.elapsed_time();
        let max_duration = Duration::from_secs(self.config.hard_expiry as u64 * 60);
        
        if elapsed >= max_duration {
            Duration::ZERO
        } else {
            max_duration - elapsed
        }
    }

    /// Get remaining time in minutes
    pub fn remaining_minutes(&self) -> f64 {
        self.time_remaining().as_secs_f64() / 60.0
    }

    /// Get current time window
    pub fn get_current_window(&self) -> TimeWindow {
        let elapsed_minutes = self.elapsed_minutes() as u16;

        if elapsed_minutes <= self.config.golden_window_end {
            TimeWindow::Golden {
                start_minutes: 0,
                end_minutes: self.config.golden_window_end,
            }
        } else if elapsed_minutes <= self.config.decay_window_end {
            TimeWindow::Decay {
                start_minutes: self.config.golden_window_end,
                end_minutes: self.config.decay_window_end,
                decay_rate: self.config.decay_percentage,
            }
        } else {
            TimeWindow::HardExpiry {
                expiry_minutes: self.config.hard_expiry,
            }
        }
    }

    /// Calculate exit strategy based on current time
    pub fn exit_strategy(&mut self) -> ExitPercentage {
        let elapsed_minutes = self.elapsed_minutes();
        
        debug!("⏰ Time protocol check: {:.1} minutes elapsed", elapsed_minutes);

        match self.get_current_window() {
            TimeWindow::Golden { .. } => {
                info!("✨ Golden window active - no exit required");
                ExitPercentage::new(0.0)
            },
            
            TimeWindow::Decay { .. } => {
                self.calculate_decay_exit()
            },
            
            TimeWindow::HardExpiry { .. } => {
                warn!("⏰ Hard expiry reached - forcing full exit");
                ExitPercentage::new(1.0)
            },
        }
    }

    /// Calculate decay window exit percentage
    fn calculate_decay_exit(&mut self) -> ExitPercentage {
        let now = SystemTime::now();
        let elapsed_minutes = self.elapsed_minutes();
        
        // Check if we need to perform decay check
        let should_decay = match self.last_decay_check {
            None => true,
            Some(last_check) => {
                let time_since_last = now.duration_since(last_check)
                    .unwrap_or(Duration::ZERO)
                    .as_secs() / 60; // Convert to minutes
                
                time_since_last >= self.config.decay_interval as u64
            }
        };

        if should_decay {
            // Calculate how many decay intervals have passed since golden window
            let decay_start = self.config.golden_window_end as f64;
            let time_in_decay = elapsed_minutes - decay_start;
            let decay_intervals = (time_in_decay / self.config.decay_interval as f64).floor() as u32;
            
            // Calculate total exit percentage based on intervals
            let target_exit = (decay_intervals as f64 * self.config.decay_percentage).min(1.0);
            
            // Only exit additional percentage if we haven't already
            if target_exit > self.cumulative_exit_percentage {
                let additional_exit = target_exit - self.cumulative_exit_percentage;
                self.cumulative_exit_percentage = target_exit;
                self.last_decay_check = Some(now);
                
                info!("📉 Decay window: exiting additional {:.1}% (total: {:.1}%)", 
                      additional_exit * 100.0, target_exit * 100.0);
                
                return ExitPercentage::new(additional_exit);
            }
        }

        ExitPercentage::new(0.0)
    }

    /// Check if emergency exit buffer is reached
    pub fn is_emergency_buffer_reached(&self) -> bool {
        let remaining = self.remaining_minutes();
        remaining <= self.config.emergency_exit_buffer as f64
    }

    /// Check if position should be force-closed
    pub fn should_force_close(&self) -> bool {
        self.time_remaining() == Duration::ZERO
    }

    /// Get time until next decay check
    pub fn time_until_next_decay(&self) -> Option<Duration> {
        if let TimeWindow::Decay { .. } = self.get_current_window() {
            if let Some(last_check) = self.last_decay_check {
                let next_check = last_check + Duration::from_secs(self.config.decay_interval as u64 * 60);
                let now = SystemTime::now();
                
                if next_check > now {
                    return Some(next_check.duration_since(now).unwrap_or(Duration::ZERO));
                }
            } else {
                // First decay check should happen immediately
                return Some(Duration::ZERO);
            }
        }
        None
    }

    /// Get position timing summary
    pub fn get_timing_summary(&self) -> TimingSummary {
        let elapsed = self.elapsed_minutes();
        let remaining = self.remaining_minutes();
        let window = self.get_current_window();
        
        TimingSummary {
            elapsed_minutes: elapsed,
            remaining_minutes: remaining,
            current_window: window,
            cumulative_exit_percentage: self.cumulative_exit_percentage,
            is_emergency_buffer: self.is_emergency_buffer_reached(),
            should_force_close: self.should_force_close(),
        }
    }

    /// Reset protocol for new position
    pub fn reset(&mut self) {
        self.position_start = SystemTime::now();
        self.last_decay_check = None;
        self.cumulative_exit_percentage = 0.0;
        info!("🔄 Time protocol reset for new position");
    }

    /// Update configuration
    pub fn update_config(&mut self, config: TimeProtocolConfig) {
        self.config = config;
        info!("⚙️ Time protocol configuration updated");
    }
}

impl Default for TimeProtocol {
    fn default() -> Self {
        Self::new()
    }
}

/// Timing summary for monitoring and reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingSummary {
    pub elapsed_minutes: f64,
    pub remaining_minutes: f64,
    pub current_window: TimeWindow,
    pub cumulative_exit_percentage: f64,
    pub is_emergency_buffer: bool,
    pub should_force_close: bool,
}

/// Time-based exit recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeBasedExitRecommendation {
    pub exit_percentage: ExitPercentage,
    pub reason: String,
    pub urgency: ExitUrgency,
    pub time_remaining: Duration,
}

/// Exit urgency levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExitUrgency {
    None,       // Golden window
    Low,        // Early decay
    Medium,     // Mid decay
    High,       // Late decay
    Critical,   // Emergency buffer
    Immediate,  // Hard expiry
}

/// Get time-based exit recommendation
pub fn get_time_based_recommendation(protocol: &mut TimeProtocol) -> TimeBasedExitRecommendation {
    let exit_percentage = protocol.exit_strategy();
    let timing = protocol.get_timing_summary();
    
    let (reason, urgency) = match timing.current_window {
        TimeWindow::Golden { .. } => {
            ("Golden window active - hold position".to_string(), ExitUrgency::None)
        },
        TimeWindow::Decay { .. } => {
            if timing.is_emergency_buffer {
                ("Emergency buffer reached - prepare for exit".to_string(), ExitUrgency::Critical)
            } else if timing.remaining_minutes < 15.0 {
                ("Approaching expiry - increase exit rate".to_string(), ExitUrgency::High)
            } else if timing.remaining_minutes < 25.0 {
                ("Mid decay window - gradual exit".to_string(), ExitUrgency::Medium)
            } else {
                ("Early decay window - minimal exit".to_string(), ExitUrgency::Low)
            }
        },
        TimeWindow::HardExpiry { .. } => {
            ("Hard expiry reached - immediate full exit".to_string(), ExitUrgency::Immediate)
        },
    };

    TimeBasedExitRecommendation {
        exit_percentage,
        reason,
        urgency,
        time_remaining: protocol.time_remaining(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_protocol_creation() {
        let protocol = TimeProtocol::new();
        assert!(protocol.elapsed_minutes() < 1.0); // Should be very recent
        assert_eq!(protocol.cumulative_exit_percentage, 0.0);
    }

    #[test]
    fn test_exit_percentage() {
        let exit = ExitPercentage::new(0.5);
        assert_eq!(exit.as_decimal(), 0.5);
        assert_eq!(exit.as_percentage(), 50.0);
        assert!(!exit.is_full_exit());
        assert!(!exit.is_no_exit());
    }

    #[test]
    fn test_golden_window() {
        let protocol = TimeProtocol::new();
        match protocol.get_current_window() {
            TimeWindow::Golden { start_minutes, end_minutes } => {
                assert_eq!(start_minutes, 0);
                assert_eq!(end_minutes, 15);
            },
            _ => panic!("Expected golden window for new protocol"),
        }
    }

    #[test]
    fn test_time_remaining() {
        let protocol = TimeProtocol::new();
        let remaining = protocol.remaining_minutes();
        assert!(remaining > 54.0 && remaining <= 55.0); // Should be close to 55 minutes
    }
}
