/*
THE OVERMIND PROTOCOL - Module Organization
Organized into logical groups for better maintainability and clarity
*/

// ============================================================================
// CORE SYSTEM MODULES
// ============================================================================
pub mod data_ingestor;
pub mod executor;
pub mod persistence;
pub mod error_handling;
pub mod metrics;

// ============================================================================
// AI & INTELLIGENCE MODULES
// ============================================================================
pub mod ai_connector;
pub mod deepseek_connector;
pub mod enhanced_ai_brain;
pub mod jina_ai_connector;
pub mod tensorzero_client;

// ============================================================================
// TRADING & EXECUTION MODULES
// ============================================================================
pub mod strategy;
pub mod risk;
pub mod hft_engine;
pub mod real_sell_executor;

// ============================================================================
// DEX & MARKET DATA MODULES
// ============================================================================
pub mod dex_integration;
pub mod hybrid_price_fetcher;
pub mod real_price_fetcher;
pub mod jupiter_dex;
pub mod rpc_failover;

// ============================================================================
// JITO & MEV MODULES (THE OVERMIND PROTOCOL)
// ============================================================================
pub mod jito_client;
pub mod jito_v2_client;
pub mod advanced_mev_engine;
pub mod advanced_mev_strategies;
pub mod overmind_mev_pipeline;

// ============================================================================
// HELIUS INTEGRATION (THE OVERMIND PROTOCOL)
// ============================================================================
pub mod helius_streamer;

// ============================================================================
// WALLET & SECURITY MODULES
// ============================================================================
pub mod wallet_manager;
pub mod wallet_orchestrator;
pub mod dynamic_wallet_generator;
pub mod multi_wallet_config;
pub mod multi_wallet_executor;
pub mod multi_wallet_load_balancer;
pub mod encrypted_key_storage;
pub mod secure_wallet_manager;

// ============================================================================
// ADVANCED STRATEGIES
// ============================================================================
pub mod cross_dex_arbitrage;
pub mod liquidity_sniping;
pub mod mev_arbitrage;
pub mod dev_tracker;
pub mod meteora_damm;
pub mod soul_meteor;
pub mod rugpull_scanner;

// ============================================================================
// PERFORMANCE & OPTIMIZATION MODULES
// ============================================================================
pub mod memory_optimizer;
pub mod performance_optimizer;
pub mod realtime_monitor;
pub mod submillisecond_optimizer;

// ============================================================================
// RISK MANAGEMENT MODULES
// ============================================================================
pub mod advanced_risk_management;
pub mod dynamic_position_sizing;
pub mod portfolio_rebalancer;
pub mod profit_manager;
pub mod resource_manager;

// ============================================================================
// SNIPLE SOLANA BOT ENHANCED MODULES
// ============================================================================
pub mod cluster_orchestrator;
pub mod vault;
pub mod jito_bundler;
pub mod dex_aggregator;
pub mod sniple_config;

// Re-export main types for easier access
// Note: Exports commented out to avoid unused import warnings in skeleton
// pub use data_ingestor::DataIngestor;
// pub use executor::Executor;
// pub use persistence::PersistenceManager;
// pub use risk::RiskManager;
// pub use strategy::StrategyEngine;
