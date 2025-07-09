//! Fallback logic for CHIMERA Client
//! 
//! Provides static rule-based decision making when AI services are unavailable.
//! This ensures THE OVERMIND PROTOCOL can continue operating even during AI outages.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Trading action types for fallback decisions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TradingAction {
    Buy,
    Sell,
    Hold,
    StopLoss,
    TakeProfit,
}

/// Market condition indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketCondition {
    /// Current price
    pub price: f64,
    
    /// 24h price change percentage
    pub price_change_24h: f64,
    
    /// Trading volume
    pub volume: f64,
    
    /// Market volatility indicator
    pub volatility: f64,
    
    /// RSI indicator (0-100)
    pub rsi: Option<f64>,
    
    /// Moving average indicators
    pub ma_short: Option<f64>,
    pub ma_long: Option<f64>,
}

/// Fallback trading decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackDecision {
    /// Recommended action
    pub action: TradingAction,
    
    /// Confidence level (0.0 - 1.0)
    pub confidence: f64,
    
    /// Reasoning for the decision
    pub reasoning: String,
    
    /// Risk level (0.0 - 1.0)
    pub risk_level: f64,
    
    /// Suggested position size (percentage of portfolio)
    pub position_size: f64,
}

/// Static rule-based trading engine
#[derive(Debug, Clone)]
pub struct FallbackEngine {
    /// Risk tolerance settings
    risk_tolerance: f64,
    
    /// Maximum position size
    max_position_size: f64,
    
    /// Stop loss percentage
    stop_loss_pct: f64,
    
    /// Take profit percentage
    take_profit_pct: f64,
    
    /// RSI overbought threshold
    rsi_overbought: f64,
    
    /// RSI oversold threshold
    rsi_oversold: f64,
    
    /// Volatility threshold for high-risk conditions
    volatility_threshold: f64,
}

impl FallbackEngine {
    /// Create a new fallback engine with default conservative settings
    pub fn new() -> Self {
        Self {
            risk_tolerance: 0.3,        // Conservative
            max_position_size: 0.1,     // 10% max position
            stop_loss_pct: 0.05,        // 5% stop loss
            take_profit_pct: 0.15,      // 15% take profit
            rsi_overbought: 70.0,       // RSI > 70 = overbought
            rsi_oversold: 30.0,         // RSI < 30 = oversold
            volatility_threshold: 0.5,   // High volatility threshold
        }
    }
    
    /// Create aggressive fallback engine for higher risk tolerance
    pub fn aggressive() -> Self {
        Self {
            risk_tolerance: 0.7,
            max_position_size: 0.25,    // 25% max position
            stop_loss_pct: 0.08,        // 8% stop loss
            take_profit_pct: 0.25,      // 25% take profit
            rsi_overbought: 75.0,
            rsi_oversold: 25.0,
            volatility_threshold: 0.7,
        }
    }
    
    /// Create ultra-conservative fallback engine
    pub fn conservative() -> Self {
        Self {
            risk_tolerance: 0.1,
            max_position_size: 0.05,    // 5% max position
            stop_loss_pct: 0.03,        // 3% stop loss
            take_profit_pct: 0.10,      // 10% take profit
            rsi_overbought: 65.0,
            rsi_oversold: 35.0,
            volatility_threshold: 0.3,
        }
    }
    
    /// Make a trading decision based on market conditions
    pub fn make_decision(&self, market: &MarketCondition) -> FallbackDecision {
        info!("Fallback engine making decision for price: ${:.4}", market.price);
        
        // Check for high volatility - be more conservative
        if market.volatility > self.volatility_threshold {
            return self.high_volatility_decision(market);
        }
        
        // RSI-based decisions
        if let Some(rsi) = market.rsi {
            if rsi > self.rsi_overbought {
                return self.overbought_decision(market, rsi);
            } else if rsi < self.rsi_oversold {
                return self.oversold_decision(market, rsi);
            }
        }
        
        // Moving average crossover strategy
        if let (Some(ma_short), Some(ma_long)) = (market.ma_short, market.ma_long) {
            if ma_short > ma_long && market.price > ma_short {
                return self.bullish_decision(market);
            } else if ma_short < ma_long && market.price < ma_short {
                return self.bearish_decision(market);
            }
        }
        
        // Price momentum based decision
        if market.price_change_24h > 5.0 {
            return self.strong_uptrend_decision(market);
        } else if market.price_change_24h < -5.0 {
            return self.strong_downtrend_decision(market);
        }
        
        // Default to hold in uncertain conditions
        self.hold_decision(market)
    }
    
