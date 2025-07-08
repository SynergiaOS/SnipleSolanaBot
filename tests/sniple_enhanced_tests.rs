// THE OVERMIND PROTOCOL - SnipleSolanaBot Enhanced Tests
// Comprehensive integration tests for enhanced SnipleSolanaBot modules

use anyhow::Result;
use overmind_protocol::modules::{
    vault::{Vault, VaultConfig},
    jito_bundler::{JitoBundler, JitoError},
    dex_aggregator::DexAggregator,
    sniple_config::SnipleConfig,
};
use std::env;
use tempfile::TempDir;
use tokio_test;

#[tokio::test]
async fn test_vault_encryption_decryption() -> Result<()> {
    // Set up temporary master key
    env::set_var("VAULT_MASTER_KEY", "test_master_key_32_bytes_long_123456");
    
    let vault = Vault::new()?;
    
    // Test encryption/decryption
    let plaintext = "secret_api_key_12345";
    let encrypted = vault.encrypt(plaintext)?;
    let decrypted = vault.decrypt(&encrypted)?;
    
    assert_eq!(plaintext, decrypted);
    assert_ne!(encrypted, plaintext.as_bytes());
    
    Ok(())
}

#[tokio::test]
async fn test_vault_env_fallback() -> Result<()> {
    // Set up environment
    env::set_var("VAULT_MASTER_KEY", "test_master_key_32_bytes_long_123456");
    env::set_var("TEST_SECRET", "env_secret_value");
    
    let mut vault = Vault::new()?;
    
    // Should get from environment first
    let secret = vault.get_env_or_vault("TEST_SECRET")?;
    assert_eq!(secret, "env_secret_value");
    
    // Clean up
    env::remove_var("TEST_SECRET");
    
    Ok(())
}

#[tokio::test]
async fn test_vault_file_storage() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let secrets_path = temp_dir.path().join("secrets");
    
    env::set_var("VAULT_MASTER_KEY", "test_master_key_32_bytes_long_123456");
    
    let config = VaultConfig {
        master_key_env: "VAULT_MASTER_KEY".to_string(),
        secrets_dir: secrets_path.to_string_lossy().to_string(),
        encryption_enabled: true,
        fallback_to_env: true,
        key_rotation_enabled: false,
    };
    
    let mut vault = Vault::with_config(config)?;
    
    // Store and retrieve secret
    vault.store_secret("test_key", "test_value")?;
    let retrieved = vault.get_env_or_vault("test_key")?;
    
    assert_eq!(retrieved, "test_value");
    
    Ok(())
}

#[tokio::test]
async fn test_vault_integrity_validation() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let secrets_path = temp_dir.path().join("secrets");
    
    env::set_var("VAULT_MASTER_KEY", "test_master_key_32_bytes_long_123456");
    
    let config = VaultConfig {
        master_key_env: "VAULT_MASTER_KEY".to_string(),
        secrets_dir: secrets_path.to_string_lossy().to_string(),
        encryption_enabled: true,
        fallback_to_env: true,
        key_rotation_enabled: false,
    };
    
    let mut vault = Vault::with_config(config)?;
    
    // Store multiple secrets
    vault.store_secret("key1", "value1")?;
    vault.store_secret("key2", "value2")?;
    vault.store_secret("key3", "value3")?;
    
    // Validate integrity
    let integrity_ok = vault.validate_integrity()?;
    assert!(integrity_ok);
    
    Ok(())
}

#[tokio::test]
async fn test_jito_bundler_creation() {
    let bundler = JitoBundler::new(
        "test_auth_key".to_string(),
        "https://test.endpoint.com".to_string(),
    );
    
    let bundle = bundler.create_bundle(
        vec!["tx1".to_string(), "tx2".to_string()],
        "96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5".to_string(),
        50000,
    );
    
    assert_eq!(bundle.transactions.len(), 2);
    assert_eq!(bundle.tip_amount, 50000);
    assert!(!bundle.bundle_id.is_empty());
}

#[tokio::test]
async fn test_jito_bundler_validation() {
    let bundler = JitoBundler::new(
        "test_auth_key".to_string(),
        "https://test.endpoint.com".to_string(),
    );
    
    // Valid bundle
    let valid_bundle = bundler.create_bundle(
        vec!["tx1".to_string()],
        "96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5".to_string(),
        50000,
    );
    
    assert!(bundler.validate_bundle(&valid_bundle).is_ok());
    
    // Empty bundle (invalid)
    let empty_bundle = bundler.create_bundle(
        vec![],
        "96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5".to_string(),
        50000,
    );
    
    assert!(bundler.validate_bundle(&empty_bundle).is_err());
    
    // Too many transactions (invalid)
    let large_bundle = bundler.create_bundle(
        vec!["tx1".to_string(); 10], // 10 transactions (max is 5)
        "96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5".to_string(),
        50000,
    );
    
    assert!(bundler.validate_bundle(&large_bundle).is_err());
}

#[tokio::test]
async fn test_dex_aggregator_creation() -> Result<()> {
    let aggregator = DexAggregator::new()?;
    
    // Test cache statistics
    let (entry_count, cache_size) = aggregator.get_cache_stats().await;
    assert_eq!(entry_count, 0); // Empty cache initially
    
    Ok(())
}

#[tokio::test]
async fn test_dex_aggregator_cache() -> Result<()> {
    let aggregator = DexAggregator::new()?;
    
    // Clear cache
    aggregator.clear_cache().await;
    
    let (entry_count, _) = aggregator.get_cache_stats().await;
    assert_eq!(entry_count, 0);
    
    Ok(())
}

