//! METRICS MODULE
//! 
//! Performance tracking and reporting for micro-lightning operations
//! Implements comprehensive statistics and monitoring capabilities

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, Duration};
use tracing::{debug, info};

/// Micro trading statistics (based on 500 operation simulation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroTradingStats {
    pub total_operations: u32,
    pub successful_operations: u32,
    pub failed_operations: u32,
    pub avg_hold_time_minutes: f64,
    pub avg_profit_percentage: f64,
    pub avg_loss_percentage: f64,
    pub win_rate: f64,
    pub max_drawdown: f64,
    pub survival_rate: f64,
    pub total_profit: f64,
    pub total_loss: f64,
    pub net_profit: f64,
    pub sharpe_ratio: f64,
    pub max_consecutive_wins: u32,
    pub max_consecutive_losses: u32,
    pub psychology_fund_total: f64,
}

impl Default for MicroTradingStats {
    fn default() -> Self {
        // Based on simulation data from the original implementation
        Self {
            total_operations: 500,
            successful_operations: 290,
            failed_operations: 210,
            avg_hold_time_minutes: 17.0,
            avg_profit_percentage: 2.85,
            avg_loss_percentage: -1.45,
            win_rate: 0.58,
            max_drawdown: 6.80,
            survival_rate: 0.92,
            total_profit: 826.5,
            total_loss: -304.5,
            net_profit: 522.0,
            sharpe_ratio: 1.34,
            max_consecutive_wins: 7,
            max_consecutive_losses: 4,
            psychology_fund_total: 82.65, // 10% of profits
        }
    }
}

impl MicroTradingStats {
    /// Create new empty statistics
    pub fn new() -> Self {
        Self {
            total_operations: 0,
            successful_operations: 0,
            failed_operations: 0,
            avg_hold_time_minutes: 0.0,
            avg_profit_percentage: 0.0,
            avg_loss_percentage: 0.0,
            win_rate: 0.0,
            max_drawdown: 0.0,
            survival_rate: 1.0,
            total_profit: 0.0,
            total_loss: 0.0,
            net_profit: 0.0,
            sharpe_ratio: 0.0,
            max_consecutive_wins: 0,
            max_consecutive_losses: 0,
            psychology_fund_total: 0.0,
        }
    }

    /// Update statistics with new operation result
    pub fn update(&mut self, profit_loss: f64, hold_time_minutes: f64, success: bool) {
        self.total_operations += 1;
        
        if success {
            self.successful_operations += 1;
            self.total_profit += profit_loss;
        } else {
            self.failed_operations += 1;
            self.total_loss += profit_loss.abs();
        }

        // Update averages
        self.win_rate = self.successful_operations as f64 / self.total_operations as f64;
        
        // Update average hold time
        let total_time = self.avg_hold_time_minutes * (self.total_operations - 1) as f64;
        self.avg_hold_time_minutes = (total_time + hold_time_minutes) / self.total_operations as f64;

        // Update profit/loss averages
        if success && self.successful_operations > 0 {
            let total_profit_pct = self.avg_profit_percentage * (self.successful_operations - 1) as f64;
            self.avg_profit_percentage = (total_profit_pct + profit_loss) / self.successful_operations as f64;
        } else if !success && self.failed_operations > 0 {
            let total_loss_pct = self.avg_loss_percentage * (self.failed_operations - 1) as f64;
            self.avg_loss_percentage = (total_loss_pct + profit_loss) / self.failed_operations as f64;
        }

        // Update net profit
        self.net_profit = self.total_profit - self.total_loss;

        // Update psychology fund (10% of profits)
        if success && profit_loss > 0.0 {
            self.psychology_fund_total += profit_loss * 0.1;
        }

        debug!("📊 Statistics updated: {} operations, {:.1}% win rate, ${:.2} net profit",
               self.total_operations, self.win_rate * 100.0, self.net_profit);
    }

    /// Calculate Sharpe ratio
    pub fn calculate_sharpe_ratio(&mut self, returns: &[f64]) -> f64 {
        if returns.len() < 2 {
            return 0.0;
        }

        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns.iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>() / returns.len() as f64;
        let std_dev = variance.sqrt();

        if std_dev == 0.0 {
            0.0
        } else {
            mean_return / std_dev
        }
    }

    /// Get performance summary
    pub fn get_summary(&self) -> PerformanceSummary {
        PerformanceSummary {
            total_operations: self.total_operations,
            win_rate: self.win_rate,
            avg_profit: self.avg_profit_percentage,
            net_profit: self.net_profit,
            max_drawdown: self.max_drawdown,
            sharpe_ratio: self.sharpe_ratio,
            avg_hold_time: self.avg_hold_time_minutes,
            survival_rate: self.survival_rate,
        }
    }
}

