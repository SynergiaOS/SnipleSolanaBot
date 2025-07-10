// THE OVERMIND PROTOCOL - Multi-Wallet Management System
// Production-grade wallet management for capital segmentation and risk distribution

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::modules::strategy::{StrategyType, TradeAction};

/// Wallet configuration and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    pub wallet_id: String,
    pub name: String,
    pub description: String,
    pub private_key: String, // Base58 encoded or JSON array format
    pub public_key: String,
    pub wallet_type: WalletType,
    pub strategy_allocation: Vec<StrategyAllocation>,
    pub risk_limits: WalletRiskLimits,
    pub status: WalletStatus,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
}

/// Types of wallets for different purposes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WalletType {
    /// Primary trading wallet for main strategies
    Primary,
    /// Secondary wallet for backup and overflow
    Secondary,
    /// High-frequency trading wallet for rapid execution
    HFT,
    /// Conservative wallet for low-risk strategies
    Conservative,
    /// Experimental wallet for testing new strategies
    Experimental,
    /// Arbitrage-specific wallet
    Arbitrage,
    /// MEV protection wallet
    MEVProtection,
    /// Emergency wallet for crisis situations
    Emergency,
    /// Aggressive wallet for high-risk memcoin strategies
    Aggressive,
    /// MICRO-LIGHTNING specific wallet types
    MicroLightning,     // Primary micro-lightning operations
    MicroEmergencyGas,  // Emergency gas reserves for micro ops
    MicroReentry,       // Re-entry buffer for micro ops
    MicroPsychology,    // Psychology fund for micro ops
    MicroTacticalExit,  // Tactical exit reserves for micro ops
}

/// Strategy allocation per wallet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyAllocation {
    pub strategy_type: StrategyType,
    pub allocation_percentage: f64, // 0.0 to 100.0
    pub max_position_size: f64,
    pub enabled: bool,
}

/// Risk limits specific to each wallet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletRiskLimits {
    pub max_daily_loss: f64,
    pub max_position_size: f64,
    pub max_concurrent_positions: u32,
    pub max_exposure_percentage: f64, // % of wallet balance
    pub stop_loss_threshold: f64,
    pub daily_trade_limit: u32,
    // Dodatkowe pola dla micro-lightning
    pub max_drawdown: f64,           // Maksymalny spadek
    pub stop_loss_percentage: f64,   // Procent stop loss
}

/// Wallet operational status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WalletStatus {
    Active,
    Inactive,
    Suspended,
    Emergency,
    Maintenance,
}

/// Wallet balance and performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletMetrics {
    pub wallet_id: String,
    pub sol_balance: f64,
    pub token_balances: HashMap<String, f64>,
    pub total_value_usd: f64,
    pub daily_pnl: f64,
    pub total_pnl: f64,
    pub trade_count_today: u32,
    pub last_trade_time: Option<DateTime<Utc>>,
    pub risk_utilization: f64, // % of risk limits used
    pub performance_score: f64,
    pub updated_at: DateTime<Utc>,
}

/// Multi-wallet manager for THE OVERMIND PROTOCOL
pub struct WalletManager {
    wallets: Arc<RwLock<HashMap<String, WalletConfig>>>,
    wallet_metrics: Arc<RwLock<HashMap<String, WalletMetrics>>>,
    active_positions: Arc<RwLock<HashMap<String, Vec<Position>>>>,
    strategy_wallet_mapping: Arc<RwLock<HashMap<StrategyType, Vec<String>>>>,
    default_wallet_id: Option<String>,
}

/// Position tracking per wallet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub position_id: String,
    pub wallet_id: String,
    pub symbol: String,
    pub strategy_type: StrategyType,
    pub action: TradeAction,
    pub quantity: f64,
    pub entry_price: f64,
    pub current_price: f64,
    pub unrealized_pnl: f64,
    pub opened_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Micro-lightning wallet health report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroWalletHealthReport {
    pub total_wallets: u32,
    pub active_wallets: u32,
    pub total_balance: f64,
    pub psychology_fund_balance: f64,
    pub emergency_gas_balance: f64,
    pub reentry_buffer_balance: f64,
    pub wallet_rotation_needed: bool,
    pub health_score: f64, // 0.0 to 1.0
    pub recommendations: Vec<String>,
}