    /// Decision for high volatility conditions
    fn high_volatility_decision(&self, market: &MarketCondition) -> FallbackDecision {
        warn!("High volatility detected: {:.2}%", market.volatility * 100.0);
        
        FallbackDecision {
            action: TradingAction::Hold,
            confidence: 0.8,
            reasoning: format!(
                "High volatility ({:.1}%) detected. Holding position to avoid whipsaws.",
                market.volatility * 100.0
            ),
            risk_level: 0.9,
            position_size: 0.0,
        }
    }
    
    /// Decision for overbought conditions
    fn overbought_decision(&self, market: &MarketCondition, rsi: f64) -> FallbackDecision {
        debug!("Overbought condition: RSI = {:.1}", rsi);
        
        FallbackDecision {
            action: TradingAction::Sell,
            confidence: 0.7,
            reasoning: format!(
                "RSI overbought at {:.1}. Taking profits on potential reversal.",
                rsi
            ),
            risk_level: 0.4,
            position_size: self.max_position_size * 0.5,
        }
    }
    
    /// Decision for oversold conditions
    fn oversold_decision(&self, market: &MarketCondition, rsi: f64) -> FallbackDecision {
        debug!("Oversold condition: RSI = {:.1}", rsi);
        
        FallbackDecision {
            action: TradingAction::Buy,
            confidence: 0.6,
            reasoning: format!(
                "RSI oversold at {:.1}. Potential bounce opportunity.",
                rsi
            ),
            risk_level: 0.5,
            position_size: self.max_position_size * 0.7,
        }
    }
    
    /// Decision for bullish trend
    fn bullish_decision(&self, market: &MarketCondition) -> FallbackDecision {
        debug!("Bullish trend detected");
        
        FallbackDecision {
            action: TradingAction::Buy,
            confidence: 0.65,
            reasoning: "Short MA above long MA with price above short MA. Bullish trend.".to_string(),
            risk_level: 0.6,
            position_size: self.max_position_size * self.risk_tolerance,
        }
    }
    
    /// Decision for bearish trend
    fn bearish_decision(&self, market: &MarketCondition) -> FallbackDecision {
        debug!("Bearish trend detected");
        
        FallbackDecision {
            action: TradingAction::Sell,
            confidence: 0.65,
            reasoning: "Short MA below long MA with price below short MA. Bearish trend.".to_string(),
            risk_level: 0.6,
            position_size: self.max_position_size * self.risk_tolerance,
        }
    }
    
    /// Decision for strong uptrend
    fn strong_uptrend_decision(&self, market: &MarketCondition) -> FallbackDecision {
        debug!("Strong uptrend: +{:.1}% in 24h", market.price_change_24h);
        
        FallbackDecision {
            action: TradingAction::Buy,
            confidence: 0.55,
            reasoning: format!(
                "Strong 24h momentum: +{:.1}%. Following the trend.",
                market.price_change_24h
            ),
            risk_level: 0.7,
            position_size: self.max_position_size * 0.8,
        }
    }
    
    /// Decision for strong downtrend
    fn strong_downtrend_decision(&self, market: &MarketCondition) -> FallbackDecision {
        debug!("Strong downtrend: {:.1}% in 24h", market.price_change_24h);
        
        FallbackDecision {
            action: TradingAction::Sell,
            confidence: 0.55,
            reasoning: format!(
                "Strong 24h decline: {:.1}%. Avoiding further losses.",
                market.price_change_24h
            ),
            risk_level: 0.7,
            position_size: self.max_position_size * 0.6,
        }
    }
    
