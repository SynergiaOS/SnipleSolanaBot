//! EMERGENCY PROTOCOLS MODULE
//! 
//! Panic exit and emergency safety systems for micro-lightning operations
//! Implements rapid response to critical market conditions

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, Duration};
use tracing::{error, warn, info};

use super::micro_wallet::WalletType;
use super::{Position, TradeContext};

/// Emergency trigger types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EmergencyTrigger {
    CreatorSellDetected {
        wallet_address: String,
        sell_amount: f64,
        percentage_of_supply: f64,
    },
    LiquidityDrop {
        previous_liquidity: f64,
        current_liquidity: f64,
        drop_percentage: f64,
    },
    TimeExceeded {
        max_time_minutes: u16,
        actual_time_minutes: u16,
    },
    MassiveDump {
        price_drop_percentage: f64,
        volume_spike: f64,
    },
    HoneypotDetected {
        detection_method: String,
        confidence: f64,
    },
    NetworkCongestion {
        gas_price_spike: f64,
        transaction_failures: u32,
    },
    RiskLimitBreached {
        risk_type: String,
        current_value: f64,
        limit_value: f64,
    },
}

/// Emergency action types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    CancelAllOrders,
    MarketSell {
        token: String,
        amount: f64,
        slippage: f64,
    },
    Transfer {
        destination: WalletType,
        amount: f64,
    },
    FlagToken {
        token: String,
        reason: FlagReason,
    },
    NotifyOperator {
        message: String,
        severity: AlertSeverity,
    },
    ActivateCircuitBreaker {
        duration_minutes: u16,
    },
    EmergencyWithdraw {
        protocol: String,
        amount: f64,
    },
}

/// Token flagging reasons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FlagReason {
    Scam,
    Honeypot,
    RugPull,
    LiquidityIssues,
    CreatorDump,
    NetworkIssues,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Emergency exit execution plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyExit {
    pub trigger: EmergencyTrigger,
    pub actions: Vec<Action>,
    pub execution_order: Vec<usize>, // Order of action execution
    pub max_execution_time_seconds: u64,
    pub fallback_actions: Vec<Action>,
}

/// Emergency protocol configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyConfig {
    pub creator_sell_threshold: f64,        // % of supply that triggers emergency
    pub liquidity_drop_threshold: f64,      // % drop that triggers emergency
    pub max_hold_time_minutes: u16,         // Maximum hold time before force exit
    pub price_drop_threshold: f64,          // % price drop that triggers emergency
    pub honeypot_confidence_threshold: f64, // Confidence level for honeypot detection
    pub gas_spike_threshold: f64,           // Gas price spike multiplier
    pub max_transaction_failures: u32,      // Max failed transactions before emergency
    pub emergency_slippage: f64,            // High slippage for emergency exits
}

impl Default for EmergencyConfig {
    fn default() -> Self {
        Self {
            creator_sell_threshold: 0.05,      // 5% of supply
            liquidity_drop_threshold: 0.30,    // 30% liquidity drop
            max_hold_time_minutes: 55,         // 55 minutes max hold
            price_drop_threshold: 0.40,        // 40% price drop
            honeypot_confidence_threshold: 0.8, // 80% confidence
            gas_spike_threshold: 3.0,          // 3x gas price spike
            max_transaction_failures: 3,       // 3 failed transactions
            emergency_slippage: 45.0,          // 45% slippage for emergency
        }
    }
}

/// Emergency protocol manager
pub struct EmergencyProtocol {
    config: EmergencyConfig,
    active_emergencies: Vec<EmergencyExit>,
    circuit_breaker_active: bool,
    circuit_breaker_end_time: Option<SystemTime>,
}