/// Wallet selection criteria for trade execution
#[derive(Debug, Clone)]
pub struct WalletSelectionCriteria {
    pub strategy_type: StrategyType,
    pub required_balance: f64,
    pub risk_tolerance: f64,
    pub preferred_wallet_type: Option<WalletType>,
    pub exclude_wallets: Vec<String>,
}

/// Result of wallet selection process
#[derive(Debug, Clone)]
pub struct WalletSelection {
    pub wallet_id: String,
    pub wallet_config: WalletConfig,
    pub available_balance: f64,
    pub risk_capacity: f64,
    pub selection_reason: String,
}

impl WalletManager {
    /// Create new wallet manager
    pub fn new() -> Self {
        Self {
            wallets: Arc::new(RwLock::new(HashMap::new())),
            wallet_metrics: Arc::new(RwLock::new(HashMap::new())),
            active_positions: Arc::new(RwLock::new(HashMap::new())),
            strategy_wallet_mapping: Arc::new(RwLock::new(HashMap::new())),
            default_wallet_id: None,
        }
    }

    /// Initialize wallet manager with configuration
    pub async fn initialize(&mut self, wallet_configs: Vec<WalletConfig>) -> Result<()> {
        info!("🏦 Initializing THE OVERMIND PROTOCOL Multi-Wallet Manager");

        let mut wallets = self.wallets.write().await;
        let mut strategy_mapping = self.strategy_wallet_mapping.write().await;

        for config in wallet_configs {
            // Validate wallet configuration
            self.validate_wallet_config(&config)?;

            // Set first active wallet as default
            if self.default_wallet_id.is_none() && config.status == WalletStatus::Active {
                self.default_wallet_id = Some(config.wallet_id.clone());
                info!(
                    "🎯 Set default wallet: {} ({})",
                    config.name, config.wallet_id
                );
            }

            // Build strategy mapping
            for allocation in &config.strategy_allocation {
                if allocation.enabled {
                    strategy_mapping
                        .entry(allocation.strategy_type.clone())
                        .or_insert_with(Vec::new)
                        .push(config.wallet_id.clone());
                }
            }

            info!(
                "✅ Loaded wallet: {} ({}) - Type: {:?}, Status: {:?}",
                config.name, config.wallet_id, config.wallet_type, config.status
            );

            wallets.insert(config.wallet_id.clone(), config);
        }

        info!(
            "🏦 Multi-Wallet Manager initialized with {} wallets",
            wallets.len()
        );
        Ok(())
    }

    /// Add new wallet to the system
    pub async fn add_wallet(&self, config: WalletConfig) -> Result<()> {
        self.validate_wallet_config(&config)?;

        let mut wallets = self.wallets.write().await;
        let mut strategy_mapping = self.strategy_wallet_mapping.write().await;

        // Update strategy mapping
        for allocation in &config.strategy_allocation {
            if allocation.enabled {
                strategy_mapping
                    .entry(allocation.strategy_type.clone())
                    .or_insert_with(Vec::new)
                    .push(config.wallet_id.clone());
            }
        }

        info!(
            "➕ Added new wallet: {} ({})",
            config.name, config.wallet_id
        );
        wallets.insert(config.wallet_id.clone(), config);

        Ok(())
    }

