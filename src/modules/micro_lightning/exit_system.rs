//! EXIT SYSTEM MODULE
//! 
//! 3-layer exit strategy implementation for micro-lightning operations
//! Implements take-profit radar, volatility circuit breaker, and sentiment collapse detection

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{SystemTime, Duration};
use tracing::{debug, info, warn};

use super::{TradeContext, SocialMention};

/// Exit command types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExitCommand {
    NoExit,
    PartialExit(f64),  // Percentage to exit (0.0 to 1.0)
    FullExit,
    EmergencyExit,
}

/// Exit trigger reasons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExitReason {
    TakeProfit { level: u8, profit_percentage: f64 },
    VolatilityBreaker { volatility: f64, red_candles: u32 },
    SentimentCollapse { negative_mentions: u32 },
    TimeBasedExit { elapsed_minutes: f64 },
    StopLoss { loss_percentage: f64 },
    ManualExit,
}

/// Take profit radar configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TakeProfitRadar {
    pub levels: Vec<TakeProfitLevel>,
    pub dynamic_adjustment: bool,
    pub volatility_multiplier: f64,
}

/// Take profit level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TakeProfitLevel {
    pub profit_threshold: f64,    // Profit percentage threshold
    pub exit_percentage: f64,     // Percentage of position to exit
    pub triggered: bool,          // Whether this level has been triggered
}

impl Default for TakeProfitRadar {
    fn default() -> Self {
        Self {
            levels: vec![
                TakeProfitLevel {
                    profit_threshold: 0.15,  // 15% profit
                    exit_percentage: 0.25,   // Exit 25%
                    triggered: false,
                },
                TakeProfitLevel {
                    profit_threshold: 0.35,  // 35% profit
                    exit_percentage: 0.40,   // Exit 40%
                    triggered: false,
                },
                TakeProfitLevel {
                    profit_threshold: 0.60,  // 60% profit
                    exit_percentage: 0.50,   // Exit 50%
                    triggered: false,
                },
                TakeProfitLevel {
                    profit_threshold: 1.00,  // 100% profit
                    exit_percentage: 0.75,   // Exit 75%
                    triggered: false,
                },
            ],
            dynamic_adjustment: true,
            volatility_multiplier: 1.2,
        }
    }
}

/// Volatility circuit breaker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatilityCircuit {
    pub volatility_threshold: f64,
    pub red_candle_threshold: u32,
    pub time_window_minutes: u16,
    pub price_drop_threshold: f64,
    pub volume_spike_threshold: f64,
}

impl Default for VolatilityCircuit {
    fn default() -> Self {
        Self {
            volatility_threshold: 0.25,    // 25% volatility
            red_candle_threshold: 3,       // 3 consecutive red candles
            time_window_minutes: 15,       // 15-minute window
            price_drop_threshold: 0.20,    // 20% price drop
            volume_spike_threshold: 3.0,   // 3x volume spike
        }
    }
}

/// Sentiment collapse detector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentDetector {
    pub negative_threshold: f64,
    pub mention_count_threshold: u32,
    pub sentiment_window_minutes: u16,
    pub panic_keywords: Vec<String>,
}

impl Default for SentimentDetector {
    fn default() -> Self {
        Self {
            negative_threshold: -0.7,      // -0.7 sentiment score
            mention_count_threshold: 15,   // 15 negative mentions
            sentiment_window_minutes: 10,  // 10-minute window
            panic_keywords: vec![
                "rug".to_string(),
                "scam".to_string(),
                "dump".to_string(),
                "honeypot".to_string(),
                "exit".to_string(),
                "sell".to_string(),
                "crash".to_string(),
            ],
        }
    }
}

/// 3-layer exit system
pub struct ExitSystem {
    take_profit_radar: TakeProfitRadar,
    volatility_breaker: VolatilityCircuit,
    sentiment_detector: SentimentDetector,
    price_history: VecDeque<PricePoint>,
    last_exit_check: Option<SystemTime>,
}

/// Price point for history tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricePoint {
    pub price: f64,
    pub timestamp: SystemTime,
    pub volume: f64,
}

impl ExitSystem {
    /// Create new exit system with default configuration
    pub fn new() -> Self {
        Self {
            take_profit_radar: TakeProfitRadar::default(),
            volatility_breaker: VolatilityCircuit::default(),
            sentiment_detector: SentimentDetector::default(),
            price_history: VecDeque::with_capacity(100),
            last_exit_check: None,
        }
    }

    /// Create exit system with custom configuration
    pub fn with_config(
        take_profit: TakeProfitRadar,
        volatility: VolatilityCircuit,
        sentiment: SentimentDetector,
    ) -> Self {
        Self {
            take_profit_radar: take_profit,
            volatility_breaker: volatility,
            sentiment_detector: sentiment,
            price_history: VecDeque::with_capacity(100),
            last_exit_check: None,
        }
    }

