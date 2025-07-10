//! OPERATION CONTROL MODULE
//! 
//! Implementation of the 5 Commandments (Nakazów) for micro-lightning operations
//! Enforces operational discipline and risk management

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, Duration};
use tracing::{debug, info, warn, error};

/// Operation control errors
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum OperationError {
    #[error("Hold time violation: minimum 55 minutes between operations")]
    HoldTimeViolation,
    
    #[error("Wallet rotation required: maximum 3 operations per wallet")]
    WalletRotationRequired,
    
    #[error("Cool down period active: 3 consecutive losses require 30-minute break")]
    CoolDownPeriod,
    
    #[error("Daily operation limit exceeded: maximum {limit} operations per day")]
    DailyLimitExceeded { limit: u8 },
    
    #[error("Psychology fund insufficient: minimum balance required")]
    PsychologyFundInsufficient,
    
    #[error("Battlefield validation failed: {reason}")]
    BattlefieldValidationFailed { reason: String },
    
    #[error("Circuit breaker active: trading suspended")]
    CircuitBreakerActive,
}

/// The 5 Commandments configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandmentConfig {
    // Commandment 1: Life Limit
    pub min_hold_time_minutes: u16,
    pub max_hold_time_minutes: u16,
    
    // Commandment 2: Wallet Reincarnation
    pub max_operations_per_wallet: u8,
    pub wallet_rotation_cooldown_minutes: u16,
    
    // Commandment 3: Militia Strategy
    pub max_consecutive_losses: u8,
    pub cooldown_after_losses_minutes: u16,
    
    // Commandment 4: Emotional Accounting
    pub psychology_tax_rate: f64,
    pub min_psychology_fund_balance: f64,
    
    // Commandment 5: Battlefield Selection
    pub min_liquidity: f64,
    pub max_liquidity: f64,
    pub min_holder_count: usize,
    pub max_holder_count: usize,
}

impl Default for CommandmentConfig {
    fn default() -> Self {
        Self {
            // Commandment 1: Life Limit
            min_hold_time_minutes: 55,
            max_hold_time_minutes: 60,
            
            // Commandment 2: Wallet Reincarnation
            max_operations_per_wallet: 3,
            wallet_rotation_cooldown_minutes: 30,
            
            // Commandment 3: Militia Strategy
            max_consecutive_losses: 3,
            cooldown_after_losses_minutes: 30,
            
            // Commandment 4: Emotional Accounting
            psychology_tax_rate: 0.10, // 10% of profits
            min_psychology_fund_balance: 2.0, // $2 minimum
            
            // Commandment 5: Battlefield Selection
            min_liquidity: 2000.0,
            max_liquidity: 10000.0,
            min_holder_count: 50,
            max_holder_count: 500,
        }
    }
}

/// Operation control state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationControl {
    config: CommandmentConfig,
    
    // Commandment 1: Life Limit tracking
    last_trade_time: Option<SystemTime>,
    current_position_start: Option<SystemTime>,
    
    // Commandment 2: Wallet Reincarnation tracking
    wallet_counter: u8,
    operations_this_wallet: u8,
    last_wallet_rotation: Option<SystemTime>,
    
    // Commandment 3: Militia Strategy tracking
    loss_streak: u8,
    cooldown_end_time: Option<SystemTime>,
    
    // Commandment 4: Emotional Accounting tracking
    psychology_fund_balance: f64,
    total_profits_taxed: f64,
    
    // Commandment 5: Battlefield Selection tracking
    validated_tokens: Vec<String>,
    rejected_tokens: Vec<String>,
    
    // General operation tracking
    daily_operations: u8,
    last_operation_date: Option<SystemTime>,
    total_operations: u32,
    successful_operations: u32,
}