#[tokio::test]
async fn test_sniple_config_default() -> Result<()> {
    let config = SnipleConfig::default();
    
    assert_eq!(config.trading_config.trading_mode, "paper");
    assert!(config.risk_limits.max_position_size_usd > 0.0);
    assert!(!config.rpc_endpoints.is_empty());
    assert!(config.jito_enabled());
    
    Ok(())
}

#[tokio::test]
async fn test_sniple_config_validation() -> Result<()> {
    let config = SnipleConfig::default();
    
    // Should validate successfully
    assert!(config.validate().is_ok());
    
    // Test invalid config
    let mut invalid_config = config.clone();
    invalid_config.risk_limits.max_position_size_usd = -100.0; // Invalid negative value
    
    assert!(invalid_config.validate().is_err());
    
    Ok(())
}

#[tokio::test]
async fn test_sniple_config_environment_specific() -> Result<()> {
    let dev_config = SnipleConfig::for_environment("development")?;
    let prod_config = SnipleConfig::for_environment("production")?;
    
    assert_eq!(dev_config.trading_config.trading_mode, "paper");
    assert_eq!(prod_config.trading_config.trading_mode, "live");
    
    assert!(dev_config.risk_limits.max_position_size_usd < prod_config.risk_limits.max_position_size_usd);
    assert!(prod_config.security_config.multi_sig_required);
    
    Ok(())
}

#[tokio::test]
async fn test_sniple_config_env_vars() -> Result<()> {
    let config = SnipleConfig::default();
    let env_vars = config.to_env_vars();
    
    assert!(env_vars.contains_key("SNIPLE_TRADING_MODE"));
    assert!(env_vars.contains_key("SNIPLE_MAX_POSITION_SIZE_USD"));
    assert!(env_vars.contains_key("SNIPLE_MAX_DAILY_LOSS_PERCENT"));
    
    Ok(())
}

#[tokio::test]
async fn test_sniple_config_update_from_env() -> Result<()> {
    env::set_var("SNIPLE_TRADING_MODE", "live");
    env::set_var("SNIPLE_MAX_POSITION_SIZE_USD", "2000.0");
    
    let mut config = SnipleConfig::default();
    config.update_from_env()?;
    
    assert_eq!(config.trading_config.trading_mode, "live");
    assert_eq!(config.risk_limits.max_position_size_usd, 2000.0);
    
    // Clean up
    env::remove_var("SNIPLE_TRADING_MODE");
    env::remove_var("SNIPLE_MAX_POSITION_SIZE_USD");
    
    Ok(())
}

#[tokio::test]
async fn test_sniple_config_save_load() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("test_config.toml");
    
    let original_config = SnipleConfig::default();
    original_config.save_to_file(config_path.to_str().unwrap())?;
    
    // Verify file was created
    assert!(config_path.exists());
    
    // Read file content
    let content = std::fs::read_to_string(&config_path)?;
    assert!(content.contains("trading_mode"));
    assert!(content.contains("max_position_size_usd"));
    
    Ok(())
}

// Integration tests that would require network access
#[cfg(feature = "integration")]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_dex_aggregator_real_quotes() -> Result<()> {
        let aggregator = DexAggregator::new()?;
        
        // SOL to USDC
        let sol_mint = "So11111111111111111111111111111111111111112";
        let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
        let amount = 1_000_000_000; // 1 SOL
        
        let quote = aggregator.get_best_quote(sol_mint, usdc_mint, amount).await?;
        
        assert!(!quote.all_quotes.is_empty());
        assert!(quote.best_quote.output_amount > 0);
        assert!(!quote.recommended_dex.is_empty());
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_cluster_orchestrator_real_redis() -> Result<()> {
        // This test requires a running Redis instance
        // Skip if Redis is not available
        if env::var("REDIS_URL").is_err() {
            return Ok(());
        }
        
        let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
        let main_rpc = "https://api.mainnet-beta.solana.com".to_string();
        let backup_rpcs = vec![
            "https://solana-api.projectserum.com".to_string(),
        ];
        
        // This would test real Redis connectivity
        // let orchestrator = ClusterOrchestrator::new(
        //     &redis_url,
        //     main_rpc,
        //     backup_rpcs,
        //     true,
        // ).await?;
        
        // let current_rpc = orchestrator.get_current_rpc().await;
        // assert!(!current_rpc.is_empty());
        
        Ok(())
    }
}

// Performance benchmarks
#[cfg(feature = "bench")]
mod benchmarks {
    use super::*;
    use std::time::Instant;
    
    #[tokio::test]
    async fn bench_vault_encryption() -> Result<()> {
        env::set_var("VAULT_MASTER_KEY", "test_master_key_32_bytes_long_123456");
        let vault = Vault::new()?;
        
        let plaintext = "test_secret_value_for_benchmarking";
        let iterations = 1000;
        
        let start = Instant::now();
        for _ in 0..iterations {
            let encrypted = vault.encrypt(plaintext)?;
            let _decrypted = vault.decrypt(&encrypted)?;
        }
        let duration = start.elapsed();
        
        println!("Vault encryption/decryption: {} ops in {:?} ({:.2} ops/sec)", 
                 iterations, duration, iterations as f64 / duration.as_secs_f64());
        
        Ok(())
    }
    
    #[tokio::test]
    async fn bench_dex_aggregator_cache() -> Result<()> {
        let aggregator = DexAggregator::new()?;
        
        // This would benchmark cache performance
        // Implementation depends on having mock data
        
        Ok(())
    }
}