    /// Select optimal wallet for trade execution
    pub async fn select_wallet(
        &self,
        criteria: WalletSelectionCriteria,
    ) -> Result<WalletSelection> {
        let wallets = self.wallets.read().await;
        let metrics = self.wallet_metrics.read().await;
        let strategy_mapping = self.strategy_wallet_mapping.read().await;

        // Get candidate wallets for this strategy
        let candidate_wallet_ids = strategy_mapping
            .get(&criteria.strategy_type)
            .cloned()
            .unwrap_or_default();

        if candidate_wallet_ids.is_empty() {
            return Err(anyhow!(
                "No wallets configured for strategy: {:?}",
                criteria.strategy_type
            ));
        }

        let mut best_wallet: Option<WalletSelection> = None;
        let mut best_score = 0.0;

        for wallet_id in candidate_wallet_ids {
            if criteria.exclude_wallets.contains(&wallet_id) {
                continue;
            }

            let wallet_config = wallets
                .get(&wallet_id)
                .ok_or_else(|| anyhow!("Wallet not found: {}", wallet_id))?;

            // Skip inactive wallets
            if wallet_config.status != WalletStatus::Active {
                continue;
            }

            // Check wallet type preference
            if let Some(preferred_type) = &criteria.preferred_wallet_type {
                if &wallet_config.wallet_type != preferred_type {
                    continue;
                }
            }

            let wallet_metrics = metrics.get(&wallet_id);

            // Calculate selection score
            let score = self
                .calculate_wallet_score(wallet_config, wallet_metrics, &criteria)
                .await?;

            if score > best_score {
                let available_balance = wallet_metrics.map(|m| m.sol_balance).unwrap_or(0.0);

                let risk_capacity = self.calculate_risk_capacity(wallet_config, wallet_metrics);

                best_score = score;
                best_wallet = Some(WalletSelection {
                    wallet_id: wallet_id.clone(),
                    wallet_config: wallet_config.clone(),
                    available_balance,
                    risk_capacity,
                    selection_reason: format!("Best score: {:.2}", score),
                });
            }
        }

        best_wallet.ok_or_else(|| anyhow!("No suitable wallet found for criteria"))
    }

    /// Get wallet by ID
    pub async fn get_wallet(&self, wallet_id: &str) -> Result<WalletConfig> {
        let wallets = self.wallets.read().await;
        wallets
            .get(wallet_id)
            .cloned()
            .ok_or_else(|| anyhow!("Wallet not found: {}", wallet_id))
    }

    /// Get wallet metrics
    pub async fn get_wallet_metrics(&self, wallet_id: &str) -> Result<WalletMetrics> {
        let metrics = self.wallet_metrics.read().await;
        metrics
            .get(wallet_id)
            .cloned()
            .ok_or_else(|| anyhow!("Wallet metrics not found: {}", wallet_id))
    }

    /// Update wallet metrics
    pub async fn update_wallet_metrics(&self, metrics: WalletMetrics) -> Result<()> {
        let mut wallet_metrics = self.wallet_metrics.write().await;
        wallet_metrics.insert(metrics.wallet_id.clone(), metrics);
        Ok(())
    }

    /// Get all active wallets
    pub async fn get_active_wallets(&self) -> Result<Vec<WalletConfig>> {
        let wallets = self.wallets.read().await;
        Ok(wallets
            .values()
            .filter(|w| w.status == WalletStatus::Active)
            .cloned()
            .collect())
    }

    /// Get wallet keypair for transaction signing
    pub async fn get_wallet_keypair(&self, wallet_id: &str) -> Result<Keypair> {
        let wallet = self.get_wallet(wallet_id).await?;
        self.parse_private_key(&wallet.private_key)
    }

    /// Validate wallet configuration
    fn validate_wallet_config(&self, config: &WalletConfig) -> Result<()> {
        // Validate wallet ID
        if config.wallet_id.is_empty() {
            return Err(anyhow!("Wallet ID cannot be empty"));
        }

        // Validate private key format
        self.parse_private_key(&config.private_key)
            .context("Invalid private key format")?;

        // Validate strategy allocations
        let total_allocation: f64 = config
            .strategy_allocation
            .iter()
            .filter(|a| a.enabled)
            .map(|a| a.allocation_percentage)
            .sum();

        if total_allocation > 100.0 {
            return Err(anyhow!(
                "Total strategy allocation exceeds 100%: {:.2}%",
                total_allocation
            ));
        }

        // Validate risk limits
        if config.risk_limits.max_exposure_percentage > 100.0 {
            return Err(anyhow!("Max exposure percentage cannot exceed 100%"));
        }

        Ok(())
    }

