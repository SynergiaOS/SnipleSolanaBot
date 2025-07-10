//! ENTRY CONDITIONS MODULE
//! 
//! Rapid token evaluation and filtering for micro-lightning operations
//! Implements strict criteria for 15-minute token age window

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};

/// Token data structure for evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenData {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub age_minutes: u8,
    pub liquidity: f64,
    pub holders: usize,
    pub creator_txn_count: u32,
    pub is_honeypot: bool,
    pub entry_price: f64,
    pub market_cap: f64,
    pub volume_24h: f64,
    pub price_change_5m: f64,
    pub price_change_15m: f64,
    pub social_score: f64,
    pub risk_score: f64,
    pub created_at: SystemTime,
}

impl TokenData {
    /// Create new token data
    pub fn new(address: String, symbol: String, name: String) -> Self {
        Self {
            address,
            symbol,
            name,
            age_minutes: 0,
            liquidity: 0.0,
            holders: 0,
            creator_txn_count: 0,
            is_honeypot: false,
            entry_price: 0.0,
            market_cap: 0.0,
            volume_24h: 0.0,
            price_change_5m: 0.0,
            price_change_15m: 0.0,
            social_score: 0.0,
            risk_score: 0.0,
            created_at: SystemTime::now(),
        }
    }

    /// Update token age based on creation time
    pub fn update_age(&mut self) {
        if let Ok(duration) = SystemTime::now().duration_since(self.created_at) {
            self.age_minutes = (duration.as_secs() / 60) as u8;
        }
    }

    /// Calculate risk score based on multiple factors
    pub fn calculate_risk_score(&mut self) {
        let mut risk: f64 = 0.0;

        // Age risk (higher risk for very new tokens)
        if self.age_minutes < 5 {
            risk += 0.3;
        } else if self.age_minutes < 10 {
            risk += 0.2;
        } else if self.age_minutes < 15 {
            risk += 0.1;
        }

        // Liquidity risk
        if self.liquidity < 1000.0 {
            risk += 0.4;
        } else if self.liquidity < 2500.0 {
            risk += 0.2;
        }

        // Holder concentration risk
        if self.holders < 25 {
            risk += 0.3;
        } else if self.holders < 50 {
            risk += 0.1;
        }

        // Creator activity risk
        if self.creator_txn_count > 1 {
            risk += 0.2;
        }

        // Honeypot risk
        if self.is_honeypot {
            risk += 1.0; // Maximum risk
        }

        // Social sentiment risk
        if self.social_score < 0.3 {
            risk += 0.2;
        }

        self.risk_score = risk.min(1.0); // Cap at 1.0
    }

    /// Check if token is in battlefield range (liquidity)
    pub fn is_in_battlefield_range(&self) -> bool {
        (2000.0..=10000.0).contains(&self.liquidity)
    }

    /// Get token quality score
    pub fn get_quality_score(&self) -> f64 {
        let mut score = 0.0;

        // Liquidity score
        if self.liquidity >= 5000.0 {
            score += 0.3;
        } else if self.liquidity >= 2500.0 {
            score += 0.2;
        }

        // Holder distribution score
        if self.holders >= 100 {
            score += 0.2;
        } else if self.holders >= 50 {
            score += 0.1;
        }

        // Volume score
        if self.volume_24h >= 10000.0 {
            score += 0.2;
        } else if self.volume_24h >= 5000.0 {
            score += 0.1;
        }

        // Social score
        score += self.social_score * 0.3;

        score.min(1.0)
    }
}

/// Entry conditions configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryConditions {
    pub max_age_minutes: u8,
    pub min_liquidity: f64,
    pub max_liquidity: f64,
    pub min_holders: usize,
    pub max_holders: usize,
    pub min_mentions: u32,
    pub max_risk_score: f64,
    pub min_quality_score: f64,
    pub require_single_creator_txn: bool,
    pub honeypot_check_enabled: bool,
    pub social_sentiment_threshold: f64,
}

impl Default for EntryConditions {
    fn default() -> Self {
        Self {
            max_age_minutes: 15,        // Maximum 15 minutes old
            min_liquidity: 2500.0,      // Minimum $2,500 liquidity
            max_liquidity: 10000.0,     // Maximum $10,000 liquidity (battlefield range)
            min_holders: 50,            // Minimum 50 holders
            max_holders: 500,           // Maximum 500 holders (avoid over-distributed)
            min_mentions: 30,           // Minimum 30 social mentions
            max_risk_score: 0.6,        // Maximum risk score
            min_quality_score: 0.4,     // Minimum quality score
            require_single_creator_txn: true, // Creator should have only 1 transaction
            honeypot_check_enabled: true,     // Enable honeypot detection
            social_sentiment_threshold: 0.3,  // Minimum social sentiment
        }
    }
}

impl EntryConditions {
    /// Create custom entry conditions
    pub fn new() -> Self {
        Self::default()
    }

    /// Create strict entry conditions for high-risk tolerance
    pub fn strict() -> Self {
        Self {
            max_age_minutes: 10,
            min_liquidity: 5000.0,
            max_liquidity: 8000.0,
            min_holders: 75,
            max_holders: 300,
            min_mentions: 50,
            max_risk_score: 0.4,
            min_quality_score: 0.6,
            require_single_creator_txn: true,
            honeypot_check_enabled: true,
            social_sentiment_threshold: 0.5,
        }
    }

    /// Create relaxed entry conditions for lower-risk tolerance
    pub fn relaxed() -> Self {
        Self {
            max_age_minutes: 20,
            min_liquidity: 1500.0,
            max_liquidity: 15000.0,
            min_holders: 30,
            max_holders: 1000,
            min_mentions: 20,
            max_risk_score: 0.8,
            min_quality_score: 0.2,
            require_single_creator_txn: false,
            honeypot_check_enabled: true,
            social_sentiment_threshold: 0.2,
        }
    }
}