/// System status report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusReport {
    pub module_active: bool,
    pub remaining_ops: u8,
    pub wallet_rotation: Duration,
    pub mev_warning: bool,
    pub message: String,
}

impl StatusReport {
    /// Create status report
    pub fn new(active: bool, remaining: u8, rotation_time: Duration, warning: bool) -> Self {
        let message = if active {
            "🟢 MODUŁ MIKRO-BŁYSKAWICA - AKTYWNY".to_string()
        } else {
            "🔴 MODUŁ MIKRO-BŁYSKAWICA - NIEAKTYWNY".to_string()
        };

        Self {
            module_active: active,
            remaining_ops: remaining,
            wallet_rotation: rotation_time,
            mev_warning: warning,
            message,
        }
    }

    /// Get status as formatted string
    pub fn format_status(&self) -> String {
        format!(
            "{}\nOperacje pozostałe: {}\nRotacja portfela: {} min\nOstrzeżenie MEV: {}",
            self.message,
            self.remaining_ops,
            self.wallet_rotation.as_secs() / 60,
            if self.mev_warning { "TAK" } else { "NIE" }
        )
    }
}

/// Performance summary for quick overview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub total_operations: u32,
    pub win_rate: f64,
    pub avg_profit: f64,
    pub net_profit: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    pub avg_hold_time: f64,
    pub survival_rate: f64,
}

/// Real-time metrics collector
pub struct MetricsCollector {
    stats: MicroTradingStats,
    operation_history: VecDeque<OperationRecord>,
    returns_history: VecDeque<f64>,
    drawdown_tracker: DrawdownTracker,
    performance_windows: HashMap<String, PerformanceWindow>,
}

/// Individual operation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationRecord {
    pub operation_id: u32,
    pub timestamp: SystemTime,
    pub token_symbol: String,
    pub entry_price: f64,
    pub exit_price: f64,
    pub profit_loss: f64,
    pub profit_percentage: f64,
    pub hold_time_minutes: f64,
    pub success: bool,
    pub exit_reason: String,
}

/// Drawdown tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawdownTracker {
    peak_value: f64,
    current_value: f64,
    max_drawdown: f64,
    current_drawdown: f64,
    drawdown_start: Option<SystemTime>,
    drawdown_duration: Duration,
}

impl DrawdownTracker {
    pub fn new(initial_value: f64) -> Self {
        Self {
            peak_value: initial_value,
            current_value: initial_value,
            max_drawdown: 0.0,
            current_drawdown: 0.0,
            drawdown_start: None,
            drawdown_duration: Duration::ZERO,
        }
    }

    pub fn update(&mut self, new_value: f64) {
        self.current_value = new_value;

        if new_value > self.peak_value {
            self.peak_value = new_value;
            self.current_drawdown = 0.0;
            self.drawdown_start = None;
        } else {
            self.current_drawdown = (self.peak_value - new_value) / self.peak_value;
            
            if self.current_drawdown > self.max_drawdown {
                self.max_drawdown = self.current_drawdown;
            }

            if self.drawdown_start.is_none() {
                self.drawdown_start = Some(SystemTime::now());
            }
        }
    }
}

/// Performance window for time-based analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceWindow {
    pub window_name: String,
    pub duration: Duration,
    pub operations: VecDeque<OperationRecord>,
    pub total_profit: f64,
    pub win_rate: f64,
    pub avg_hold_time: f64,
}

impl MetricsCollector {
    /// Create new metrics collector
    pub fn new() -> Self {
        let mut performance_windows = HashMap::new();
        
        // Add different time windows for analysis
        performance_windows.insert(
            "1h".to_string(),
            PerformanceWindow {
                window_name: "1 Hour".to_string(),
                duration: Duration::from_secs(3600),
                operations: VecDeque::new(),
                total_profit: 0.0,
                win_rate: 0.0,
                avg_hold_time: 0.0,
            }
        );

        performance_windows.insert(
            "24h".to_string(),
            PerformanceWindow {
                window_name: "24 Hours".to_string(),
                duration: Duration::from_secs(86400),
                operations: VecDeque::new(),
                total_profit: 0.0,
                win_rate: 0.0,
                avg_hold_time: 0.0,
            }
        );

        Self {
            stats: MicroTradingStats::new(),
            operation_history: VecDeque::with_capacity(1000),
            returns_history: VecDeque::with_capacity(500),
            drawdown_tracker: DrawdownTracker::new(20.0), // Starting with $20
            performance_windows,
        }
    }

