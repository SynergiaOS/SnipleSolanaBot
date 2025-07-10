// Risk Manager Module
// Evaluates trading signals against risk parameters
// Enhanced with KINETIC SHIELD system for memcoin strategies

use crate::modules::strategy::TradingSignal;
use crate::modules::memcoin_strategies::KineticShieldConfig;
use crate::modules::micro_lightning::{
    OperationControl, OperationError, EmergencyTrigger, TimeProtocol
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskParameters {
    pub max_position_size: f64,
    pub max_daily_loss: f64,
    pub min_confidence_threshold: f64,
    // KINETIC SHIELD parameters
    pub kinetic_shield: KineticShieldConfig,
    // MICRO-LIGHTNING specific parameters
    pub micro_lightning_enabled: bool,
    pub micro_operation_max_loss: f64,
    pub micro_time_limit_minutes: u16,
    pub micro_emergency_slippage: f64,
}

impl Default for RiskParameters {
    fn default() -> Self {
        Self {
            max_position_size: 1000.0,
            max_daily_loss: 100.0,
            min_confidence_threshold: 0.7,
            kinetic_shield: KineticShieldConfig::default(),
            // MICRO-LIGHTNING defaults
            micro_lightning_enabled: true,
            micro_operation_max_loss: 4.0,  // $4 max loss per micro operation
            micro_time_limit_minutes: 55,   // 55-minute time limit
            micro_emergency_slippage: 45.0, // 45% emergency slippage
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovedSignal {
    pub original_signal: TradingSignal,
    pub approved_quantity: f64,
    pub risk_score: f64,
    pub approval_timestamp: chrono::DateTime<chrono::Utc>,
    pub kinetic_shield_status: KineticShieldStatus,
}

/// Micro-lightning risk adjustment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroRiskAdjustment {
    pub action: MicroRiskAction,
    pub adjusted_quantity: f64,
    pub slippage_override: Option<f64>,
    pub reason: String,
}

/// Micro-lightning risk actions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MicroRiskAction {
    Continue,       // Continue normal operation
    ReduceSize,     // Reduce position size
    PrepareExit,    // Prepare for exit
    ForceExit,      // Force immediate exit
    ImmediateExit,  // Emergency immediate exit
    Halt,           // Halt all operations
}

/// Micro-lightning emergency response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroEmergencyResponse {
    pub emergency_type: EmergencyTrigger,
    pub system_state: SystemState,
    pub recommended_action: MicroRiskAction,
    pub circuit_breaker_duration: Option<u16>, // Minutes
    pub emergency_slippage: f64,
}

/// Status KINETIC SHIELD
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KineticShieldStatus {
    Active,
    CircuitBreakerTriggered,
    VolatilityScaled,
    ExposureLimited,
    Bypassed,
}

/// Statystyki strat dla KINETIC SHIELD
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LossStatistics {
    pub daily_drawdown: f32,
    pub hourly_loss_streak: u32,
    pub last_loss_time: chrono::DateTime<chrono::Utc>,
    pub total_losses_today: f64,
    pub consecutive_losses: u32,
}

impl Default for LossStatistics {
    fn default() -> Self {
        Self {
            daily_drawdown: 0.0,
            hourly_loss_streak: 0,
            last_loss_time: chrono::Utc::now(),
            total_losses_today: 0.0,
            consecutive_losses: 0,
        }
    }
}

pub struct RiskManager {
    signal_receiver: mpsc::UnboundedReceiver<TradingSignal>,
    execution_sender: mpsc::UnboundedSender<ApprovedSignal>,
    risk_params: RiskParameters,
    daily_pnl: f64,
    is_running: bool,
    // KINETIC SHIELD components
    loss_stats: Arc<RwLock<LossStatistics>>,
    token_exposures: Arc<RwLock<HashMap<String, f64>>>, // token -> exposure amount
    system_state: Arc<RwLock<SystemState>>,
}

/// Stan systemu KINETIC SHIELD
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemState {
    Normal,
    Lockdown,
    VolatilityProtection,
    ExposureProtection,
}

#[allow(dead_code)]
impl RiskManager {
    pub fn new(
        signal_receiver: mpsc::UnboundedReceiver<TradingSignal>,
        execution_sender: mpsc::UnboundedSender<ApprovedSignal>,
        risk_params: RiskParameters,
    ) -> Self {
        Self {
            signal_receiver,
            execution_sender,
            risk_params,
            daily_pnl: 0.0,
            is_running: false,
            // KINETIC SHIELD initialization
            loss_stats: Arc::new(RwLock::new(LossStatistics::default())),
            token_exposures: Arc::new(RwLock::new(HashMap::new())),
            system_state: Arc::new(RwLock::new(SystemState::Normal)),
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        info!(
            "🛡️ RiskManager starting with params: {:?}",
            self.risk_params
        );
        self.is_running = true;

        while self.is_running {
            if let Some(signal) = self.signal_receiver.recv().await {
                self.evaluate_signal(signal).await?;
            }
        }

        Ok(())
    }

    pub async fn stop(&mut self) {
        info!("🛑 RiskManager stopping...");
        self.is_running = false;
    }

    async fn evaluate_signal(&mut self, signal: TradingSignal) -> Result<()> {
        debug!("Evaluating signal: {}", signal.signal_id);

        // Check confidence threshold
        if signal.confidence < self.risk_params.min_confidence_threshold {
            warn!(
                "Signal {} rejected: confidence {} below threshold {}",
                signal.signal_id, signal.confidence, self.risk_params.min_confidence_threshold
            );
            return Ok(());
        }

        // Check position size limits
        let approved_quantity = self.check_position_limits(&signal)?;
        if approved_quantity <= 0.0 {
            warn!(
                "Signal {} rejected: position size limits exceeded",
                signal.signal_id
            );
            return Ok(());
        }

        // Check daily loss limits
        if !self.check_daily_loss_limits()? {
            warn!(
                "Signal {} rejected: daily loss limits exceeded",
                signal.signal_id
            );
            return Ok(());
        }

        // Calculate risk score
        let risk_score = self.calculate_risk_score(&signal)?;

        // Approve signal
        let approved_signal = ApprovedSignal {
            original_signal: signal.clone(),
            approved_quantity,
            risk_score,
            approval_timestamp: chrono::Utc::now(),
            kinetic_shield_status: KineticShieldStatus::Active, // Domyślnie aktywny
        };

        self.send_approved_signal(approved_signal).await?;
        info!(
            "✅ Signal {} approved with quantity {}",
            signal.signal_id, approved_quantity
        );

        Ok(())
    }

    fn check_position_limits(&self, signal: &TradingSignal) -> Result<f64> {
        if signal.quantity > self.risk_params.max_position_size {
            return Ok(self.risk_params.max_position_size);
        }
        Ok(signal.quantity)
    }

    fn check_daily_loss_limits(&self) -> Result<bool> {
        Ok(self.daily_pnl > -self.risk_params.max_daily_loss)
    }

    fn calculate_risk_score(&self, signal: &TradingSignal) -> Result<f64> {
        let mut risk_score = 0.0;

        // Base risk from confidence (lower confidence = higher risk)
        risk_score += (1.0 - signal.confidence) * 0.4;

        // Position size risk
        let position_ratio = signal.quantity / self.risk_params.max_position_size;
        risk_score += position_ratio * 0.3;

        // Strategy type risk
        risk_score += match signal.strategy_type {
            crate::modules::strategy::StrategyType::TokenSniping => 0.3,
            crate::modules::strategy::StrategyType::Arbitrage => 0.1,
            crate::modules::strategy::StrategyType::MomentumTrading => 0.2,
            crate::modules::strategy::StrategyType::SoulMeteorSniping => 0.25,
            crate::modules::strategy::StrategyType::MeteoraDAMM => 0.8, // Very high risk
            crate::modules::strategy::StrategyType::DeveloperTracking => 0.7, // High risk
            crate::modules::strategy::StrategyType::AxiomMemeCoin => 0.9, // Extreme risk
            crate::modules::strategy::StrategyType::AIDecision => 0.7, // AI decisions have moderate-high risk
            // NEW ADVANCED STRATEGIES
            crate::modules::strategy::StrategyType::MEVArbitrage => 0.4, // Moderate risk
            crate::modules::strategy::StrategyType::CrossDexArbitrage => 0.3, // Lower risk
            crate::modules::strategy::StrategyType::LiquiditySniping => 0.6, // High risk
            crate::modules::strategy::StrategyType::VolumeAnalysis => 0.4, // Moderate risk
            crate::modules::strategy::StrategyType::SocialSentiment => 0.8, // High risk
            crate::modules::strategy::StrategyType::FlashLoanArbitrage => 0.5, // Moderate-high risk
            crate::modules::strategy::StrategyType::YieldFarming => 0.2, // Low risk
            crate::modules::strategy::StrategyType::OptionsStrategy => 0.6, // High risk
            // MEMCOIN SWARMGUARD STRATEGIES
            crate::modules::strategy::StrategyType::LiquidityTsunami => 0.7, // High risk - fast execution
            crate::modules::strategy::StrategyType::SocialFission => 0.8, // Very high risk - social hype
            crate::modules::strategy::StrategyType::WhaleShadowing => 0.6, // High risk - whale following
            crate::modules::strategy::StrategyType::DeathSpiralIntercept => 0.9, // Extreme risk - panic sells
            crate::modules::strategy::StrategyType::MemeVirus => 0.8, // Very high risk - meme cycles
        };

        Ok(risk_score.min(1.0))
    }

    async fn send_approved_signal(&self, signal: ApprovedSignal) -> Result<()> {
        if let Err(e) = self.execution_sender.send(signal) {
            error!("Failed to send approved signal: {}", e);
            return Err(anyhow::anyhow!("Failed to send approved signal"));
        }
        Ok(())
    }

    pub fn update_daily_pnl(&mut self, pnl_change: f64) {
        self.daily_pnl += pnl_change;
    }

    pub fn get_daily_pnl(&self) -> f64 {
        self.daily_pnl
    }

    // ========== KINETIC SHIELD IMPLEMENTATION ==========

    /// Circuit Breaker - automatyczne wyłączenie handlu
    pub async fn check_circuit_breaker(&self) -> Result<bool> {
        let loss_stats = self.loss_stats.read().await;
        let mut system_state = self.system_state.write().await;

        // Sprawdź daily drawdown limit (7.5%)
        if loss_stats.daily_drawdown > self.risk_params.kinetic_shield.daily_drawdown_limit {
            warn!("🚨 KINETIC SHIELD: Circuit breaker triggered - daily drawdown {}% > {}%",
                  loss_stats.daily_drawdown, self.risk_params.kinetic_shield.daily_drawdown_limit);
            *system_state = SystemState::Lockdown;
            return Ok(true);
        }

        // Sprawdź hourly loss streak (5 strat z rzędu)
        if loss_stats.hourly_loss_streak > self.risk_params.kinetic_shield.hourly_loss_streak_limit {
            warn!("🚨 KINETIC SHIELD: Circuit breaker triggered - loss streak {} > {}",
                  loss_stats.hourly_loss_streak, self.risk_params.kinetic_shield.hourly_loss_streak_limit);
            *system_state = SystemState::Lockdown;
            return Ok(true);
        }

        Ok(false)
    }

    /// Volatility Scaling - dynamiczna korekta wielkości pozycji
    pub async fn apply_volatility_scaling(&self, signal: &TradingSignal, volatility: f32) -> f64 {
        let base_size = signal.quantity;

        if volatility > self.risk_params.kinetic_shield.max_volatility {
            let scaling_factor = 1.0 - (volatility / self.risk_params.kinetic_shield.max_volatility).min(0.5);
            let scaled_size = base_size * scaling_factor as f64;

            info!("🛡️ KINETIC SHIELD: Volatility scaling applied - volatility={}, factor={}, size: {} → {}",
                  volatility, scaling_factor, base_size, scaled_size);

            return scaled_size;
        }

        base_size
    }

    /// Cross-Strategy Exposure Limit - maksymalna ekspozycja na token
    pub async fn check_exposure_limit(&self, token: &str, additional_exposure: f64) -> Result<f64> {
        let mut exposures = self.token_exposures.write().await;
        let current_exposure = *exposures.get(token).unwrap_or(&0.0);
        let total_exposure = current_exposure + additional_exposure;

        let capital = 10000.0; // TODO: Get from config
        let max_exposure = capital * self.risk_params.kinetic_shield.exposure_limit_per_token as f64;

        if total_exposure > max_exposure {
            let allowed_exposure = (max_exposure - current_exposure).max(0.0);
            warn!("🛡️ KINETIC SHIELD: Exposure limit reached for {} - requested: {}, allowed: {}",
                  token, additional_exposure, allowed_exposure);

            // Aktualizuj ekspozycję
            exposures.insert(token.to_string(), current_exposure + allowed_exposure);
            return Ok(allowed_exposure);
        }

        // Aktualizuj ekspozycję
        exposures.insert(token.to_string(), total_exposure);
        Ok(additional_exposure)
    }

    /// Aktualizacja statystyk strat
    pub async fn update_loss_statistics(&self, pnl: f64) -> Result<()> {
        let mut loss_stats = self.loss_stats.write().await;
        let now = chrono::Utc::now();

        if pnl < 0.0 {
            // Strata
            loss_stats.total_losses_today += pnl.abs();
            loss_stats.consecutive_losses += 1;
            loss_stats.last_loss_time = now;

            // Sprawdź czy to strata w ostatniej godzinie
            let hour_ago = now - chrono::Duration::hours(1);
            if loss_stats.last_loss_time > hour_ago {
                loss_stats.hourly_loss_streak += 1;
            } else {
                loss_stats.hourly_loss_streak = 1; // Reset streak
            }

            // Oblicz daily drawdown
            let capital = 10000.0; // TODO: Get from config
            loss_stats.daily_drawdown = ((loss_stats.total_losses_today / capital) * 100.0) as f32;

            info!("📉 KINETIC SHIELD: Loss recorded - daily drawdown: {}%, hourly streak: {}",
                  loss_stats.daily_drawdown, loss_stats.hourly_loss_streak);
        } else {
            // Zysk - reset consecutive losses
            loss_stats.consecutive_losses = 0;
        }

        Ok(())
    }

    /// Sprawdzenie stanu systemu KINETIC SHIELD
    pub async fn get_kinetic_shield_status(&self) -> KineticShieldStatus {
        let system_state = self.system_state.read().await;

        match *system_state {
            SystemState::Lockdown => KineticShieldStatus::CircuitBreakerTriggered,
            SystemState::VolatilityProtection => KineticShieldStatus::VolatilityScaled,
            SystemState::ExposureProtection => KineticShieldStatus::ExposureLimited,
            SystemState::Normal => KineticShieldStatus::Active,
        }
    }

    /// Aplikacja KINETIC SHIELD do sygnału
    pub async fn apply_kinetic_shield(&self, signal: TradingSignal) -> Result<Option<ApprovedSignal>> {
        // 1. Circuit Breaker Check
        if self.check_circuit_breaker().await? {
            warn!("🚨 KINETIC SHIELD: Signal blocked by circuit breaker");
            return Ok(None);
        }

        // 2. Exposure Limit Check
        let allowed_quantity = self.check_exposure_limit(&signal.symbol, signal.quantity).await?;
        if allowed_quantity <= 0.0 {
            warn!("🛡️ KINETIC SHIELD: Signal blocked by exposure limit");
            return Ok(None);
        }

        // 3. Volatility Scaling (symulacja volatility)
        let simulated_volatility = 0.3; // TODO: Get real volatility
        let scaled_quantity = self.apply_volatility_scaling(&signal, simulated_volatility).await;
        let final_quantity = allowed_quantity.min(scaled_quantity);

        // 4. Calculate risk score
        let risk_score = self.calculate_risk_score(&signal)?;

        // 5. Get KINETIC SHIELD status
        let shield_status = self.get_kinetic_shield_status().await;

        let approved_signal = ApprovedSignal {
            original_signal: signal,
            approved_quantity: final_quantity,
            risk_score,
            approval_timestamp: chrono::Utc::now(),
            kinetic_shield_status: shield_status,
        };

        info!("✅ KINETIC SHIELD: Signal approved - quantity: {} → {}",
              approved_signal.original_signal.quantity, final_quantity);

        Ok(Some(approved_signal))
    }

    /// MICRO-LIGHTNING specific risk validation
    pub async fn validate_micro_operation(&self, operation_control: &OperationControl) -> Result<bool> {
        if !self.risk_params.micro_lightning_enabled {
            warn!("🚫 Micro-lightning operations disabled in risk parameters");
            return Ok(false);
        }

        // Check operational conditions (5 Commandments)
        if let Err(e) = operation_control.check_conditions() {
            warn!("❌ Micro operation validation failed: {}", e);
            return Ok(false);
        }

        // Check daily loss limits for micro operations
        if self.daily_pnl < -self.risk_params.micro_operation_max_loss {
            warn!("📉 Micro operation blocked: daily loss ${:.2} exceeds limit ${:.2}",
                  self.daily_pnl.abs(), self.risk_params.micro_operation_max_loss);
            return Ok(false);
        }

        info!("✅ Micro operation validation passed");
        Ok(true)
    }

    /// Apply micro-lightning specific risk controls
    pub async fn apply_micro_risk_controls(&self, signal: &TradingSignal, time_protocol: &TimeProtocol) -> Result<Option<MicroRiskAdjustment>> {
        // Check time limits
        if time_protocol.elapsed_minutes() > self.risk_params.micro_time_limit_minutes as f64 {
            warn!("⏰ Micro operation time limit exceeded: {:.1} > {} minutes",
                  time_protocol.elapsed_minutes(), self.risk_params.micro_time_limit_minutes);

            return Ok(Some(MicroRiskAdjustment {
                action: MicroRiskAction::ForceExit,
                adjusted_quantity: signal.quantity,
                slippage_override: Some(self.risk_params.micro_emergency_slippage),
                reason: "Time limit exceeded".to_string(),
            }));
        }

        // Check if emergency buffer is reached
        if time_protocol.is_emergency_buffer_reached() {
            warn!("🚨 Emergency buffer reached, preparing for exit");

            return Ok(Some(MicroRiskAdjustment {
                action: MicroRiskAction::PrepareExit,
                adjusted_quantity: signal.quantity * 0.5, // Reduce position size
                slippage_override: Some(self.risk_params.micro_emergency_slippage * 0.8),
                reason: "Emergency buffer reached".to_string(),
            }));
        }

        Ok(None)
    }

    /// Handle micro-lightning emergency triggers
    pub async fn handle_micro_emergency(&mut self, trigger: &EmergencyTrigger) -> Result<MicroEmergencyResponse> {
        error!("🚨 Handling micro emergency: {:?}", trigger);

        // Update system state based on emergency type
        let mut system_state = self.system_state.write().await;
        *system_state = match trigger {
            EmergencyTrigger::CreatorSellDetected { .. } => SystemState::Lockdown,
            EmergencyTrigger::LiquidityDrop { drop_percentage, .. } => {
                if *drop_percentage > 0.5 {
                    SystemState::Lockdown
                } else {
                    SystemState::VolatilityProtection
                }
            },
            EmergencyTrigger::TimeExceeded { .. } => SystemState::VolatilityProtection,
            EmergencyTrigger::MassiveDump { .. } => SystemState::Lockdown,
            EmergencyTrigger::HoneypotDetected { .. } => SystemState::Lockdown,
            EmergencyTrigger::NetworkCongestion { .. } => SystemState::VolatilityProtection,
            EmergencyTrigger::RiskLimitBreached { .. } => SystemState::ExposureProtection,
        };

        let response = MicroEmergencyResponse {
            emergency_type: trigger.clone(),
            system_state: system_state.clone(),
            recommended_action: match trigger {
                EmergencyTrigger::HoneypotDetected { .. } |
                EmergencyTrigger::MassiveDump { .. } => MicroRiskAction::ImmediateExit,
                _ => MicroRiskAction::ForceExit,
            },
            circuit_breaker_duration: match trigger {
                EmergencyTrigger::HoneypotDetected { .. } => Some(30), // 30 minutes
                EmergencyTrigger::MassiveDump { .. } => Some(15),      // 15 minutes
                _ => Some(5),                                          // 5 minutes
            },
            emergency_slippage: self.risk_params.micro_emergency_slippage,
        };

        warn!("🛡️ Micro emergency response: {:?}", response.recommended_action);
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::modules::strategy::{StrategyType, TradeAction};

    #[tokio::test]
    async fn test_risk_manager_creation() {
        let (_signal_tx, signal_rx) = mpsc::unbounded_channel();
        let (execution_tx, _execution_rx) = mpsc::unbounded_channel();

        let risk_params = RiskParameters {
            max_position_size: 1000.0,
            max_daily_loss: 500.0,
            min_confidence_threshold: 0.6,
            kinetic_shield: KineticShieldConfig::default(),
        };

        let manager = RiskManager::new(signal_rx, execution_tx, risk_params);
        assert!(!manager.is_running);
        assert_eq!(manager.daily_pnl, 0.0);
    }
}
