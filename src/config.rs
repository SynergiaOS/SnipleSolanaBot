// Configuration management for SNIPERCOR
// Handles environment variables and system configuration

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{info, warn};
use crate::security::infisical_client::SecureEnvLoader;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub trading: TradingConfig,
    pub solana: SolanaConfig,
    pub api: ApiConfig,
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub logging: LoggingConfig,
    // THE OVERMIND PROTOCOL - HFT Engine Configuration
    pub overmind: OvermindConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingConfig {
    pub mode: TradingMode,
    pub max_position_size: f64,
    pub max_daily_loss: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradingMode {
    Paper,
    Live,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaConfig {
    pub rpc_url: String,
    pub wallet_private_key: String,
    // Multi-wallet support
    pub multi_wallet_enabled: bool,
    pub default_wallet_id: Option<String>,
    // RPC Failover support
    pub rpc_endpoints: Vec<RpcEndpoint>,
    pub failover_enabled: bool,
    pub health_check_interval_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcEndpoint {
    pub url: String,
    pub name: String,
    pub priority: u8, // 1 = highest priority
    pub timeout_ms: u64,
    pub max_retries: u32,
    pub is_healthy: bool,
    pub last_health_check: Option<chrono::DateTime<chrono::Utc>>,
    pub avg_latency_ms: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub helius_api_key: String,
    pub helius_rpc_url: String,
    pub helius_ws_url: String,
    pub quicknode_api_key: String,
    pub quicknode_ws_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
}

// THE OVERMIND PROTOCOL - HFT Engine Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OvermindConfig {
    pub enabled: bool,
    pub tensorzero_gateway_url: String,
    pub jito_endpoint: String,
    pub max_execution_latency_ms: u64,
    pub ai_confidence_threshold: f64,
}

#[allow(dead_code)]
impl Config {
    /// Get multiple RPC endpoints with failover configuration
    fn get_rpc_endpoints() -> Result<Vec<RpcEndpoint>> {
        let app_env = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());

        let endpoints = match app_env.as_str() {
            "development" => vec![
                RpcEndpoint {
                    url: env::var("QUICKNODE_DEVNET_RPC_URL")
                        .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string()),
                    name: "QuickNode Devnet".to_string(),
                    priority: 1,
                    timeout_ms: 5000,
                    max_retries: 3,
                    is_healthy: true,
                    last_health_check: None,
                    avg_latency_ms: None,
                },
                RpcEndpoint {
                    url: "https://api.devnet.solana.com".to_string(),
                    name: "Solana Devnet".to_string(),
                    priority: 2,
                    timeout_ms: 8000,
                    max_retries: 2,
                    is_healthy: true,
                    last_health_check: None,
                    avg_latency_ms: None,
                },
            ],
            "production" | "live" => vec![
                RpcEndpoint {
                    url: env::var("QUICKNODE_MAINNET_RPC_URL").unwrap_or_else(|_| {
                        env::var("SOLANA_RPC_URL")
                            .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
                    }),
                    name: "QuickNode Mainnet".to_string(),
                    priority: 1,
                    timeout_ms: 3000,
                    max_retries: 3,
                    is_healthy: true,
                    last_health_check: None,
                    avg_latency_ms: None,
                },
                RpcEndpoint {
                    url: env::var("HELIUS_RPC_URL")
                        .unwrap_or_else(|_| "https://mainnet.helius-rpc.com".to_string()),
                    name: "Helius Mainnet".to_string(),
                    priority: 2,
                    timeout_ms: 4000,
                    max_retries: 2,
                    is_healthy: true,
                    last_health_check: None,
                    avg_latency_ms: None,
                },
                RpcEndpoint {
                    url: "https://api.mainnet-beta.solana.com".to_string(),
                    name: "Solana Mainnet".to_string(),
                    priority: 3,
                    timeout_ms: 8000,
                    max_retries: 2,
                    is_healthy: true,
                    last_health_check: None,
                    avg_latency_ms: None,
                },
            ],
            _ => vec![RpcEndpoint {
                url: env::var("SOLANA_RPC_URL")
                    .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string()),
                name: "Default RPC".to_string(),
                priority: 1,
                timeout_ms: 5000,
                max_retries: 3,
                is_healthy: true,
                last_health_check: None,
                avg_latency_ms: None,
            }],
        };

        Ok(endpoints)
    }

    /// Get dynamic RPC URL based on APP_ENV
    fn get_dynamic_rpc_url() -> Result<String> {
        let app_env = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());

        match app_env.as_str() {
            "development" => {
                // Devnet configuration
                env::var("QUICKNODE_DEVNET_RPC_URL")
                    .or_else(|_| env::var("SOLANA_DEVNET_RPC_URL"))
                    .or_else(|_| env::var("SOLANA_RPC_URL"))
                    .context("No Devnet RPC URL configured")
            }
            "production" | "live" => {
                // Mainnet configuration
                env::var("QUICKNODE_MAINNET_RPC_URL")
                    .or_else(|_| env::var("SOLANA_MAINNET_RPC_URL"))
                    .or_else(|_| env::var("SOLANA_RPC_URL"))
                    .context("No Mainnet RPC URL configured")
            }
            _ => {
                // Fallback to any available RPC URL
                env::var("SOLANA_RPC_URL").context("SOLANA_RPC_URL is required")
            }
        }
    }

    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok(); // Load .env file if present (fallback only)

        let trading_mode = match env::var("SNIPER_TRADING_MODE")
            .unwrap_or_else(|_| "paper".to_string())
            .to_lowercase()
            .as_str()
        {
            "live" => TradingMode::Live,
            _ => TradingMode::Paper,
        };

        let config = Config {
            trading: TradingConfig {
                mode: trading_mode,
                max_position_size: env::var("SNIPER_MAX_POSITION_SIZE")
                    .unwrap_or_else(|_| "1000".to_string())
                    .parse()
                    .context("Invalid SNIPER_MAX_POSITION_SIZE")?,
                max_daily_loss: env::var("SNIPER_MAX_DAILY_LOSS")
                    .unwrap_or_else(|_| "500".to_string())
                    .parse()
                    .context("Invalid SNIPER_MAX_DAILY_LOSS")?,
            },
            solana: SolanaConfig {
                rpc_url: Self::get_dynamic_rpc_url().unwrap_or_else(|_| "https://api.devnet.solana.com".to_string()),
                wallet_private_key: env::var("SNIPER_WALLET_PRIVATE_KEY")
                    .unwrap_or_else(|_| "placeholder-private-key-for-development".to_string()),
                multi_wallet_enabled: env::var("OVERMIND_MULTI_WALLET_ENABLED")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .unwrap_or(false),
                default_wallet_id: env::var("OVERMIND_DEFAULT_WALLET").ok(),
                rpc_endpoints: Self::get_rpc_endpoints()?,
                failover_enabled: env::var("SOLANA_RPC_FAILOVER_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                health_check_interval_ms: env::var("SOLANA_RPC_HEALTH_CHECK_INTERVAL_MS")
                    .unwrap_or_else(|_| "30000".to_string())
                    .parse()
                    .unwrap_or(30000),
            },
            api: ApiConfig {
                helius_api_key: env::var("SNIPER_HELIUS_API_KEY")
                    .unwrap_or_else(|_| "155e1444-1d0d-4a79-a6c7-0c2e89e77f0c".to_string()),
                helius_rpc_url: env::var("SNIPER_HELIUS_RPC_URL")
                    .unwrap_or_else(|_| "https://mainnet.helius-rpc.com".to_string()),
                helius_ws_url: env::var("SNIPER_HELIUS_WS_URL")
                    .unwrap_or_else(|_| "wss://mainnet.helius-rpc.com".to_string()),
                quicknode_api_key: env::var("SNIPER_QUICKNODE_API_KEY")
                    .unwrap_or_else(|_| "placeholder-key".to_string()),
                quicknode_ws_url: env::var("SNIPER_QUICKNODE_WS_URL")
                    .unwrap_or_else(|_| "wss://api.mainnet-beta.solana.com".to_string()),
            },
            database: DatabaseConfig {
                url: env::var("SNIPER_DATABASE_URL")
                    .unwrap_or_else(|_| "sqlite://overmind.db".to_string()),
            },
            server: ServerConfig {
                port: env::var("SNIPER_SERVER_PORT")
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()
                    .context("Invalid SNIPER_SERVER_PORT")?,
            },
            logging: LoggingConfig {
                level: env::var("SNIPER_LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            },
            // THE OVERMIND PROTOCOL - HFT Engine Configuration
            overmind: OvermindConfig {
                enabled: env::var("OVERMIND_ENABLED")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .unwrap_or(false),
                tensorzero_gateway_url: env::var("OVERMIND_TENSORZERO_URL")
                    .unwrap_or_else(|_| "http://localhost:3000".to_string()),
                jito_endpoint: env::var("OVERMIND_JITO_ENDPOINT")
                    .unwrap_or_else(|_| "https://mainnet.block-engine.jito.wtf".to_string()),
                max_execution_latency_ms: env::var("OVERMIND_MAX_LATENCY_MS")
                    .unwrap_or_else(|_| "25".to_string())
                    .parse()
                    .unwrap_or(25),
                ai_confidence_threshold: env::var("OVERMIND_AI_CONFIDENCE_THRESHOLD")
                    .unwrap_or_else(|_| "0.7".to_string())
                    .parse()
                    .unwrap_or(0.7),
            },
        };

        // Validate configuration
        config.validate()?;

        Ok(config)
    }

    /// Load configuration from Infisical with environment fallback
    pub async fn from_infisical() -> Result<Self> {
        info!("🔐 Loading configuration from Infisical...");

        let secure_loader = SecureEnvLoader::new(true).await;

        let trading_mode = match secure_loader.get("SNIPER_TRADING_MODE").await
            .unwrap_or_else(|| "paper".to_string())
            .to_lowercase()
            .as_str()
        {
            "live" => TradingMode::Live,
            _ => TradingMode::Paper,
        };

        let config = Config {
            trading: TradingConfig {
                mode: trading_mode,
                max_position_size: secure_loader.get("SNIPER_MAX_POSITION_SIZE").await
                    .unwrap_or_else(|| "1000".to_string())
                    .parse()
                    .context("Invalid SNIPER_MAX_POSITION_SIZE")?,
                max_daily_loss: secure_loader.get("SNIPER_MAX_DAILY_LOSS").await
                    .unwrap_or_else(|| "500".to_string())
                    .parse()
                    .context("Invalid SNIPER_MAX_DAILY_LOSS")?,
            },
            solana: SolanaConfig {
                rpc_url: secure_loader.get("SOLANA_RPC_URL").await
                    .unwrap_or_else(|| "https://api.devnet.solana.com".to_string()),
                wallet_private_key: secure_loader.get("SNIPER_WALLET_PRIVATE_KEY").await
                    .unwrap_or_else(|| "placeholder-private-key-for-development".to_string()),
                multi_wallet_enabled: secure_loader.get("OVERMIND_MULTI_WALLET_ENABLED").await
                    .unwrap_or_else(|| "false".to_string())
                    .parse()
                    .unwrap_or(false),
                max_slippage: secure_loader.get("SNIPER_MAX_SLIPPAGE").await
                    .unwrap_or_else(|| "0.05".to_string())
                    .parse()
                    .unwrap_or(0.05),
                priority_fee: secure_loader.get("SNIPER_PRIORITY_FEE").await
                    .unwrap_or_else(|| "0.001".to_string())
                    .parse()
                    .unwrap_or(0.001),
            },
            ai: AIConfig {
                enabled: secure_loader.get("OVERMIND_AI_MODE").await
                    .unwrap_or_else(|| "enabled".to_string()) == "enabled",
                model: secure_loader.get("OVERMIND_AI_MODEL").await
                    .unwrap_or_else(|| "gpt-4".to_string()),
                max_tokens: secure_loader.get("OVERMIND_AI_MAX_TOKENS").await
                    .unwrap_or_else(|| "4000".to_string())
                    .parse()
                    .unwrap_or(4000),
                temperature: secure_loader.get("OVERMIND_AI_TEMPERATURE").await
                    .unwrap_or_else(|| "0.7".to_string())
                    .parse()
                    .unwrap_or(0.7),
                openai_api_key: secure_loader.get("OPENAI_API_KEY").await
                    .unwrap_or_else(|| "placeholder-openai-key".to_string()),
            },
            api: APIConfig {
                helius_api_key: secure_loader.get("HELIUS_API_KEY").await
                    .unwrap_or_else(|| "placeholder-helius-key".to_string()),
                quicknode_api_key: secure_loader.get("QUICKNODE_API_KEY").await
                    .unwrap_or_else(|| "placeholder-quicknode-key".to_string()),
                jina_api_key: secure_loader.get("JINA_API_KEY").await
                    .unwrap_or_else(|| "placeholder-jina-key".to_string()),
                deepseek_api_key: secure_loader.get("DEEPSEEK_API_KEY").await
                    .unwrap_or_else(|| "placeholder-deepseek-key".to_string()),
            },
            monitoring: MonitoringConfig {
                prometheus_enabled: secure_loader.get("PROMETHEUS_ENABLED").await
                    .unwrap_or_else(|| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                prometheus_port: secure_loader.get("PROMETHEUS_PORT").await
                    .unwrap_or_else(|| "9090".to_string())
                    .parse()
                    .unwrap_or(9090),
                log_level: secure_loader.get("LOG_LEVEL").await
                    .unwrap_or_else(|| "info".to_string()),
                enable_metrics: secure_loader.get("ENABLE_METRICS").await
                    .unwrap_or_else(|| "true".to_string())
                    .parse()
                    .unwrap_or(true),
            },
        };

        info!("✅ Configuration loaded from Infisical successfully");
        Ok(config)
    }

    /// Validate configuration values
    fn validate(&self) -> Result<()> {
        if self.trading.max_position_size <= 0.0 {
            anyhow::bail!("max_position_size must be positive");
        }

        if self.trading.max_daily_loss <= 0.0 {
            anyhow::bail!("max_daily_loss must be positive");
        }

        if self.server.port == 0 {
            anyhow::bail!("server port must be valid");
        }

        Ok(())
    }

    /// Check if running in live trading mode
    pub fn is_live_trading(&self) -> bool {
        matches!(self.trading.mode, TradingMode::Live)
    }

    /// Get trading mode as string
    pub fn trading_mode_str(&self) -> &'static str {
        match self.trading.mode {
            TradingMode::Paper => "paper",
            TradingMode::Live => "live",
        }
    }

    /// Check if THE OVERMIND PROTOCOL is enabled
    pub fn is_overmind_enabled(&self) -> bool {
        self.overmind.enabled
    }

    /// Get OVERMIND mode description
    pub fn overmind_mode_str(&self) -> &'static str {
        if self.overmind.enabled {
            "THE OVERMIND PROTOCOL (AI-Enhanced)"
        } else {
            "Standard Mode"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use std::env; // Commented out to avoid unused import warning

    #[test]
    fn test_config_validation() {
        let mut config = Config {
            trading: TradingConfig {
                mode: TradingMode::Paper,
                max_position_size: 1000.0,
                max_daily_loss: 500.0,
            },
            solana: SolanaConfig {
                rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
                wallet_private_key: "test_key".to_string(),
                multi_wallet_enabled: false,
                default_wallet_id: None,
                rpc_endpoints: vec![],
                failover_enabled: false,
                health_check_interval_ms: 30000,
            },
            api: ApiConfig {
                helius_api_key: "test_key".to_string(),
                helius_rpc_url: "https://devnet.helius-rpc.com".to_string(),
                helius_ws_url: "wss://devnet.helius-rpc.com".to_string(),
                quicknode_api_key: "test_key".to_string(),
                quicknode_ws_url: "wss://test.quiknode.pro".to_string(),
            },
            database: DatabaseConfig {
                url: "postgresql://test".to_string(),
            },
            server: ServerConfig { port: 8080 },
            logging: LoggingConfig {
                level: "info".to_string(),
            },
            overmind: OvermindConfig {
                enabled: false,
                tensorzero_gateway_url: "http://localhost:3000".to_string(),
                jito_endpoint: "https://mainnet.block-engine.jito.wtf".to_string(),
                max_execution_latency_ms: 25,
                ai_confidence_threshold: 0.7,
            },
        };

        assert!(config.validate().is_ok());

        // Test invalid position size
        config.trading.max_position_size = -100.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_trading_mode() {
        let config = Config {
            trading: TradingConfig {
                mode: TradingMode::Paper,
                max_position_size: 1000.0,
                max_daily_loss: 500.0,
            },
            solana: SolanaConfig {
                rpc_url: "test".to_string(),
                wallet_private_key: "test".to_string(),
                multi_wallet_enabled: false,
                default_wallet_id: None,
                rpc_endpoints: vec![],
                failover_enabled: false,
                health_check_interval_ms: 30000,
            },
            api: ApiConfig {
                helius_api_key: "test".to_string(),
                helius_rpc_url: "https://devnet.helius-rpc.com".to_string(),
                helius_ws_url: "wss://devnet.helius-rpc.com".to_string(),
                quicknode_api_key: "test".to_string(),
                quicknode_ws_url: "wss://test.quiknode.pro".to_string(),
            },
            database: DatabaseConfig {
                url: "test".to_string(),
            },
            server: ServerConfig { port: 8080 },
            logging: LoggingConfig {
                level: "info".to_string(),
            },
            overmind: OvermindConfig {
                enabled: false,
                tensorzero_gateway_url: "http://localhost:3000".to_string(),
                jito_endpoint: "https://mainnet.block-engine.jito.wtf".to_string(),
                max_execution_latency_ms: 25,
                ai_confidence_threshold: 0.7,
            },
        };

        assert!(!config.is_live_trading());
        assert_eq!(config.trading_mode_str(), "paper");
    }
}