impl OperationControl {
    /// Create new operation control with default configuration
    pub fn new() -> Self {
        Self {
            config: CommandmentConfig::default(),
            last_trade_time: None,
            current_position_start: None,
            wallet_counter: 1,
            operations_this_wallet: 0,
            last_wallet_rotation: None,
            loss_streak: 0,
            cooldown_end_time: None,
            psychology_fund_balance: 4.0, // Initial psychology fund
            total_profits_taxed: 0.0,
            validated_tokens: Vec::new(),
            rejected_tokens: Vec::new(),
            daily_operations: 0,
            last_operation_date: None,
            total_operations: 0,
            successful_operations: 0,
        }
    }

    /// Create operation control with custom configuration
    pub fn with_config(config: CommandmentConfig) -> Self {
        let mut control = Self::new();
        control.config = config;
        control.psychology_fund_balance = config.min_psychology_fund_balance;
        control
    }

    /// Check all operational conditions (The 5 Commandments)
    pub fn check_conditions(&self) -> Result<(), OperationError> {
        debug!("🔍 Checking operational conditions (5 Commandments)");

        // Commandment 1: Life Limit
        self.enforce_life_limit()?;
        
        // Commandment 2: Wallet Reincarnation
        self.enforce_wallet_reincarnation()?;
        
        // Commandment 3: Militia Strategy
        self.enforce_militia_strategy()?;
        
        // Commandment 4: Emotional Accounting
        self.enforce_emotional_accounting()?;
        
        // Commandment 5: Battlefield Selection (checked per token)
        // This is validated separately in validate_battlefield()

        info!("✅ All operational conditions satisfied");
        Ok(())
    }

    /// Commandment 1: Enforce life limit (55-minute minimum between operations)
    fn enforce_life_limit(&self) -> Result<(), OperationError> {
        if let Some(last_trade) = self.last_trade_time {
            let elapsed = SystemTime::now()
                .duration_since(last_trade)
                .unwrap_or(Duration::ZERO);
            
            let min_duration = Duration::from_secs(self.config.min_hold_time_minutes as u64 * 60);
            
            if elapsed < min_duration {
                let remaining = min_duration - elapsed;
                warn!("⏰ Life limit violation: {} minutes remaining", remaining.as_secs() / 60);
                return Err(OperationError::HoldTimeViolation);
            }
        }
        
        debug!("✅ Commandment 1 (Life Limit) satisfied");
        Ok(())
    }

    /// Commandment 2: Enforce wallet reincarnation (max 3 operations per wallet)
    fn enforce_wallet_reincarnation(&self) -> Result<(), OperationError> {
        if self.operations_this_wallet >= self.config.max_operations_per_wallet {
            warn!("🔄 Wallet rotation required: {} operations completed", self.operations_this_wallet);
            return Err(OperationError::WalletRotationRequired);
        }
        
        debug!("✅ Commandment 2 (Wallet Reincarnation) satisfied");
        Ok(())
    }

    /// Commandment 3: Enforce militia strategy (cooldown after 3 losses)
    fn enforce_militia_strategy(&self) -> Result<(), OperationError> {
        if let Some(cooldown_end) = self.cooldown_end_time {
            if SystemTime::now() < cooldown_end {
                let remaining = cooldown_end.duration_since(SystemTime::now())
                    .unwrap_or(Duration::ZERO);
                warn!("❄️ Cooldown period active: {} minutes remaining", remaining.as_secs() / 60);
                return Err(OperationError::CoolDownPeriod);
            }
        }
        
        debug!("✅ Commandment 3 (Militia Strategy) satisfied");
        Ok(())
    }

    /// Commandment 4: Enforce emotional accounting (psychology fund balance)
    fn enforce_emotional_accounting(&self) -> Result<(), OperationError> {
        if self.psychology_fund_balance < self.config.min_psychology_fund_balance {
            warn!("🧠 Psychology fund insufficient: ${:.2} < ${:.2}", 
                  self.psychology_fund_balance, self.config.min_psychology_fund_balance);
            return Err(OperationError::PsychologyFundInsufficient);
        }
        
        debug!("✅ Commandment 4 (Emotional Accounting) satisfied");
        Ok(())
    }