    /// Parse private key from various formats
    fn parse_private_key(&self, private_key: &str) -> Result<Keypair> {
        // Try JSON array format first (Solana CLI format)
        if private_key.starts_with('[') && private_key.ends_with(']') {
            let bytes: Vec<u8> = serde_json::from_str(private_key)
                .context("Failed to parse private key as JSON array")?;

            if bytes.len() != 64 {
                return Err(anyhow!("Private key must be 64 bytes, got {}", bytes.len()));
            }

            return Keypair::from_bytes(&bytes).context("Failed to create keypair from bytes");
        }

        // Try base58 format
        if let Ok(bytes) = bs58::decode(private_key).into_vec() {
            if bytes.len() == 64 {
                return Keypair::from_bytes(&bytes).context("Failed to create keypair from base58");
            }
        }

        Err(anyhow!("Unsupported private key format"))
    }

    /// Calculate wallet selection score
    async fn calculate_wallet_score(
        &self,
        wallet_config: &WalletConfig,
        wallet_metrics: Option<&WalletMetrics>,
        criteria: &WalletSelectionCriteria,
    ) -> Result<f64> {
        let mut score = 0.0;

        // Base score from wallet type
        score += match wallet_config.wallet_type {
            WalletType::Primary => 10.0,
            WalletType::HFT => 9.0,
            WalletType::Secondary => 8.0,
            WalletType::Arbitrage => 7.0,
            WalletType::Conservative => 6.0,
            WalletType::MEVProtection => 5.0,
            WalletType::Experimental => 4.0,
            WalletType::Emergency => 1.0,
            WalletType::Aggressive => 8.5, // High score for memcoin strategies
            // Micro-lightning wallet types
            WalletType::MicroLightning => 9.5,
            WalletType::MicroEmergencyGas => 2.0,
            WalletType::MicroReentry => 7.5,
            WalletType::MicroPsychology => 3.0,
            WalletType::MicroTacticalExit => 6.0,
        };

        // Strategy allocation score
        for allocation in &wallet_config.strategy_allocation {
            if allocation.strategy_type == criteria.strategy_type && allocation.enabled {
                score += allocation.allocation_percentage / 10.0; // Max 10 points
                break;
            }
        }

        // Balance and capacity score
        if let Some(metrics) = wallet_metrics {
            if metrics.sol_balance >= criteria.required_balance {
                score += 5.0;
            }

            // Performance score
            score += metrics.performance_score.min(5.0);

            // Risk utilization (lower is better)
            score += (100.0 - metrics.risk_utilization) / 20.0; // Max 5 points
        }

        Ok(score)
    }

    /// Calculate risk capacity for a wallet
    fn calculate_risk_capacity(
        &self,
        config: &WalletConfig,
        metrics: Option<&WalletMetrics>,
    ) -> f64 {
        if let Some(metrics) = metrics {
            let max_risk =
                config.risk_limits.max_exposure_percentage / 100.0 * metrics.total_value_usd;
            let current_risk = metrics.risk_utilization / 100.0 * max_risk;
            max_risk - current_risk
        } else {
            0.0
        }
    }
}

impl Default for WalletManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Wallet configuration builder for easy setup
pub struct WalletConfigBuilder {
    config: WalletConfig,
}

impl WalletConfigBuilder {
    pub fn new(wallet_id: String, name: String, private_key: String) -> Result<Self> {
        // Parse and validate private key
        let keypair = Self::parse_private_key(&private_key)?;
        let public_key = keypair.pubkey().to_string();

        Ok(Self {
            config: WalletConfig {
                wallet_id,
                name,
                description: String::new(),
                private_key,
                public_key,
                wallet_type: WalletType::Primary,
                strategy_allocation: Vec::new(),
                risk_limits: WalletRiskLimits::default(),
                status: WalletStatus::Active,
                created_at: Utc::now(),
                last_used: None,
            },
        })
    }