/// Check if token meets entry conditions
pub fn check_entry(token: &TokenData, mentions: u32) -> bool {
    let conditions = EntryConditions::default();
    check_entry_with_conditions(token, mentions, &conditions)
}

/// Check entry conditions with custom configuration
pub fn check_entry_with_conditions(token: &TokenData, mentions: u32, conditions: &EntryConditions) -> bool {
    debug!("🔍 Evaluating token: {} ({})", token.symbol, token.address);

    // Age check
    if token.age_minutes > conditions.max_age_minutes {
        debug!("❌ Token too old: {} minutes > {}", token.age_minutes, conditions.max_age_minutes);
        return false;
    }

    // Liquidity range check
    if token.liquidity < conditions.min_liquidity || token.liquidity > conditions.max_liquidity {
        debug!("❌ Liquidity out of range: ${:.2} not in ${:.2}-${:.2}", 
               token.liquidity, conditions.min_liquidity, conditions.max_liquidity);
        return false;
    }

    // Holder count check
    if token.holders < conditions.min_holders || token.holders > conditions.max_holders {
        debug!("❌ Holder count out of range: {} not in {}-{}", 
               token.holders, conditions.min_holders, conditions.max_holders);
        return false;
    }

    // Social mentions check
    if mentions < conditions.min_mentions {
        debug!("❌ Insufficient social mentions: {} < {}", mentions, conditions.min_mentions);
        return false;
    }

    // Honeypot check
    if conditions.honeypot_check_enabled && token.is_honeypot {
        warn!("🍯 Honeypot detected for token: {}", token.address);
        return false;
    }

    // Creator transaction check
    if conditions.require_single_creator_txn && token.creator_txn_count != 1 {
        debug!("❌ Creator has multiple transactions: {}", token.creator_txn_count);
        return false;
    }

    // Risk score check
    if token.risk_score > conditions.max_risk_score {
        debug!("❌ Risk score too high: {:.2} > {:.2}", token.risk_score, conditions.max_risk_score);
        return false;
    }

    // Quality score check
    let quality_score = token.get_quality_score();
    if quality_score < conditions.min_quality_score {
        debug!("❌ Quality score too low: {:.2} < {:.2}", quality_score, conditions.min_quality_score);
        return false;
    }

    // Social sentiment check
    if token.social_score < conditions.social_sentiment_threshold {
        debug!("❌ Social sentiment too low: {:.2} < {:.2}", 
               token.social_score, conditions.social_sentiment_threshold);
        return false;
    }

    info!("✅ Token passed all entry conditions: {} (Risk: {:.2}, Quality: {:.2})", 
          token.symbol, token.risk_score, quality_score);
    
    true
}

/// Validate token data completeness
pub fn validate_token_data(token: &TokenData) -> Result<()> {
    if token.address.is_empty() {
        return Err(anyhow::anyhow!("Token address is empty"));
    }

    if token.symbol.is_empty() {
        return Err(anyhow::anyhow!("Token symbol is empty"));
    }

    if token.liquidity < 0.0 {
        return Err(anyhow::anyhow!("Invalid liquidity value: {}", token.liquidity));
    }

    if token.entry_price <= 0.0 {
        return Err(anyhow::anyhow!("Invalid entry price: {}", token.entry_price));
    }

    Ok(())
}

/// Get entry recommendation based on token analysis
pub fn get_entry_recommendation(token: &TokenData, mentions: u32) -> EntryRecommendation {
    let conditions_strict = EntryConditions::strict();
    let conditions_default = EntryConditions::default();
    let conditions_relaxed = EntryConditions::relaxed();

    if check_entry_with_conditions(token, mentions, &conditions_strict) {
        EntryRecommendation::StrongBuy
    } else if check_entry_with_conditions(token, mentions, &conditions_default) {
        EntryRecommendation::Buy
    } else if check_entry_with_conditions(token, mentions, &conditions_relaxed) {
        EntryRecommendation::WeakBuy
    } else {
        EntryRecommendation::Avoid
    }
}

/// Entry recommendation levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EntryRecommendation {
    StrongBuy,  // Meets strict conditions
    Buy,        // Meets default conditions
    WeakBuy,    // Meets relaxed conditions
    Avoid,      // Does not meet any conditions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_conditions_default() {
        let mut token = TokenData::new(
            "test_address".to_string(),
            "TEST".to_string(),
            "Test Token".to_string(),
        );
        
        token.age_minutes = 10;
        token.liquidity = 5000.0;
        token.holders = 100;
        token.creator_txn_count = 1;
        token.is_honeypot = false;
        token.social_score = 0.5;
        token.calculate_risk_score();

        assert!(check_entry(&token, 50));
    }

    #[test]
    fn test_entry_conditions_fail_age() {
        let mut token = TokenData::new(
            "test_address".to_string(),
            "TEST".to_string(),
            "Test Token".to_string(),
        );
        
        token.age_minutes = 20; // Too old
        token.liquidity = 5000.0;
        token.holders = 100;
        token.creator_txn_count = 1;
        token.is_honeypot = false;
        token.social_score = 0.5;

        assert!(!check_entry(&token, 50));
    }

    #[test]
    fn test_honeypot_detection() {
        let mut token = TokenData::new(
            "test_address".to_string(),
            "TEST".to_string(),
            "Test Token".to_string(),
        );
        
        token.age_minutes = 10;
        token.liquidity = 5000.0;
        token.holders = 100;
        token.creator_txn_count = 1;
        token.is_honeypot = true; // Honeypot
        token.social_score = 0.5;

        assert!(!check_entry(&token, 50));
    }
}