    /// Commandment 5: Validate battlefield selection (liquidity and holder criteria)
    pub fn validate_battlefield(&self, liquidity: f64, holder_count: usize) -> Result<(), OperationError> {
        if liquidity < self.config.min_liquidity || liquidity > self.config.max_liquidity {
            let reason = format!(
                "Liquidity ${:.2} not in range ${:.2}-${:.2}",
                liquidity, self.config.min_liquidity, self.config.max_liquidity
            );
            return Err(OperationError::BattlefieldValidationFailed { reason });
        }

        if holder_count < self.config.min_holder_count || holder_count > self.config.max_holder_count {
            let reason = format!(
                "Holder count {} not in range {}-{}",
                holder_count, self.config.min_holder_count, self.config.max_holder_count
            );
            return Err(OperationError::BattlefieldValidationFailed { reason });
        }

        debug!("✅ Commandment 5 (Battlefield Selection) satisfied");
        Ok(())
    }

    /// Start new operation
    pub fn start_operation(&mut self) -> Result<(), OperationError> {
        self.check_conditions()?;
        
        self.current_position_start = Some(SystemTime::now());
        self.operations_this_wallet += 1;
        self.total_operations += 1;
        
        // Update daily operations
        self.update_daily_operations();
        
        info!("🚀 Operation started: #{} (wallet operation #{})", 
              self.total_operations, self.operations_this_wallet);
        
        Ok(())
    }

    /// Complete operation with result
    pub fn complete_operation(&mut self, profit: f64, success: bool) {
        if let Some(start_time) = self.current_position_start {
            let duration = SystemTime::now().duration_since(start_time).unwrap_or(Duration::ZERO);
            info!("✅ Operation completed: profit ${:.2}, duration {} minutes", 
                  profit, duration.as_secs() / 60);
        }

        // Update success tracking
        if success {
            self.successful_operations += 1;
            self.loss_streak = 0; // Reset loss streak on success
            
            // Apply psychology tax on profits
            if profit > 0.0 {
                self.apply_psychology_tax(profit);
            }
        } else {
            self.loss_streak += 1;
            
            // Activate cooldown if needed
            if self.loss_streak >= self.config.max_consecutive_losses {
                self.activate_cooldown();
            }
        }

        // Update last trade time
        self.last_trade_time = Some(SystemTime::now());
        self.current_position_start = None;
    }

    /// Apply psychology tax (Commandment 4)
    fn apply_psychology_tax(&mut self, profit: f64) {
        let tax = profit * self.config.psychology_tax_rate;
        self.psychology_fund_balance += tax;
        self.total_profits_taxed += tax;
        
        info!("🧠 Psychology tax applied: ${:.2} ({}% of ${:.2} profit)", 
              tax, self.config.psychology_tax_rate * 100.0, profit);
    }

    /// Activate cooldown period (Commandment 3)
    fn activate_cooldown(&mut self) {
        let cooldown_duration = Duration::from_secs(self.config.cooldown_after_losses_minutes as u64 * 60);
        self.cooldown_end_time = Some(SystemTime::now() + cooldown_duration);
        
        warn!("❄️ Cooldown activated: {} consecutive losses, {} minutes cooldown", 
              self.loss_streak, self.config.cooldown_after_losses_minutes);
    }

    /// Rotate wallet (Commandment 2)
    pub fn rotate_wallet(&mut self) {
        self.wallet_counter += 1;
        self.operations_this_wallet = 0;
        self.last_wallet_rotation = Some(SystemTime::now());
        
        info!("🔄 Wallet rotated: now using wallet #{}", self.wallet_counter);
    }