    /// Main exit decision logic
    pub fn should_exit(&mut self, context: &TradeContext) -> Option<ExitCommand> {
        debug!("🔍 Checking exit conditions");

        // Update price history
        self.update_price_history(context);

        // Layer 1: Take-Profit Radar
        if let Some(exit_command) = self.check_take_profit(context) {
            return Some(exit_command);
        }

        // Layer 2: Volatility Circuit Breaker
        if let Some(exit_command) = self.check_volatility_breaker(context) {
            return Some(exit_command);
        }

        // Layer 3: Sentiment Collapse Detector
        if let Some(exit_command) = self.check_sentiment_collapse(context) {
            return Some(exit_command);
        }

        None
    }

    /// Layer 1: Check take-profit conditions
    fn check_take_profit(&mut self, context: &TradeContext) -> Option<ExitCommand> {
        for level in &mut self.take_profit_radar.levels {
            if !level.triggered && context.profit >= level.profit_threshold {
                level.triggered = true;
                
                info!("🎯 Take-profit level triggered: {:.1}% profit, exiting {:.1}%",
                      level.profit_threshold * 100.0, level.exit_percentage * 100.0);
                
                return Some(ExitCommand::PartialExit(level.exit_percentage));
            }
        }
        None
    }

    /// Layer 2: Check volatility circuit breaker
    fn check_volatility_breaker(&self, context: &TradeContext) -> Option<ExitCommand> {
        // Check volatility threshold
        if context.volatility_5min > self.volatility_breaker.volatility_threshold {
            warn!("⚡ High volatility detected: {:.2}% > {:.2}%",
                  context.volatility_5min * 100.0, 
                  self.volatility_breaker.volatility_threshold * 100.0);
            
            // Check red candle count
            if context.red_candle_count >= self.volatility_breaker.red_candle_threshold {
                warn!("🔴 Volatility circuit breaker triggered: {} red candles",
                      context.red_candle_count);
                return Some(ExitCommand::FullExit);
            }
        }

        // Check for massive price drop
        if context.profit < -self.volatility_breaker.price_drop_threshold {
            warn!("📉 Massive price drop detected: {:.1}% loss",
                  context.profit.abs() * 100.0);
            return Some(ExitCommand::EmergencyExit);
        }

        None
    }

    /// Layer 3: Check sentiment collapse
    fn check_sentiment_collapse(&self, context: &TradeContext) -> Option<ExitCommand> {
        let recent_mentions = self.get_recent_mentions(&context.social_mentions);
        let negative_count = recent_mentions.iter()
            .filter(|m| m.sentiment_score < self.sentiment_detector.negative_threshold)
            .count() as u32;

        if negative_count >= self.sentiment_detector.mention_count_threshold {
            warn!("😰 Sentiment collapse detected: {} negative mentions",
                  negative_count);
            return Some(ExitCommand::FullExit);
        }

        // Check for panic keywords
        let panic_mentions = recent_mentions.iter()
            .filter(|m| self.contains_panic_keywords(m))
            .count();

        if panic_mentions >= 5 {
            warn!("😱 Panic keywords detected in {} mentions", panic_mentions);
            return Some(ExitCommand::FullExit);
        }

        None
    }

    /// Update price history for analysis
    fn update_price_history(&mut self, context: &TradeContext) {
        let price_point = PricePoint {
            price: context.position.entry_price * (1.0 + context.profit),
            timestamp: SystemTime::now(),
            volume: 0.0, // Would be populated with actual volume data
        };

        self.price_history.push_back(price_point);

        // Keep only recent history (last 100 points)
        if self.price_history.len() > 100 {
            self.price_history.pop_front();
        }
    }

    /// Get recent social mentions within time window
    fn get_recent_mentions<'a>(&self, mentions: &'a [SocialMention]) -> Vec<&'a SocialMention> {
        let cutoff_time = SystemTime::now() - Duration::from_secs(
            self.sentiment_detector.sentiment_window_minutes as u64 * 60
        );

        mentions.iter()
            .filter(|m| m.timestamp > cutoff_time)
            .collect()
    }

    /// Check if mention contains panic keywords
    fn contains_panic_keywords(&self, mention: &SocialMention) -> bool {
        // In real implementation, this would check the actual mention text
        // For now, we'll use sentiment score as a proxy
        mention.sentiment_score < -0.8
    }

    /// Calculate current volatility from price history
    pub fn calculate_current_volatility(&self) -> f64 {
        if self.price_history.len() < 2 {
            return 0.0;
        }

        let prices: Vec<f64> = self.price_history.iter().map(|p| p.price).collect();
        let returns: Vec<f64> = prices.windows(2)
            .map(|w| (w[1] - w[0]) / w[0])
            .collect();

        if returns.is_empty() {
            return 0.0;
        }

        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns.iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>() / returns.len() as f64;

        variance.sqrt()
    }

    /// Reset take-profit levels for new position
    pub fn reset_take_profit_levels(&mut self) {
        for level in &mut self.take_profit_radar.levels {
            level.triggered = false;
        }
        info!("🔄 Take-profit levels reset");
    }

    /// Get exit system status
    pub fn get_status(&self) -> ExitSystemStatus {
        let triggered_levels = self.take_profit_radar.levels.iter()
            .filter(|l| l.triggered)
            .count();

        ExitSystemStatus {
            take_profit_levels_triggered: triggered_levels,
            total_take_profit_levels: self.take_profit_radar.levels.len(),
            current_volatility: self.calculate_current_volatility(),
            price_history_length: self.price_history.len(),
            last_check: self.last_exit_check,
        }
    }

    /// Update exit system configuration
    pub fn update_take_profit_config(&mut self, config: TakeProfitRadar) {
        self.take_profit_radar = config;
        info!("⚙️ Take-profit configuration updated");
    }

    /// Update volatility circuit configuration
    pub fn update_volatility_config(&mut self, config: VolatilityCircuit) {
        self.volatility_breaker = config;
        info!("⚙️ Volatility circuit configuration updated");
    }

    /// Update sentiment detector configuration
    pub fn update_sentiment_config(&mut self, config: SentimentDetector) {
        self.sentiment_detector = config;
        info!("⚙️ Sentiment detector configuration updated");
    }
}

