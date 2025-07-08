// THE OVERMIND PROTOCOL - Sniple Configuration Module
// Dynamic configuration management with environment overrides and validation

use anyhow::{anyhow, Result};
use config::{Config, File, Environment};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnipleConfig {
    pub rpc_endpoints: Vec<String>,
    pub jito_block_engine_url: Option<String>,
    pub dex_aggregators: Vec<String>,
    pub risk_limits: RiskLimits,
    pub trading_config: TradingConfig,
    pub security_config: SecurityConfig,
    pub performance_config: PerformanceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskLimits {
    pub max_position_size_usd: f64,
    pub max_daily_loss_percent: f64,
    pub max_slippage_bps: u16,
    pub max_leverage: f64,
    pub stop_loss_percent: f64,
    pub take_profit_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingConfig {
    pub trading_mode: String, // "paper" or "live"
    pub default_trade_size_usd: f64,
    pub min_trade_size_usd: f64,
    pub max_trade_size_usd: f64,
    pub auto_compound: bool,
    pub compound_threshold_usd: f64,
    pub preferred_dexes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub encryption_enabled: bool,
    pub vault_enabled: bool,
    pub multi_sig_required: bool,
    pub ip_whitelist: Vec<String>,
    pub rate_limit_per_minute: u32,
    pub session_timeout_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub max_concurrent_trades: u32,
    pub rpc_timeout_ms: u64,
    pub cache_ttl_seconds: u64,
    pub batch_size: u32,
    pub worker_threads: u32,
    pub memory_limit_mb: u64,
}

impl Default for RiskLimits {
    fn default() -> Self {
        Self {
            max_position_size_usd: 1000.0,
            max_daily_loss_percent: 5.0,
            max_slippage_bps: 100, // 1%
            max_leverage: 1.0, // No leverage by default
            stop_loss_percent: 10.0,
            take_profit_percent: 20.0,
        }
    }
}

impl Default for TradingConfig {
    fn default() -> Self {
        Self {
            trading_mode: "paper".to_string(),
            default_trade_size_usd: 100.0,
            min_trade_size_usd: 10.0,
            max_trade_size_usd: 1000.0,
            auto_compound: false,
            compound_threshold_usd: 1000.0,
            preferred_dexes: vec![
                "Jupiter".to_string(),
                "Raydium".to_string(),
                "Orca".to_string(),
            ],
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            encryption_enabled: true,
            vault_enabled: true,
            multi_sig_required: false,
            ip_whitelist: vec![],
            rate_limit_per_minute: 60,
            session_timeout_minutes: 30,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_concurrent_trades: 10,
            rpc_timeout_ms: 5000,
            cache_ttl_seconds: 5,
            batch_size: 100,
            worker_threads: 4,
            memory_limit_mb: 2048,
        }
    }
}

impl Default for SnipleConfig {
    fn default() -> Self {
        Self {
            rpc_endpoints: vec![
                "https://api.mainnet-beta.solana.com".to_string(),
                "https://solana-api.projectserum.com".to_string(),
            ],
            jito_block_engine_url: Some("https://mainnet.block-engine.jito.wtf".to_string()),
            dex_aggregators: vec![
                "Jupiter".to_string(),
                "Raydium".to_string(),
                "Orca".to_string(),
            ],
            risk_limits: RiskLimits::default(),
            trading_config: TradingConfig::default(),
            security_config: SecurityConfig::default(),
            performance_config: PerformanceConfig::default(),
        }
    }
}

impl SnipleConfig {
    /// Load configuration with environment overrides
    pub fn load() -> Result<Self> {
        info!("📋 Loading Sniple configuration");
        
        let mut cfg = Config::builder();
        
        // Start with default configuration
        let default_config = SnipleConfig::default();
        cfg = cfg.add_source(config::Config::try_from(&default_config)?);
        
        // Load base configuration file
        cfg = cfg.add_source(File::with_name("config/base").required(false));
        
        // Load environment-specific configuration
        if let Ok(env) = std::env::var("SNIPLE_ENV") {
            info!("🌍 Loading environment-specific config: {}", env);
            cfg = cfg.add_source(File::with_name(&format!("config/{}", env)).required(false));
        }
        
        // Environment variables override (highest priority)
        cfg = cfg.add_source(Environment::with_prefix("SNIPLE").separator("_"));
        
        let config: SnipleConfig = cfg.build()?.try_deserialize()?;
        
        // Validate configuration
        config.validate()?;
        
        info!("✅ Configuration loaded successfully");
        info!("📊 Trading mode: {}", config.trading_config.trading_mode);
        info!("🔒 Security enabled: {}", config.security_config.encryption_enabled);
        info!("🚀 Max concurrent trades: {}", config.performance_config.max_concurrent_trades);
        
        Ok(config)
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<()> {
        // Validate RPC endpoints
        if self.rpc_endpoints.is_empty() {
            return Err(anyhow!("At least one RPC endpoint must be configured"));
        }

        for endpoint in &self.rpc_endpoints {
            if !endpoint.starts_with("http") {
                return Err(anyhow!("Invalid RPC endpoint format: {}", endpoint));
            }
        }

        // Validate risk limits
        if self.risk_limits.max_position_size_usd <= 0.0 {
            return Err(anyhow!("Max position size must be positive"));
        }

        if self.risk_limits.max_daily_loss_percent <= 0.0 || self.risk_limits.max_daily_loss_percent > 100.0 {
            return Err(anyhow!("Daily loss percent must be between 0 and 100"));
        }

        if self.risk_limits.max_slippage_bps > 10000 {
            return Err(anyhow!("Max slippage cannot exceed 100% (10000 bps)"));
        }

        // Validate trading config
        if !["paper", "live"].contains(&self.trading_config.trading_mode.as_str()) {
            return Err(anyhow!("Trading mode must be 'paper' or 'live'"));
        }

        if self.trading_config.min_trade_size_usd >= self.trading_config.max_trade_size_usd {
            return Err(anyhow!("Min trade size must be less than max trade size"));
        }

        // Validate performance config
        if self.performance_config.max_concurrent_trades == 0 {
            return Err(anyhow!("Max concurrent trades must be at least 1"));
        }

        if self.performance_config.worker_threads == 0 {
            return Err(anyhow!("Worker threads must be at least 1"));
        }

        info!("✅ Configuration validation passed");
        Ok(())
    }

    /// Get configuration as environment variables map
    pub fn to_env_vars(&self) -> HashMap<String, String> {
        let mut env_vars = HashMap::new();
        
        // RPC configuration
        env_vars.insert("SNIPLE_RPC_ENDPOINTS".to_string(), self.rpc_endpoints.join(","));
        
        if let Some(jito_url) = &self.jito_block_engine_url {
            env_vars.insert("SNIPLE_JITO_BLOCK_ENGINE_URL".to_string(), jito_url.clone());
        }
        
        // Trading configuration
        env_vars.insert("SNIPLE_TRADING_MODE".to_string(), self.trading_config.trading_mode.clone());
        env_vars.insert("SNIPLE_DEFAULT_TRADE_SIZE_USD".to_string(), self.trading_config.default_trade_size_usd.to_string());
        
        // Risk limits
        env_vars.insert("SNIPLE_MAX_POSITION_SIZE_USD".to_string(), self.risk_limits.max_position_size_usd.to_string());
        env_vars.insert("SNIPLE_MAX_DAILY_LOSS_PERCENT".to_string(), self.risk_limits.max_daily_loss_percent.to_string());
        env_vars.insert("SNIPLE_MAX_SLIPPAGE_BPS".to_string(), self.risk_limits.max_slippage_bps.to_string());
        
        // Performance configuration
        env_vars.insert("SNIPLE_MAX_CONCURRENT_TRADES".to_string(), self.performance_config.max_concurrent_trades.to_string());
        env_vars.insert("SNIPLE_RPC_TIMEOUT_MS".to_string(), self.performance_config.rpc_timeout_ms.to_string());
        
        env_vars
    }

    /// Update configuration from environment variables
    pub fn update_from_env(&mut self) -> Result<()> {
        // Update trading mode
        if let Ok(mode) = std::env::var("SNIPLE_TRADING_MODE") {
            if ["paper", "live"].contains(&mode.as_str()) {
                self.trading_config.trading_mode = mode;
            }
        }

        // Update risk limits
        if let Ok(max_pos) = std::env::var("SNIPLE_MAX_POSITION_SIZE_USD") {
            if let Ok(value) = max_pos.parse::<f64>() {
                self.risk_limits.max_position_size_usd = value;
            }
        }

        if let Ok(max_loss) = std::env::var("SNIPLE_MAX_DAILY_LOSS_PERCENT") {
            if let Ok(value) = max_loss.parse::<f64>() {
                self.risk_limits.max_daily_loss_percent = value;
            }
        }

        // Update performance config
        if let Ok(max_trades) = std::env::var("SNIPLE_MAX_CONCURRENT_TRADES") {
            if let Ok(value) = max_trades.parse::<u32>() {
                self.performance_config.max_concurrent_trades = value;
            }
        }

        // Validate after updates
        self.validate()?;
        
        info!("🔄 Configuration updated from environment variables");
        Ok(())
    }

    /// Check if trading is enabled
    pub fn is_live_trading(&self) -> bool {
        self.trading_config.trading_mode == "live"
    }

    /// Check if paper trading is enabled
    pub fn is_paper_trading(&self) -> bool {
        self.trading_config.trading_mode == "paper"
    }

    /// Get maximum position size in USD
    pub fn max_position_size(&self) -> f64 {
        self.risk_limits.max_position_size_usd
    }

    /// Get maximum daily loss percentage
    pub fn max_daily_loss(&self) -> f64 {
        self.risk_limits.max_daily_loss_percent
    }

    /// Get preferred DEX list
    pub fn preferred_dexes(&self) -> &Vec<String> {
        &self.trading_config.preferred_dexes
    }

    /// Check if Jito is enabled
    pub fn jito_enabled(&self) -> bool {
        self.jito_block_engine_url.is_some()
    }

    /// Get Jito endpoint URL
    pub fn jito_endpoint(&self) -> Option<&String> {
        self.jito_block_engine_url.as_ref()
    }

    /// Save configuration to file
    pub fn save_to_file(&self, path: &str) -> Result<()> {
        let config_toml = toml::to_string_pretty(self)
            .map_err(|e| anyhow!("Failed to serialize config: {}", e))?;
        
        std::fs::write(path, config_toml)
            .map_err(|e| anyhow!("Failed to write config file: {}", e))?;
        
        info!("💾 Configuration saved to: {}", path);
        Ok(())
    }

    /// Create configuration for specific environment
    pub fn for_environment(env: &str) -> Result<Self> {
        let mut config = Self::default();
        
        match env {
            "development" => {
                config.trading_config.trading_mode = "paper".to_string();
                config.risk_limits.max_position_size_usd = 100.0;
                config.performance_config.max_concurrent_trades = 5;
            }
            "staging" => {
                config.trading_config.trading_mode = "paper".to_string();
                config.risk_limits.max_position_size_usd = 500.0;
                config.performance_config.max_concurrent_trades = 10;
            }
            "production" => {
                config.trading_config.trading_mode = "live".to_string();
                config.risk_limits.max_position_size_usd = 1000.0;
                config.performance_config.max_concurrent_trades = 20;
                config.security_config.multi_sig_required = true;
            }
            _ => {
                warn!("⚠️ Unknown environment '{}', using default config", env);
            }
        }
        
        config.validate()?;
        Ok(config)
    }
}