    /// Update daily operations counter
    fn update_daily_operations(&mut self) {
        let now = SystemTime::now();
        let today_start = now.duration_since(UNIX_EPOCH).unwrap().as_secs() / 86400 * 86400;
        let today_start_time = UNIX_EPOCH + Duration::from_secs(today_start);

        if let Some(last_date) = self.last_operation_date {
            if last_date < today_start_time {
                // New day, reset counter
                self.daily_operations = 1;
            } else {
                self.daily_operations += 1;
            }
        } else {
            self.daily_operations = 1;
        }

        self.last_operation_date = Some(now);
    }

    /// Get remaining operations for current wallet
    pub fn remaining_operations(&self) -> u8 {
        self.config.max_operations_per_wallet - self.operations_this_wallet
    }

    /// Get time until wallet rotation is allowed
    pub fn time_until_rotation(&self) -> Duration {
        if let Some(last_rotation) = self.last_wallet_rotation {
            let cooldown = Duration::from_secs(self.config.wallet_rotation_cooldown_minutes as u64 * 60);
            let elapsed = SystemTime::now().duration_since(last_rotation).unwrap_or(Duration::ZERO);
            
            if elapsed < cooldown {
                return cooldown - elapsed;
            }
        }
        Duration::ZERO
    }

    /// Check if MEV warning should be displayed
    pub fn has_mev_warning(&self) -> bool {
        self.loss_streak >= 2 || self.operations_this_wallet >= 2
    }

    /// Get operation statistics
    pub fn get_statistics(&self) -> OperationStatistics {
        let win_rate = if self.total_operations > 0 {
            self.successful_operations as f64 / self.total_operations as f64
        } else {
            0.0
        };

        OperationStatistics {
            total_operations: self.total_operations,
            successful_operations: self.successful_operations,
            win_rate,
            current_loss_streak: self.loss_streak,
            current_wallet: self.wallet_counter,
            operations_this_wallet: self.operations_this_wallet,
            psychology_fund_balance: self.psychology_fund_balance,
            daily_operations: self.daily_operations,
            is_cooldown_active: self.cooldown_end_time.is_some(),
        }
    }

    /// Increment operation count (for external tracking)
    pub fn increment_operation_count(&mut self) {
        self.total_operations += 1;
        self.operations_this_wallet += 1;
        self.update_daily_operations();
    }
}

use std::time::UNIX_EPOCH;

impl Default for OperationControl {
    fn default() -> Self {
        Self::new()
    }
}

/// Operation statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationStatistics {
    pub total_operations: u32,
    pub successful_operations: u32,
    pub win_rate: f64,
    pub current_loss_streak: u8,
    pub current_wallet: u8,
    pub operations_this_wallet: u8,
    pub psychology_fund_balance: f64,
    pub daily_operations: u8,
    pub is_cooldown_active: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_control_creation() {
        let control = OperationControl::new();
        assert_eq!(control.wallet_counter, 1);
        assert_eq!(control.operations_this_wallet, 0);
        assert_eq!(control.loss_streak, 0);
    }

    #[test]
    fn test_wallet_rotation_requirement() {
        let mut control = OperationControl::new();
        control.operations_this_wallet = 3;
        
        let result = control.check_conditions();
        assert!(matches!(result, Err(OperationError::WalletRotationRequired)));
    }

    #[test]
    fn test_psychology_tax() {
        let mut control = OperationControl::new();
        let initial_balance = control.psychology_fund_balance;
        
        control.apply_psychology_tax(10.0);
        assert_eq!(control.psychology_fund_balance, initial_balance + 1.0); // 10% of $10
    }

    #[test]
    fn test_battlefield_validation() {
        let control = OperationControl::new();
        
        // Valid battlefield
        assert!(control.validate_battlefield(5000.0, 100).is_ok());
        
        // Invalid liquidity
        assert!(control.validate_battlefield(1000.0, 100).is_err());
        
        // Invalid holder count
        assert!(control.validate_battlefield(5000.0, 10).is_err());
    }
}