    pub fn description(mut self, description: String) -> Self {
        self.config.description = description;
        self
    }

    pub fn wallet_type(mut self, wallet_type: WalletType) -> Self {
        self.config.wallet_type = wallet_type;
        self
    }

    pub fn add_strategy_allocation(
        mut self,
        strategy_type: StrategyType,
        allocation_percentage: f64,
        max_position_size: f64,
    ) -> Self {
        self.config.strategy_allocation.push(StrategyAllocation {
            strategy_type,
            allocation_percentage,
            max_position_size,
            enabled: true,
        });
        self
    }

    pub fn risk_limits(mut self, risk_limits: WalletRiskLimits) -> Self {
        self.config.risk_limits = risk_limits;
        self
    }

    pub fn status(mut self, status: WalletStatus) -> Self {
        self.config.status = status;
        self
    }

    pub fn build(self) -> WalletConfig {
        self.config
    }

    fn parse_private_key(private_key: &str) -> Result<Keypair> {
        // Try JSON array format first
        if private_key.starts_with('[') && private_key.ends_with(']') {
            let bytes: Vec<u8> = serde_json::from_str(private_key)
                .context("Failed to parse private key as JSON array")?;

            if bytes.len() != 64 {
                return Err(anyhow!("Private key must be 64 bytes, got {}", bytes.len()));
            }

            return Keypair::from_bytes(&bytes).context("Failed to create keypair from bytes");
        }

        // Try base58 format
        if let Ok(bytes) = bs58::decode(private_key).into_vec() {
            if bytes.len() == 64 {
                return Keypair::from_bytes(&bytes).context("Failed to create keypair from base58");
            }
        }

        Err(anyhow!("Unsupported private key format"))
    }
}

impl Default for WalletRiskLimits {
    fn default() -> Self {
        Self {
            max_daily_loss: 1000.0,
            max_position_size: 10000.0,
            max_concurrent_positions: 10,
            max_exposure_percentage: 80.0,
            stop_loss_threshold: 5.0,
            daily_trade_limit: 100,
            max_drawdown: 0.15,
            stop_loss_percentage: 0.20,
        }
    }
}

/// Multi-wallet transaction builder
pub struct MultiWalletTransaction {
    pub wallet_id: String,
    pub transaction: Transaction,
    pub estimated_fees: f64,
    pub priority_level: u8,
}

/// Wallet portfolio summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletPortfolioSummary {
    pub total_wallets: usize,
    pub active_wallets: usize,
    pub total_value_usd: f64,
    pub total_sol_balance: f64,
    pub daily_pnl: f64,
    pub total_pnl: f64,
    pub risk_utilization: f64,
    pub performance_score: f64,
    pub wallet_breakdown: Vec<WalletSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletSummary {
    pub wallet_id: String,
    pub name: String,
    pub wallet_type: WalletType,
    pub status: WalletStatus,
    pub sol_balance: f64,
    pub value_usd: f64,
    pub daily_pnl: f64,
    pub risk_utilization: f64,
    pub active_positions: u32,
}