    /// Record new operation
    pub fn record_operation(&mut self, record: OperationRecord) {
        // Update main statistics
        self.stats.update(record.profit_loss, record.hold_time_minutes, record.success);

        // Add to history
        self.operation_history.push_back(record.clone());
        if self.operation_history.len() > 1000 {
            self.operation_history.pop_front();
        }

        // Update returns history
        self.returns_history.push_back(record.profit_percentage);
        if self.returns_history.len() > 500 {
            self.returns_history.pop_front();
        }

        // Update drawdown tracker
        let new_portfolio_value = 20.0 + self.stats.net_profit; // $20 starting capital
        self.drawdown_tracker.update(new_portfolio_value);

        // Update performance windows
        self.update_performance_windows(&record);

        // Recalculate Sharpe ratio
        let returns: Vec<f64> = self.returns_history.iter().cloned().collect();
        self.stats.sharpe_ratio = self.stats.calculate_sharpe_ratio(&returns);

        info!("📊 Operation recorded: {} total, {:.1}% win rate, ${:.2} net",
              self.stats.total_operations, self.stats.win_rate * 100.0, self.stats.net_profit);
    }

    /// Update performance windows
    fn update_performance_windows(&mut self, record: &OperationRecord) {
        let cutoff_time = SystemTime::now();

        for window in self.performance_windows.values_mut() {
            // Add new record
            window.operations.push_back(record.clone());

            // Remove old records outside the window
            let window_start = cutoff_time - window.duration;
            while let Some(front) = window.operations.front() {
                if front.timestamp < window_start {
                    window.operations.pop_front();
                } else {
                    break;
                }
            }

            // Recalculate window statistics
            window.total_profit = window.operations.iter().map(|op| op.profit_loss).sum();
            window.win_rate = if window.operations.is_empty() {
                0.0
            } else {
                window.operations.iter().filter(|op| op.success).count() as f64 / window.operations.len() as f64
            };
            window.avg_hold_time = if window.operations.is_empty() {
                0.0
            } else {
                window.operations.iter().map(|op| op.hold_time_minutes).sum::<f64>() / window.operations.len() as f64
            };
        }
    }

    /// Get current statistics
    pub fn get_stats(&self) -> &MicroTradingStats {
        &self.stats
    }

    /// Get performance window
    pub fn get_performance_window(&self, window_name: &str) -> Option<&PerformanceWindow> {
        self.performance_windows.get(window_name)
    }

    /// Get recent operations
    pub fn get_recent_operations(&self, count: usize) -> Vec<&OperationRecord> {
        self.operation_history.iter().rev().take(count).collect()
    }

    /// Get drawdown information
    pub fn get_drawdown_info(&self) -> &DrawdownTracker {
        &self.drawdown_tracker
    }

    /// Export statistics to JSON
    pub fn export_stats(&self) -> Result<String> {
        serde_json::to_string_pretty(&self.stats)
            .map_err(|e| anyhow::anyhow!("Failed to serialize stats: {}", e))
    }

    /// Generate performance report
    pub fn generate_report(&self) -> PerformanceReport {
        let recent_24h = self.get_performance_window("24h");
        let recent_1h = self.get_performance_window("1h");

        PerformanceReport {
            overall_stats: self.stats.clone(),
            last_24h: recent_24h.cloned(),
            last_1h: recent_1h.cloned(),
            current_drawdown: self.drawdown_tracker.current_drawdown,
            max_drawdown: self.drawdown_tracker.max_drawdown,
            total_operations: self.operation_history.len(),
            generated_at: SystemTime::now(),
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Comprehensive performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub overall_stats: MicroTradingStats,
    pub last_24h: Option<PerformanceWindow>,
    pub last_1h: Option<PerformanceWindow>,
    pub current_drawdown: f64,
    pub max_drawdown: f64,
    pub total_operations: usize,
    pub generated_at: SystemTime,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_micro_trading_stats() {
        let mut stats = MicroTradingStats::new();
        assert_eq!(stats.total_operations, 0);
        assert_eq!(stats.win_rate, 0.0);

        stats.update(5.0, 20.0, true);
        assert_eq!(stats.total_operations, 1);
        assert_eq!(stats.successful_operations, 1);
        assert_eq!(stats.win_rate, 1.0);
    }

    #[test]
    fn test_drawdown_tracker() {
        let mut tracker = DrawdownTracker::new(100.0);
        
        tracker.update(110.0); // New peak
        assert_eq!(tracker.peak_value, 110.0);
        assert_eq!(tracker.current_drawdown, 0.0);

        tracker.update(90.0); // Drawdown
        assert!(tracker.current_drawdown > 0.0);
        assert_eq!(tracker.max_drawdown, tracker.current_drawdown);
    }

    #[test]
    fn test_status_report() {
        let status = StatusReport::new(true, 3, Duration::from_secs(1800), false);
        assert!(status.module_active);
        assert_eq!(status.remaining_ops, 3);
        assert!(!status.mev_warning);
    }
}