impl EmergencyProtocol {
    /// Create new emergency protocol manager
    pub fn new() -> Self {
        Self {
            config: EmergencyConfig::default(),
            active_emergencies: Vec::new(),
            circuit_breaker_active: false,
            circuit_breaker_end_time: None,
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: EmergencyConfig) -> Self {
        Self {
            config,
            active_emergencies: Vec::new(),
            circuit_breaker_active: false,
            circuit_breaker_end_time: None,
        }
    }

    /// Check for emergency triggers
    pub fn check_triggers(&self, context: &TradeContext) -> Option<EmergencyTrigger> {
        // Check time limit
        if let Some(trigger) = self.check_time_limit(context) {
            return Some(trigger);
        }

        // Check liquidity drop
        if let Some(trigger) = self.check_liquidity_drop(context) {
            return Some(trigger);
        }

        // Check massive dump
        if let Some(trigger) = self.check_massive_dump(context) {
            return Some(trigger);
        }

        // Check creator sell
        if let Some(trigger) = self.check_creator_sell(context) {
            return Some(trigger);
        }

        None
    }

    /// Check time limit trigger
    fn check_time_limit(&self, context: &TradeContext) -> Option<EmergencyTrigger> {
        // This would be implemented with actual position timing
        // For now, return None as placeholder
        None
    }

    /// Check liquidity drop trigger
    fn check_liquidity_drop(&self, context: &TradeContext) -> Option<EmergencyTrigger> {
        // This would check actual liquidity data
        // For now, return None as placeholder
        None
    }

    /// Check massive dump trigger
    fn check_massive_dump(&self, context: &TradeContext) -> Option<EmergencyTrigger> {
        if context.profit < -self.config.price_drop_threshold {
            return Some(EmergencyTrigger::MassiveDump {
                price_drop_percentage: context.profit.abs(),
                volume_spike: context.volatility_5min,
            });
        }
        None
    }

    /// Check creator sell trigger
    fn check_creator_sell(&self, context: &TradeContext) -> Option<EmergencyTrigger> {
        // This would check on-chain data for creator sells
        // For now, return None as placeholder
        None
    }

    /// Check if circuit breaker is active
    pub fn is_circuit_breaker_active(&self) -> bool {
        if let Some(end_time) = self.circuit_breaker_end_time {
            if SystemTime::now() > end_time {
                return false;
            }
        }
        self.circuit_breaker_active
    }

    /// Activate circuit breaker
    pub fn activate_circuit_breaker(&mut self, duration_minutes: u16) {
        self.circuit_breaker_active = true;
        self.circuit_breaker_end_time = Some(
            SystemTime::now() + Duration::from_secs(duration_minutes as u64 * 60)
        );
        
        warn!("🚨 Circuit breaker activated for {} minutes", duration_minutes);
    }

    /// Deactivate circuit breaker
    pub fn deactivate_circuit_breaker(&mut self) {
        self.circuit_breaker_active = false;
        self.circuit_breaker_end_time = None;
        info!("✅ Circuit breaker deactivated");
    }
}

impl Default for EmergencyProtocol {
    fn default() -> Self {
        Self::new()
    }
}

/// Main panic exit function
pub fn panic_exit(trigger: EmergencyTrigger, position: &Position) -> EmergencyExit {
    warn!("🚨 PANIC EXIT TRIGGERED: {:?}", trigger);

    let mut actions = Vec::new();
    let mut execution_order = Vec::new();
    let mut action_index = 0;

    // Step 1: Cancel all pending orders
    actions.push(Action::CancelAllOrders);
    execution_order.push(action_index);
    action_index += 1;

    // Step 2: Market sell with high slippage
    let emergency_slippage = match &trigger {
        EmergencyTrigger::CreatorSellDetected { .. } => 50.0,
        EmergencyTrigger::LiquidityDrop { drop_percentage, .. } => {
            if *drop_percentage > 0.5 { 60.0 } else { 45.0 }
        },
        EmergencyTrigger::TimeExceeded { .. } => 35.0,
        EmergencyTrigger::MassiveDump { price_drop_percentage, .. } => {
            if *price_drop_percentage > 0.5 { 55.0 } else { 45.0 }
        },
        EmergencyTrigger::HoneypotDetected { .. } => 70.0, // Highest slippage for honeypots
        EmergencyTrigger::NetworkCongestion { .. } => 40.0,
        EmergencyTrigger::RiskLimitBreached { .. } => 45.0,
    };

    actions.push(Action::MarketSell {
        token: position.token.clone(),
        amount: position.amount,
        slippage: emergency_slippage,
    });
    execution_order.push(action_index);
    action_index += 1;

    // Step 3: Transfer to tactical exit wallet
    actions.push(Action::Transfer {
        destination: WalletType::TacticalExit,
        amount: position.current_value,
    });
    execution_order.push(action_index);
    action_index += 1;

    // Step 4: Flag token based on trigger
    let flag_reason = match &trigger {
        EmergencyTrigger::CreatorSellDetected { .. } => FlagReason::CreatorDump,
        EmergencyTrigger::LiquidityDrop { .. } => FlagReason::LiquidityIssues,
        EmergencyTrigger::TimeExceeded { .. } => FlagReason::LiquidityIssues,
        EmergencyTrigger::MassiveDump { .. } => FlagReason::RugPull,
        EmergencyTrigger::HoneypotDetected { .. } => FlagReason::Honeypot,
        EmergencyTrigger::NetworkCongestion { .. } => FlagReason::NetworkIssues,
        EmergencyTrigger::RiskLimitBreached { .. } => FlagReason::Scam,
    };

    actions.push(Action::FlagToken {
        token: position.token.clone(),
        reason: flag_reason,
    });
    execution_order.push(action_index);
    action_index += 1;

    // Step 5: Notify operator
    let severity = match &trigger {
        EmergencyTrigger::CreatorSellDetected { .. } => AlertSeverity::High,
        EmergencyTrigger::LiquidityDrop { .. } => AlertSeverity::High,
        EmergencyTrigger::TimeExceeded { .. } => AlertSeverity::Medium,
        EmergencyTrigger::MassiveDump { .. } => AlertSeverity::Critical,
        EmergencyTrigger::HoneypotDetected { .. } => AlertSeverity::Critical,
        EmergencyTrigger::NetworkCongestion { .. } => AlertSeverity::Medium,
        EmergencyTrigger::RiskLimitBreached { .. } => AlertSeverity::High,
    };

    actions.push(Action::NotifyOperator {
        message: format!("Emergency exit executed for {}: {:?}", position.token, trigger),
        severity,
    });
    execution_order.push(action_index);
    action_index += 1;

    // Step 6: Activate circuit breaker for severe cases
    match &trigger {
        EmergencyTrigger::HoneypotDetected { .. } => {
            actions.push(Action::ActivateCircuitBreaker {
                duration_minutes: 30,
            });
            execution_order.push(action_index);
            action_index += 1;
        },
        EmergencyTrigger::MassiveDump { price_drop_percentage, .. } if price_drop_percentage > &0.6 => {
            actions.push(Action::ActivateCircuitBreaker {
                duration_minutes: 30,
            });
            execution_order.push(action_index);
        },
        _ => {}
    }

    // Fallback actions if primary actions fail
    let fallback_actions = vec![
        Action::EmergencyWithdraw {
            protocol: "Raydium".to_string(),
            amount: position.amount,
        },
        Action::NotifyOperator {
            message: format!("CRITICAL: Primary emergency exit failed for {}", position.token),
            severity: AlertSeverity::Critical,
        },
    ];

    EmergencyExit {
        trigger,
        actions,
        execution_order,
        max_execution_time_seconds: 30, // 30 seconds max execution time
        fallback_actions,
    }
}

/// Execute emergency exit plan
pub async fn execute_emergency_exit(emergency_exit: &EmergencyExit) -> Result<()> {
    error!("🚨 EXECUTING EMERGENCY EXIT: {:?}", emergency_exit.trigger);

    let start_time = SystemTime::now();
    let max_duration = Duration::from_secs(emergency_exit.max_execution_time_seconds);

    // Execute actions in order
    for &action_index in &emergency_exit.execution_order {
        if let Some(action) = emergency_exit.actions.get(action_index) {
            if let Err(e) = execute_action(action).await {
                error!("❌ Emergency action failed: {:?}, error: {}", action, e);
                
                // If we're running out of time, execute fallback actions
                if start_time.elapsed().unwrap_or(Duration::ZERO) > max_duration {
                    warn!("⏰ Emergency exit timeout, executing fallback actions");
                    for fallback_action in &emergency_exit.fallback_actions {
                        if let Err(e) = execute_action(fallback_action).await {
                            error!("❌ Fallback action failed: {:?}, error: {}", fallback_action, e);
                        }
                    }
                    break;
                }
            }
        }
    }

    info!("✅ Emergency exit execution completed");
    Ok(())
}

/// Execute individual emergency action
async fn execute_action(action: &Action) -> Result<()> {
    match action {
        Action::CancelAllOrders => {
            info!("🚫 Cancelling all orders");
            // Implementation would cancel all pending orders
            Ok(())
        },
        Action::MarketSell { token, amount, slippage } => {
            warn!("📤 Emergency market sell: {} amount: {:.2} slippage: {:.1}%", 
                  token, amount, slippage);
            // Implementation would execute market sell
            Ok(())
        },
        Action::Transfer { destination, amount } => {
            info!("💸 Emergency transfer: {:.2} to {:?}", amount, destination);
            // Implementation would transfer funds
            Ok(())
        },
        Action::FlagToken { token, reason } => {
            warn!("🚩 Flagging token {}: {:?}", token, reason);
            // Implementation would flag token in database
            Ok(())
        },
        Action::NotifyOperator { message, severity } => {
            match severity {
                AlertSeverity::Critical => error!("🚨 CRITICAL: {}", message),
                AlertSeverity::High => warn!("⚠️ HIGH: {}", message),
                AlertSeverity::Medium => warn!("⚠️ MEDIUM: {}", message),
                AlertSeverity::Low => info!("ℹ️ LOW: {}", message),
            }
            // Implementation would send notification
            Ok(())
        },
        Action::ActivateCircuitBreaker { duration_minutes } => {
            error!("🔴 Activating circuit breaker for {} minutes", duration_minutes);
            // Implementation would activate circuit breaker
            Ok(())
        },
        Action::EmergencyWithdraw { protocol, amount } => {
            error!("🆘 Emergency withdraw from {}: {:.2}", protocol, amount);
            // Implementation would withdraw from protocol
            Ok(())
        },
    }
}