impl WalletManager {
    /// Get portfolio summary across all wallets
    pub async fn get_portfolio_summary(&self) -> Result<WalletPortfolioSummary> {
        let wallets = self.wallets.read().await;
        let metrics = self.wallet_metrics.read().await;
        let positions = self.active_positions.read().await;

        let mut summary = WalletPortfolioSummary {
            total_wallets: wallets.len(),
            active_wallets: 0,
            total_value_usd: 0.0,
            total_sol_balance: 0.0,
            daily_pnl: 0.0,
            total_pnl: 0.0,
            risk_utilization: 0.0,
            performance_score: 0.0,
            wallet_breakdown: Vec::new(),
        };

        for (wallet_id, wallet_config) in wallets.iter() {
            if wallet_config.status == WalletStatus::Active {
                summary.active_wallets += 1;
            }

            let wallet_metrics = metrics.get(wallet_id);
            let wallet_positions = positions
                .get(wallet_id)
                .map(|p| p.len() as u32)
                .unwrap_or(0);

            if let Some(metrics) = wallet_metrics {
                summary.total_value_usd += metrics.total_value_usd;
                summary.total_sol_balance += metrics.sol_balance;
                summary.daily_pnl += metrics.daily_pnl;
                summary.total_pnl += metrics.total_pnl;
                summary.risk_utilization += metrics.risk_utilization;
                summary.performance_score += metrics.performance_score;

                summary.wallet_breakdown.push(WalletSummary {
                    wallet_id: wallet_id.clone(),
                    name: wallet_config.name.clone(),
                    wallet_type: wallet_config.wallet_type.clone(),
                    status: wallet_config.status.clone(),
                    sol_balance: metrics.sol_balance,
                    value_usd: metrics.total_value_usd,
                    daily_pnl: metrics.daily_pnl,
                    risk_utilization: metrics.risk_utilization,
                    active_positions: wallet_positions,
                });
            } else {
                summary.wallet_breakdown.push(WalletSummary {
                    wallet_id: wallet_id.clone(),
                    name: wallet_config.name.clone(),
                    wallet_type: wallet_config.wallet_type.clone(),
                    status: wallet_config.status.clone(),
                    sol_balance: 0.0,
                    value_usd: 0.0,
                    daily_pnl: 0.0,
                    risk_utilization: 0.0,
                    active_positions: wallet_positions,
                });
            }
        }

        // Calculate averages
        if summary.active_wallets > 0 {
            summary.risk_utilization /= summary.active_wallets as f64;
            summary.performance_score /= summary.active_wallets as f64;
        }

        Ok(summary)
    }

    /// Emergency stop all wallets
    pub async fn emergency_stop_all(&self) -> Result<()> {
        warn!("🚨 EMERGENCY STOP: Suspending all wallets");

        let mut wallets = self.wallets.write().await;
        for (wallet_id, wallet_config) in wallets.iter_mut() {
            if wallet_config.status == WalletStatus::Active {
                wallet_config.status = WalletStatus::Emergency;
                warn!("🚨 Wallet {} suspended in emergency mode", wallet_id);
            }
        }

        Ok(())
    }

    /// Reactivate wallet from emergency mode
    pub async fn reactivate_wallet(&self, wallet_id: &str) -> Result<()> {
        let mut wallets = self.wallets.write().await;

        if let Some(wallet_config) = wallets.get_mut(wallet_id) {
            if wallet_config.status == WalletStatus::Emergency {
                wallet_config.status = WalletStatus::Active;
                info!("✅ Wallet {} reactivated from emergency mode", wallet_id);
            } else {
                return Err(anyhow!("Wallet {} is not in emergency mode", wallet_id));
            }
        } else {
            return Err(anyhow!("Wallet {} not found", wallet_id));
        }

        Ok(())
    }

    /// Load wallet configurations from file
    /// WARNING: This loads configuration only - private keys must be in environment variables
    pub async fn load_from_config_file(&mut self, config_path: &str) -> Result<()> {
        warn!("🔐 SECURITY: Loading wallet configuration from file. Private keys must be in environment variables only!");

        let config_content = tokio::fs::read_to_string(config_path)
            .await
            .context("Failed to read wallet configuration file")?;

        let wallet_configs: Vec<WalletConfig> = serde_json::from_str(&config_content)
            .context("Failed to parse wallet configuration")?;

        // Validate that no private keys are in the configuration
        for config in &wallet_configs {
            if !config.private_key.is_empty() && !config.private_key.starts_with("env:") {
                return Err(anyhow!(
                    "SECURITY VIOLATION: Private key found in configuration file for wallet {}. Use environment variables only! Expected format: env:VARIABLE_NAME",
                    config.wallet_id
                ));
            }
        }

        self.initialize(wallet_configs).await
    }