    /// Default hold decision
    fn hold_decision(&self, market: &MarketCondition) -> FallbackDecision {
        debug!("No clear signal - holding position");
        
        FallbackDecision {
            action: TradingAction::Hold,
            confidence: 0.5,
            reasoning: "No clear technical signals. Maintaining current position.".to_string(),
            risk_level: 0.3,
            position_size: 0.0,
        }
    }
    
    /// Get current engine settings
    pub fn settings(&self) -> FallbackSettings {
        FallbackSettings {
            risk_tolerance: self.risk_tolerance,
            max_position_size: self.max_position_size,
            stop_loss_pct: self.stop_loss_pct,
            take_profit_pct: self.take_profit_pct,
            rsi_overbought: self.rsi_overbought,
            rsi_oversold: self.rsi_oversold,
            volatility_threshold: self.volatility_threshold,
        }
    }
}

/// Fallback engine settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackSettings {
    pub risk_tolerance: f64,
    pub max_position_size: f64,
    pub stop_loss_pct: f64,
    pub take_profit_pct: f64,
    pub rsi_overbought: f64,
    pub rsi_oversold: f64,
    pub volatility_threshold: f64,
}

impl Default for FallbackEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn sample_market() -> MarketCondition {
        MarketCondition {
            price: 100.0,
            price_change_24h: 2.5,
            volume: 1000000.0,
            volatility: 0.3,
            rsi: Some(50.0),
            ma_short: Some(98.0),
            ma_long: Some(95.0),
        }
    }
    
    #[test]
    fn test_fallback_engine_creation() {
        let engine = FallbackEngine::new();
        assert_eq!(engine.risk_tolerance, 0.3);
        assert_eq!(engine.max_position_size, 0.1);
    }
    
    #[test]
    fn test_overbought_decision() {
        let engine = FallbackEngine::new();
        let mut market = sample_market();
        market.rsi = Some(75.0);
        
        let decision = engine.make_decision(&market);
        assert_eq!(decision.action, TradingAction::Sell);
        assert!(decision.confidence > 0.5);
    }
    
    #[test]
    fn test_oversold_decision() {
        let engine = FallbackEngine::new();
        let mut market = sample_market();
        market.rsi = Some(25.0);
        
        let decision = engine.make_decision(&market);
        assert_eq!(decision.action, TradingAction::Buy);
        assert!(decision.confidence > 0.5);
    }
    
    #[test]
    fn test_high_volatility_decision() {
        let engine = FallbackEngine::new();
        let mut market = sample_market();
        market.volatility = 0.8; // High volatility
        
        let decision = engine.make_decision(&market);
        assert_eq!(decision.action, TradingAction::Hold);
        assert!(decision.risk_level > 0.8);
    }
    
    #[test]
    fn test_bullish_trend() {
        let engine = FallbackEngine::new();
        let mut market = sample_market();
        market.ma_short = Some(102.0);
        market.ma_long = Some(98.0);
        market.price = 103.0;
        
        let decision = engine.make_decision(&market);
        assert_eq!(decision.action, TradingAction::Buy);
    }
    
    #[test]
    fn test_strong_uptrend() {
        let engine = FallbackEngine::new();
        let mut market = sample_market();
        market.price_change_24h = 8.0; // Strong uptrend
        market.rsi = Some(50.0); // Neutral RSI
        
        let decision = engine.make_decision(&market);
        assert_eq!(decision.action, TradingAction::Buy);
    }
    
    #[test]
    fn test_conservative_engine() {
        let engine = FallbackEngine::conservative();
        assert!(engine.max_position_size <= 0.05);
        assert!(engine.risk_tolerance <= 0.1);
    }
    
    #[test]
    fn test_aggressive_engine() {
        let engine = FallbackEngine::aggressive();
        assert!(engine.max_position_size >= 0.2);
        assert!(engine.risk_tolerance >= 0.7);
    }
}