impl Default for ExitSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Exit system status for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitSystemStatus {
    pub take_profit_levels_triggered: usize,
    pub total_take_profit_levels: usize,
    pub current_volatility: f64,
    pub price_history_length: usize,
    pub last_check: Option<SystemTime>,
}

/// Exit recommendation with detailed analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitRecommendation {
    pub command: ExitCommand,
    pub reason: ExitReason,
    pub confidence: f64,
    pub urgency: ExitUrgency,
    pub analysis: ExitAnalysis,
}

/// Exit urgency levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExitUrgency {
    None,
    Low,
    Medium,
    High,
    Critical,
}

/// Detailed exit analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitAnalysis {
    pub profit_status: String,
    pub volatility_status: String,
    pub sentiment_status: String,
    pub risk_score: f64,
    pub recommendation_summary: String,
}

/// Get comprehensive exit recommendation
pub fn get_exit_recommendation(exit_system: &mut ExitSystem, context: &TradeContext) -> ExitRecommendation {
    let command = exit_system.should_exit(context).unwrap_or(ExitCommand::NoExit);
    
    let (reason, confidence, urgency) = match &command {
        ExitCommand::NoExit => {
            (ExitReason::ManualExit, 0.0, ExitUrgency::None)
        },
        ExitCommand::PartialExit(percentage) => {
            (ExitReason::TakeProfit { level: 1, profit_percentage: context.profit }, 0.8, ExitUrgency::Medium)
        },
        ExitCommand::FullExit => {
            if context.volatility_5min > 0.25 {
                (ExitReason::VolatilityBreaker { volatility: context.volatility_5min, red_candles: context.red_candle_count }, 0.9, ExitUrgency::High)
            } else {
                (ExitReason::SentimentCollapse { negative_mentions: context.social_mentions.len() as u32 }, 0.85, ExitUrgency::High)
            }
        },
        ExitCommand::EmergencyExit => {
            (ExitReason::StopLoss { loss_percentage: context.profit.abs() }, 1.0, ExitUrgency::Critical)
        },
    };

    let analysis = ExitAnalysis {
        profit_status: format!("Current P&L: {:.2}%", context.profit * 100.0),
        volatility_status: format!("Volatility: {:.2}%", context.volatility_5min * 100.0),
        sentiment_status: format!("Social mentions: {}", context.social_mentions.len()),
        risk_score: calculate_risk_score(context),
        recommendation_summary: format!("{:?} - Confidence: {:.1}%", command, confidence * 100.0),
    };

    ExitRecommendation {
        command,
        reason,
        confidence,
        urgency,
        analysis,
    }
}

/// Calculate overall risk score
fn calculate_risk_score(context: &TradeContext) -> f64 {
    let mut risk: f64 = 0.0;

    // Profit/loss risk
    if context.profit < -0.1 {
        risk += 0.3;
    } else if context.profit < 0.0 {
        risk += 0.1;
    }

    // Volatility risk
    if context.volatility_5min > 0.3 {
        risk += 0.4;
    } else if context.volatility_5min > 0.2 {
        risk += 0.2;
    }

    // Red candle risk
    if context.red_candle_count >= 3 {
        risk += 0.3;
    } else if context.red_candle_count >= 2 {
        risk += 0.1;
    }

    risk.min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_system_creation() {
        let exit_system = ExitSystem::new();
        assert_eq!(exit_system.take_profit_radar.levels.len(), 4);
        assert!(!exit_system.take_profit_radar.levels[0].triggered);
    }

    #[test]
    fn test_take_profit_trigger() {
        let mut exit_system = ExitSystem::new();
        let mut context = TradeContext::default();
        context.profit = 0.20; // 20% profit

        let result = exit_system.should_exit(&context);
        assert!(matches!(result, Some(ExitCommand::PartialExit(_))));
    }

    #[test]
    fn test_volatility_breaker() {
        let mut exit_system = ExitSystem::new();
        let mut context = TradeContext::default();
        context.volatility_5min = 0.30; // 30% volatility
        context.red_candle_count = 3;

        let result = exit_system.should_exit(&context);
        assert!(matches!(result, Some(ExitCommand::FullExit)));
    }
}