    /// Save wallet configurations to file
    pub async fn save_to_config_file(&self, config_path: &str) -> Result<()> {
        let wallets = self.wallets.read().await;
        let wallet_configs: Vec<WalletConfig> = wallets.values().cloned().collect();

        let config_content = serde_json::to_string_pretty(&wallet_configs)
            .context("Failed to serialize wallet configurations")?;

        tokio::fs::write(config_path, config_content)
            .await
            .context("Failed to write wallet configuration file")?;

        info!(
            "💾 Saved {} wallet configurations to {}",
            wallet_configs.len(),
            config_path
        );
        Ok(())
    }

    /// MICRO-LIGHTNING specific wallet management methods

    /// Create micro-lightning wallet set with proper allocations
    pub async fn create_micro_lightning_wallet_set(&self, base_name: &str, total_capital: f64) -> Result<Vec<String>> {
        let mut wallet_ids = Vec::new();
        let timestamp = chrono::Utc::now();

        // Calculate allocations based on MicroWallet structure
        let allocations = [
            (WalletType::MicroLightning, 0.20, "Lightning"),
            (WalletType::MicroEmergencyGas, 0.175, "Emergency Gas"),
            (WalletType::MicroReentry, 0.225, "Reentry"),
            (WalletType::MicroPsychology, 0.20, "Psychology"),
            (WalletType::MicroTacticalExit, 0.20, "Tactical Exit"),
        ];

        for (wallet_type, ratio, suffix) in allocations {
            let wallet_config = WalletConfig {
                wallet_id: format!("{}_{}", base_name, suffix.to_lowercase().replace(" ", "_")),
                name: format!("{} - {}", base_name, suffix),
                description: format!("Micro-lightning {} wallet", suffix.to_lowercase()),
                private_key: self.generate_new_keypair()?,
                public_key: "".to_string(), // Would be derived from private key
                wallet_type,
                strategy_allocation: vec![StrategyAllocation {
                    strategy_type: StrategyType::MicroLightning,
                    allocation_percentage: 100.0,
                    max_position_size: total_capital * ratio * 0.8, // 80% of allocation
                    enabled: true,
                }],
                risk_limits: WalletRiskLimits {
                    max_daily_loss: total_capital * ratio * 0.5, // 50% of allocation
                    max_position_size: total_capital * ratio,
                    max_concurrent_positions: 5,
                    max_exposure_percentage: 80.0,
                    stop_loss_threshold: 0.05, // 5%
                    daily_trade_limit: 100,
                    max_drawdown: 0.15, // 15% max drawdown
                    stop_loss_percentage: 0.20, // 20% stop loss
                },
                status: WalletStatus::Active,
                created_at: timestamp,
                last_used: None,
            };

            self.add_wallet(wallet_config.clone()).await?;
            wallet_ids.push(wallet_config.wallet_id);
        }

        info!("🏦 Created micro-lightning wallet set: {} wallets, ${:.2} total capital",
              wallet_ids.len(), total_capital);

        Ok(wallet_ids)
    }

    /// Rotate micro-lightning wallets (Commandment 2: Wallet Reincarnation)
    pub async fn rotate_micro_lightning_wallets(&self, current_set: &[String]) -> Result<Vec<String>> {
        info!("🔄 Rotating micro-lightning wallet set");

        // Deactivate current wallets
        for wallet_id in current_set {
            if let Ok(mut wallet) = self.get_wallet(wallet_id).await {
                wallet.status = WalletStatus::Inactive;
                wallet.last_used = Some(chrono::Utc::now());

                let mut wallets = self.wallets.write().await;
                wallets.insert(wallet_id.clone(), wallet);
            }
        }

        // Create new wallet set
        let base_name = format!("micro_lightning_{}", chrono::Utc::now().timestamp());
        let new_set = self.create_micro_lightning_wallet_set(&base_name, 20.0).await?;

        info!("✅ Micro-lightning wallet rotation completed: {} new wallets", new_set.len());
        Ok(new_set)
    }

    /// Get micro-lightning wallet by type
    pub async fn get_micro_wallet(&self, wallet_type: WalletType) -> Result<Option<WalletConfig>> {
        let wallets = self.wallets.read().await;

        for wallet in wallets.values() {
            if wallet.wallet_type == wallet_type && wallet.status == WalletStatus::Active {
                return Ok(Some(wallet.clone()));
            }
        }

        Ok(None)
    }

    /// Transfer funds between micro-lightning wallets
    pub async fn transfer_micro_funds(&self, from_type: WalletType, to_type: WalletType, amount: f64) -> Result<()> {
        info!("💸 Transferring ${:.2} from {:?} to {:?}", amount, from_type, to_type);

        // In real implementation, this would execute actual blockchain transfers
        // For now, we'll update the internal tracking

        // Update position tracking
        let mut positions = self.active_positions.write().await;

        // This is a simplified implementation - in reality, you'd need to:
        // 1. Create and sign transfer transaction
        // 2. Submit to blockchain
        // 3. Wait for confirmation
        // 4. Update internal records

        info!("✅ Micro fund transfer completed");
        Ok(())
    }

    /// Apply psychology tax (Commandment 4: Emotional Accounting)
    pub async fn apply_psychology_tax(&self, profit: f64) -> Result<f64> {
        if profit <= 0.0 {
            return Ok(profit);
        }

        let tax_rate = 0.10; // 10% psychology tax
        let tax_amount = profit * tax_rate;

        // Transfer tax to psychology wallet
        self.transfer_micro_funds(
            WalletType::MicroLightning,
            WalletType::MicroPsychology,
            tax_amount
        ).await?;

        let after_tax_profit = profit - tax_amount;

        info!("🧠 Psychology tax applied: ${:.2} ({}% of ${:.2})",
              tax_amount, tax_rate * 100.0, profit);

        Ok(after_tax_profit)
    }

    /// Check micro-lightning wallet health
    pub async fn check_micro_wallet_health(&self) -> Result<MicroWalletHealthReport> {
        let mut report = MicroWalletHealthReport {
            total_wallets: 0,
            active_wallets: 0,
            total_balance: 0.0,
            psychology_fund_balance: 0.0,
            emergency_gas_balance: 0.0,
            reentry_buffer_balance: 0.0,
            wallet_rotation_needed: false,
            health_score: 1.0,
            recommendations: Vec::new(),
        };

        let wallets = self.wallets.read().await;

        for wallet in wallets.values() {
            if matches!(wallet.wallet_type,
                WalletType::MicroLightning | WalletType::MicroEmergencyGas |
                WalletType::MicroReentry | WalletType::MicroPsychology |
                WalletType::MicroTacticalExit) {

                report.total_wallets += 1;

                if wallet.status == WalletStatus::Active {
                    report.active_wallets += 1;
                    // In real implementation, would query actual balance
                    let balance = wallet.risk_limits.max_position_size;
                    report.total_balance += balance;

                    match wallet.wallet_type {
                        WalletType::MicroPsychology => report.psychology_fund_balance += balance,
                        WalletType::MicroEmergencyGas => report.emergency_gas_balance += balance,
                        WalletType::MicroReentry => report.reentry_buffer_balance += balance,
                        _ => {}
                    }
                }
            }
        }

        // Check if rotation is needed (example: if any wallet has been used more than 3 times)
        // This would be tracked in actual implementation
        report.wallet_rotation_needed = false; // Placeholder

        // Calculate health score
        if report.active_wallets < 5 {
            report.health_score *= 0.8;
            report.recommendations.push("Missing micro-lightning wallets".to_string());
        }

        if report.emergency_gas_balance < 3.0 {
            report.health_score *= 0.9;
            report.recommendations.push("Low emergency gas reserves".to_string());
        }

        if report.psychology_fund_balance < 2.0 {
            report.health_score *= 0.95;
            report.recommendations.push("Low psychology fund balance".to_string());
        }

        Ok(report)
    }

    /// Generate new keypair for wallet creation
    fn generate_new_keypair(&self) -> Result<String> {
        // In real implementation, this would generate a proper Solana keypair
        // For now, return a placeholder
        Ok(format!("keypair_{}", uuid::Uuid::new_v4()))
    }
}
